use std::{
    collections::BTreeMap,
    env,
    error::Error,
    fmt::Write,
    fs,
    io::{self, Read},
    path::{Path, PathBuf},
    sync::Arc,
};

use diesel::{ConnectionError, ConnectionResult};
use diesel_async::{pooled_connection::ManagerConfig, AsyncPgConnection};
use native_tls::{Certificate, Identity, TlsConnector};
use postgres_native_tls::MakeTlsConnector;
use tokio_postgres::{
    config::{ChannelBinding, SslMode, SslNegotiation},
    error::SqlState,
    tls::MakeTlsConnect,
    Config,
    NoTls,
    Socket,
};

mod credentials;
mod pgpass;

pub fn config(url: &str) -> ConnectionResult<ManagerConfig<AsyncPgConnection>> {
    let settings = Arc::new(settings(url).map_err(connection_error)?);
    let mut config = ManagerConfig::default();
    config.custom_setup = Box::new(move |_| {
        let settings = Arc::clone(&settings);
        Box::pin(async move { connect(settings).await.map_err(connection_error) })
    });
    Ok(config)
}

type SetupResult<T> = Result<T, Box<dyn Error + Send + Sync>>;
type Options = BTreeMap<String, String>;
struct Settings {
    endpoints: Vec<Config>,
    passfile: Option<PathBuf>,
    attempts: &'static [SslMode],
    options: Options,
    default_dir: Option<PathBuf>,
    system_roots: bool,
    verify_ca: bool,
    verify_hostname: bool,
}

fn connection_error(error: Box<dyn Error + Send + Sync>) -> ConnectionError {
    let mut message = error.to_string();
    let mut source = error.source();
    while let Some(cause) = source {
        let _ = write!(message, ": {cause}");
        source = cause.source();
    }
    ConnectionError::BadConnection(message)
}

fn settings(url: &str) -> SetupResult<Settings> {
    let (base, mut options) = parse_dsn(url)?;
    if env::var_os("PGSERVICE").is_some_and(|service| !service.is_empty()) {
        return Err("PostgreSQL service definitions (PGSERVICE) are unsupported".into());
    }
    // Never silently drop a libpq security policy the TLS adapter cannot enforce.
    for (name, _) in env::vars_os() {
        if let Some(name) = name.to_str() {
            #[cfg(windows)]
            let name = name.to_ascii_uppercase();
            #[cfg(windows)]
            let name = name.as_str();
            if (name.starts_with("PGSSL")
                && !matches!(
                    name,
                    "PGSSLMODE" | "PGSSLROOTCERT" | "PGSSLCERT" | "PGSSLKEY"
                ))
                || matches!(name, "PGREQUIRESSL" | "PGREQUIREPEER")
            {
                return Err(format!("unsupported PostgreSQL TLS setting: {name}").into());
            }
        }
    }
    for (key, variable) in [
        ("sslmode", "PGSSLMODE"),
        ("sslrootcert", "PGSSLROOTCERT"),
        ("sslcert", "PGSSLCERT"),
        ("sslkey", "PGSSLKEY"),
        ("channel_binding", "PGCHANNELBINDING"),
        ("require_auth", "PGREQUIREAUTH"),
        ("gssencmode", "PGGSSENCMODE"),
        ("application_name", "PGAPPNAME"),
        ("connect_timeout", "PGCONNECT_TIMEOUT"),
        ("load_balance_hosts", "PGLOADBALANCEHOSTS"),
        ("options", "PGOPTIONS"),
        ("target_session_attrs", "PGTARGETSESSIONATTRS"),
    ] {
        if !options.contains_key(key) {
            if let Some(value) = env_value(variable)? {
                options.insert(key.to_owned(), value);
            }
        }
    }
    match options.remove("gssencmode").as_deref() {
        None | Some("disable" | "prefer") => {}
        Some(_) => return Err("unsupported PostgreSQL gssencmode (PGGSSENCMODE)".into()),
    }
    if options
        .remove("require_auth")
        .is_some_and(|value| !value.is_empty())
    {
        return Err("unsupported PostgreSQL require_auth (PGREQUIREAUTH)".into());
    }
    let tls_options: Options = ["sslmode", "sslrootcert", "sslcert", "sslkey"]
        .into_iter()
        .filter_map(|key| options.remove_entry(key))
        .collect();
    let (endpoints, passfile) = credentials::endpoints(base, options)?;
    let binding = endpoints[0].get_channel_binding();
    let system_roots = tls_options
        .get("sslrootcert")
        .is_some_and(|path| path == "system");
    let mode = tls_options
        .get("sslmode")
        .map(String::as_str)
        .unwrap_or(if system_roots {
            "verify-full"
        } else {
            "prefer"
        });
    if system_roots && mode != "verify-full" {
        return Err("sslrootcert=system requires sslmode=verify-full".into());
    }
    let verify_ca = matches!(mode, "verify-ca" | "verify-full");
    let verify_hostname = mode == "verify-full";
    if !matches!(mode, "require" | "verify-ca" | "verify-full")
        && endpoints[0].get_ssl_negotiation() == SslNegotiation::Direct
    {
        return Err("sslnegotiation=direct requires sslmode=require or stronger".into());
    }
    let attempts: &'static [SslMode] = match mode {
        "disable" if binding == ChannelBinding::Require => {
            return Err("channel_binding=require needs TLS".into())
        }
        "disable" => &[SslMode::Disable],
        "allow" | "prefer" if binding == ChannelBinding::Require => &[SslMode::Require],
        "allow" => &[SslMode::Disable, SslMode::Require],
        "prefer" => &[SslMode::Prefer, SslMode::Disable],
        "require" | "verify-ca" | "verify-full" => &[SslMode::Require],
        _ => return Err("invalid PostgreSQL sslmode".into()),
    };
    let default_dir = if cfg!(windows) {
        env::var_os("APPDATA").map(|path| PathBuf::from(path).join("postgresql"))
    } else {
        env::home_dir().map(|path| path.join(".postgresql"))
    };
    Ok(Settings {
        endpoints,
        passfile,
        attempts,
        options: tls_options,
        default_dir,
        system_roots,
        verify_ca,
        verify_hostname,
    })
}

async fn connect(settings: Arc<Settings>) -> SetupResult<AsyncPgConnection> {
    let passwords = match settings.passfile.clone() {
        Some(path) => tokio::task::spawn_blocking(move || pgpass::read(&path)).await?,
        None => Vec::new(),
    };
    let mut tls = None;
    let mut last_error = None;
    for mut endpoint in credentials::with_passwords(&settings.endpoints, &passwords) {
        if endpoint.get_hosts().is_empty() && !settings.verify_hostname {
            // The TLS connector needs a peer name. verify-full requires an explicit host.
            endpoint.host(endpoint.get_hostaddrs()[0].to_string());
        }
        #[cfg(unix)]
        let unix_socket = endpoint.get_hostaddrs().is_empty()
            && matches!(
                endpoint.get_hosts(),
                [tokio_postgres::config::Host::Unix(_)]
            );
        #[cfg(not(unix))]
        let unix_socket = false;
        let attempts = if unix_socket {
            // libpq ignores sslmode on Unix sockets. The driver still enforces channel binding.
            &[SslMode::Disable][..]
        } else {
            settings.attempts
        };
        for (index, &attempt) in attempts.iter().enumerate() {
            endpoint.ssl_mode(attempt);
            let result = if attempt == SslMode::Disable {
                open(&endpoint, NoTls).await
            } else {
                let connector = match &tls {
                    Some(connector) => connector,
                    None => {
                        let settings = Arc::clone(&settings);
                        tls.insert(
                            tokio::task::spawn_blocking(move || tls_connector(&settings)).await??,
                        )
                    }
                };
                open(&endpoint, connector.clone()).await
            };
            match result {
                Ok(connection) => return Ok(connection),
                Err(error) => {
                    // TLS negotiation and authentication rules can differ by transport.
                    // A failed socket or unavailable server should go straight to the next host.
                    let Some(driver_error) = error.downcast_ref::<tokio_postgres::Error>() else {
                        return Err(error);
                    };
                    if index + 1 == attempts.len()
                        && driver_error
                            .code()
                            .is_some_and(|code| code.code().starts_with("28"))
                    {
                        // A reachable host rejected authentication on its last allowed transport.
                        return Err(error);
                    }
                    let retry = driver_error.code() != Some(&SqlState::CANNOT_CONNECT_NOW)
                        && !driver_error
                            .source()
                            .is_some_and(|cause| cause.is::<std::io::Error>());
                    last_error = Some(error);
                    if !retry {
                        break;
                    }
                }
            }
        }
    }
    Err(last_error.unwrap_or_else(|| "no PostgreSQL hosts configured".into()))
}

async fn open<T>(config: &Config, tls: T) -> SetupResult<AsyncPgConnection>
where
    T: MakeTlsConnect<Socket>,
    T::Stream: Send + Unpin + 'static,
{
    let (client, connection) = config.connect(tls).await?;
    Ok(AsyncPgConnection::try_from_client_and_connection(client, connection).await?)
}

fn tls_connector(settings: &Settings) -> SetupResult<MakeTlsConnector> {
    let certificate_path = |name: &str, default: &str| -> std::io::Result<Option<PathBuf>> {
        if let Some(path) = settings.options.get(name) {
            return Ok(Some(PathBuf::from(path)));
        }
        let Some(dir) = &settings.default_dir else {
            return Ok(None);
        };
        let path = dir.join(default);
        Ok(path.try_exists()?.then_some(path))
    };
    if certificate_path("sslcrl", "root.crl")?.is_some() {
        return Err("PostgreSQL root.crl is unsupported by the TLS adapter".into());
    }
    let root = certificate_path("sslrootcert", "root.crt")?;
    let verify = root.is_some() || settings.verify_ca;
    if verify && root.is_none() {
        return Err("certificate verification requires sslrootcert (or PGSSLROOTCERT)".into());
    }
    let mut tls = TlsConnector::builder();
    // Match libpq: require encrypts; a configured CA also enables chain validation.
    tls.danger_accept_invalid_certs(!verify)
        .danger_accept_invalid_hostnames(!settings.verify_hostname)
        .disable_built_in_roots(!settings.system_roots);
    if let Some(root) = root.filter(|_| !settings.system_roots) {
        let certs = Certificate::stack_from_pem(&fs::read(root)?)?;
        if certs.is_empty() {
            return Err("sslrootcert contains no certificates".into());
        }
        for cert in certs {
            tls.add_root_certificate(cert);
        }
    }
    if let Some(cert) = certificate_path("sslcert", "postgresql.crt")? {
        let key = certificate_path("sslkey", "postgresql.key")?.ok_or("sslcert requires sslkey")?;
        let key_bytes = SecretFile::TlsKey.read(&key)?;
        tls.identity(Identity::from_pkcs8(&fs::read(cert)?, &key_bytes)?);
    } else if settings.options.contains_key("sslkey") {
        return Err("sslkey requires sslcert".into());
    }
    postgres_native_tls::set_postgresql_alpn(&mut tls);
    Ok(MakeTlsConnector::new(tls.build()?))
}

enum SecretFile {
    TlsKey,
    Passfile,
}

impl SecretFile {
    fn read(self, path: &Path) -> io::Result<Vec<u8>> {
        let validate = |metadata: fs::Metadata| -> io::Result<()> {
            if !metadata.is_file() {
                return Err(io::Error::other(
                    "expected a regular PostgreSQL secret file",
                ));
            }
            #[cfg(unix)]
            {
                use std::os::unix::fs::MetadataExt;
                let root_key = matches!(self, Self::TlsKey) && metadata.uid() == 0;
                let forbidden = if root_key { 0o037 } else { 0o077 };
                if metadata.mode() & forbidden != 0 {
                    return Err(io::Error::other(
                        "PostgreSQL secret file allows group or world access",
                    ));
                }
            }
            Ok(())
        };
        validate(path.metadata()?)?;
        let mut options = fs::OpenOptions::new();
        options.read(true);
        // A FIFO could replace the file between metadata() and open().
        #[cfg(unix)]
        std::os::unix::fs::OpenOptionsExt::custom_flags(&mut options, libc::O_NONBLOCK);
        let mut file = options.open(path)?;
        validate(file.metadata()?)?;
        let mut contents = Vec::new();
        file.read_to_end(&mut contents)?;
        Ok(contents)
    }
}

fn env_value(key: &str) -> SetupResult<Option<String>> {
    match env::var(key) {
        Ok(value) if !value.is_empty() => Ok(Some(value)),
        Ok(_) | Err(env::VarError::NotPresent) => Ok(None),
        Err(error) => Err(error.into()),
    }
}

// Keep URI credentials and socket paths encoded for the driver. Collect named
// options once so the same precedence rules apply to both forms of DSN.
fn parse_dsn(input: &str) -> SetupResult<(&str, Options)> {
    let mut options = Options::new();
    if input.starts_with("postgres://") || input.starts_with("postgresql://") {
        let (base, query) = input.split_once('?').unwrap_or((input, ""));
        for raw in query.split('&').filter(|part| !part.is_empty()) {
            let (key, value) = raw
                .split_once('=')
                .ok_or("invalid database URL parameter")?;
            let decode = |s: &str| {
                percent_encoding::percent_decode_str(s)
                    .decode_utf8()
                    .map(|v| v.into_owned())
            };
            options.insert(decode(key)?, decode(value)?);
        }
        return Ok((base, options));
    }

    let mut rest = input.trim_start();
    while !rest.is_empty() {
        let (key, value) = rest.split_once('=').ok_or("expected key=value")?;
        let key = key.trim_end();
        if key.is_empty() || key.contains(char::is_whitespace) {
            return Err("invalid database connection parameter".into());
        }
        let mut chars = value.trim_start().chars();
        let quoted = chars.as_str().starts_with('\'');
        if quoted {
            chars.next();
        }
        let mut value = String::new();
        loop {
            match chars.next() {
                Some('\\') => value.push(chars.next().ok_or("unterminated connection escape")?),
                Some('\'') if quoted => break,
                Some(c) if !quoted && c.is_whitespace() => break,
                Some(c) => value.push(c),
                None if quoted => return Err("unterminated connection quote".into()),
                None => break,
            }
        }
        options.insert(key.to_owned(), value);
        rest = chars.as_str().trim_start();
    }
    Ok(("postgresql://", options))
}

#[cfg(test)]
mod tests;
