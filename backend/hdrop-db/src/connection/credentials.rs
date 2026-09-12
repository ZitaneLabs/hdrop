use std::{env, fmt::Write, path::PathBuf};

use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};
use rand::seq::SliceRandom;
use tokio_postgres::{
    config::{Host, LoadBalanceHosts},
    Config,
};

use super::{env_value, pgpass, Options, SetupResult};

#[cfg(target_os = "linux")]
const DEFAULT_HOST: &str = "/var/run/postgresql";
#[cfg(all(unix, not(target_os = "linux")))]
const DEFAULT_HOST: &str = "/tmp";
#[cfg(windows)]
const DEFAULT_HOST: &str = "localhost";

/// Give each host its own password and transport retries, in libpq failover order.
pub(super) fn endpoints(
    base: &str,
    mut options: Options,
) -> SetupResult<(Vec<Config>, Option<PathBuf>)> {
    // The driver has no way to replace a Config's hosts. Parse the authority
    // separately so we can clone the shared credentials and options per host.
    let (_, location) = base.split_once("://").unwrap();
    let (authority, database) = location.split_once('/').unwrap_or((location, ""));
    let (userinfo, hosts) = authority.split_at(authority.rfind('@').map_or(0, |i| i + 1));
    let uri: Config = format!("postgres://{hosts}").parse()?;
    // The driver fills in 5432 when a URI omits its port. Allow PGPORT to
    // supply the actual default, without mistaking an IPv6 colon for a port.
    let has_port = hosts.split(',').any(|host| {
        let tail = host.rsplit_once(']').map_or(host, |(_, tail)| tail);
        tail.split_once(':')
            .is_some_and(|(_, port)| !port.is_empty())
    });
    let mut address = |key, variable, has_authority| -> SetupResult<Option<String>> {
        match options.remove(key) {
            Some(value) => Ok(Some(value)),
            None if has_authority => Ok(None),
            None => env_value(variable),
        }
    };
    let mut host_config = Config::new();
    let hosts = if let Some(value) = address("host", "PGHOST", !uri.get_hosts().is_empty())? {
        for host in value.split(',') {
            host_config.host(host);
        }
        host_config.get_hosts()
    } else {
        uri.get_hosts()
    };
    let ports = match address("port", "PGPORT", has_port)? {
        Some(value) => value
            .split(',')
            .map(|s| if s.is_empty() { Ok(5432) } else { s.parse() })
            .collect::<Result<Vec<u16>, _>>()?,
        None if has_port => uri.get_ports().to_vec(),
        None => Vec::new(),
    };
    let hostaddr = address("hostaddr", "PGHOSTADDR", false)?;
    let addrs: Vec<_> = hostaddr.iter().flat_map(|s| s.split(',')).collect();
    let count = hosts.len().max(addrs.len()).max(1);
    if (!hosts.is_empty() && hosts.len() != count)
        || (!addrs.is_empty() && addrs.len() != count)
        || (ports.len() > 1 && ports.len() != count)
    {
        return Err("number of PostgreSQL hosts, addresses or ports does not match".into());
    }

    let passfile = options.remove("passfile");
    let mut common = format!("postgresql://{userinfo}/{database}?");
    for (key, value) in options {
        write!(
            common,
            "{}={}&",
            utf8_percent_encode(&key, NON_ALPHANUMERIC),
            utf8_percent_encode(&value, NON_ALPHANUMERIC)
        )?;
    }
    let mut template: Config = common.parse()?;
    if template.get_user().is_none_or(str::is_empty) {
        template.user(match env_value("PGUSER")? {
            Some(user) => user,
            None => whoami::username()?,
        });
    }
    if template.get_dbname().is_none_or(str::is_empty) {
        let name =
            env_value("PGDATABASE")?.unwrap_or_else(|| template.get_user().unwrap().to_owned());
        template.dbname(name);
    }
    let password_file = if template.get_password().is_some_and(|p| !p.is_empty()) {
        None
    } else if let Some(password) = env::var_os("PGPASSWORD").filter(|p| !p.is_empty()) {
        template.password(password.as_encoded_bytes());
        None
    } else {
        passfile
            .filter(|s| !s.is_empty())
            .map(PathBuf::from)
            .or_else(|| {
                env::var_os("PGPASSFILE")
                    .filter(|p| !p.is_empty())
                    .map(PathBuf::from)
            })
            .or_else(default_passfile)
    };
    let mut configs = Vec::with_capacity(count);
    for i in 0..count {
        let mut config = template.clone();
        let addr = addrs.get(i).copied().filter(|s| !s.is_empty());
        match hosts.get(i) {
            Some(Host::Tcp(host)) if !host.is_empty() => {
                config.host(host);
            }
            #[cfg(unix)]
            Some(Host::Unix(path)) => {
                config.host_path(path);
            }
            _ if addr.is_none() => {
                config.host(DEFAULT_HOST);
            }
            _ => {}
        };
        if let Some(addr) = addr {
            config.hostaddr(addr.parse()?);
        }
        config.port(
            ports
                .get(i)
                .or_else(|| ports.first())
                .copied()
                .unwrap_or(5432),
        );
        configs.push(config);
    }
    Ok((configs, password_file))
}

pub(super) fn with_passwords(templates: &[Config], passwords: &[u8]) -> Vec<Config> {
    let mut configs = templates.to_vec();
    if !passwords.is_empty() {
        for config in &mut configs {
            let address;
            let host = match config.get_hosts().first() {
                Some(Host::Tcp(host)) => host.as_bytes(),
                #[cfg(unix)]
                Some(Host::Unix(path)) if path == std::path::Path::new(DEFAULT_HOST) => {
                    b"localhost"
                }
                #[cfg(unix)]
                Some(Host::Unix(path)) => path.as_os_str().as_encoded_bytes(),
                None => {
                    address = config.get_hostaddrs()[0].to_string();
                    address.as_bytes()
                }
            };
            let port = config.get_ports()[0].to_string();
            let fields = [
                host,
                port.as_bytes(),
                config.get_dbname().unwrap().as_bytes(),
                config.get_user().unwrap().as_bytes(),
            ];
            if let Some(password) = pgpass::password(passwords, fields) {
                config.password(password);
            }
        }
    }
    if configs
        .first()
        .is_some_and(|config| config.get_load_balance_hosts() == LoadBalanceHosts::Random)
    {
        configs.shuffle(&mut rand::rng());
    }
    configs
}

#[cfg(unix)]
fn default_passfile() -> Option<PathBuf> {
    env::home_dir().map(|p| p.join(".pgpass"))
}

#[cfg(windows)]
fn default_passfile() -> Option<PathBuf> {
    env::var_os("APPDATA").map(|p| PathBuf::from(p).join("postgresql/pgpass.conf"))
}
