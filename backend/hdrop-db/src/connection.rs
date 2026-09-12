use std::{collections::BTreeMap, env, fs, path::PathBuf};

use diesel::{ConnectionError, ConnectionResult};
use diesel_async::{pooled_connection::ManagerConfig, AsyncPgConnection};
use native_tls::{Certificate, Identity, TlsConnector};
use postgres_native_tls::MakeTlsConnector;
use tokio_postgres::{config::SslMode, Config, NoTls};

pub fn config() -> ManagerConfig<AsyncPgConnection> {
    let mut config = ManagerConfig::default();
    config.custom_setup = Box::new(|url| Box::pin(establish(url)));
    config
}

type SetupResult<T> = Result<T, Box<dyn std::error::Error + Send + Sync>>;

async fn establish(url: &str) -> ConnectionResult<AsyncPgConnection> {
    connect(url)
        .await
        .map_err(|error| ConnectionError::BadConnection(error.to_string()))
}

async fn connect(url: &str) -> SetupResult<AsyncPgConnection> {
    let (url, mut options) = split_tls_options(url)?;
    // Never silently drop a libpq security policy the TLS adapter cannot enforce.
    for (name, _) in env::vars_os() {
        if let Some(name) = name.to_str() {
            #[cfg(windows)]
            let name = name.to_ascii_uppercase();
            let name: &str = &name;
            if (name.starts_with("PGSSL")
                && !matches!(
                    name,
                    "PGSSLMODE" | "PGSSLROOTCERT" | "PGSSLCERT" | "PGSSLKEY"
                ))
                || name == "PGREQUIRESSL"
            {
                return Err(format!("unsupported PostgreSQL TLS setting: {name}").into());
            }
        }
    }
    for name in ["sslmode", "sslrootcert", "sslcert", "sslkey"] {
        if !options.contains_key(name) {
            match env::var(format!("PG{}", name.to_uppercase())) {
                Ok(value) if !value.is_empty() => {
                    options.insert(name.to_owned(), value);
                }
                Ok(_) => {}
                Err(env::VarError::NotPresent) => {}
                Err(error) => return Err(error.into()),
            }
        }
    }
    let system_roots = options
        .get("sslrootcert")
        .is_some_and(|path| path == "system");
    let mode = options
        .get("sslmode")
        .map(String::as_str)
        .unwrap_or(if system_roots {
            "verify-full"
        } else {
            "prefer"
        });
    let mut config: Config = url.parse()?;
    config.ssl_mode(match mode {
        "disable" | "allow" => SslMode::Disable,
        "prefer" => SslMode::Prefer,
        "require" | "verify-ca" | "verify-full" => SslMode::Require,
        _ => return Err("invalid PostgreSQL sslmode".into()),
    });
    if system_roots && mode != "verify-full" {
        return Err("sslrootcert=system requires sslmode=verify-full".into());
    }
    if mode == "disable" || mode == "allow" {
        match config.connect(NoTls).await {
            Ok((client, connection)) => {
                return Ok(
                    AsyncPgConnection::try_from_client_and_connection(client, connection).await?,
                );
            }
            Err(_) if mode == "allow" => {
                config.ssl_mode(SslMode::Require);
            }
            Err(error) => return Err(error.into()),
        }
    }

    let default_dir = if cfg!(windows) {
        env::var_os("APPDATA").map(|path| PathBuf::from(path).join("postgresql"))
    } else {
        env::home_dir().map(|path| path.join(".postgresql"))
    };
    if default_dir
        .as_ref()
        .is_some_and(|dir| dir.join("root.crl").exists())
    {
        return Err("PostgreSQL root.crl is unsupported by the TLS adapter".into());
    }
    let certificate_path = |name: &str, default: &str| {
        options.get(name).map(PathBuf::from).or_else(|| {
            default_dir
                .as_ref()
                .map(|dir| dir.join(default))
                .filter(|path| path.is_file())
        })
    };
    let root = certificate_path("sslrootcert", "root.crt");
    let verify = root.is_some() || matches!(mode, "verify-ca" | "verify-full");
    if verify && root.is_none() {
        return Err("certificate verification requires sslrootcert (or PGSSLROOTCERT)".into());
    }
    let mut tls = TlsConnector::builder();
    // Match libpq: require encrypts; a configured CA also enables chain validation.
    tls.danger_accept_invalid_certs(!verify)
        .danger_accept_invalid_hostnames(mode != "verify-full")
        .disable_built_in_roots(!system_roots);
    if let Some(root) = root.filter(|_| !system_roots) {
        let certs = Certificate::stack_from_pem(&fs::read(root)?)?;
        if certs.is_empty() {
            return Err("sslrootcert contains no certificates".into());
        }
        for cert in certs {
            tls.add_root_certificate(cert);
        }
    }
    if let Some(cert) = certificate_path("sslcert", "postgresql.crt") {
        let key = certificate_path("sslkey", "postgresql.key").ok_or("sslcert requires sslkey")?;
        tls.identity(Identity::from_pkcs8(&fs::read(cert)?, &fs::read(key)?)?);
    } else if options.contains_key("sslkey") {
        return Err("sslkey requires sslcert".into());
    }
    postgres_native_tls::set_postgresql_alpn(&mut tls);
    let (client, connection) = config.connect(MakeTlsConnector::new(tls.build()?)).await?;
    Ok(AsyncPgConnection::try_from_client_and_connection(client, connection).await?)
}

// Keep tokio-postgres responsible for parsing connection settings. Only extract
// the libpq TLS options it does not understand; preserve all other text verbatim.
fn split_tls_options(input: &str) -> SetupResult<(String, BTreeMap<String, String>)> {
    let is_uri = input.starts_with("postgres://") || input.starts_with("postgresql://");
    let mut options = BTreeMap::new();
    let mut remaining = Vec::new();
    let mut keep = |raw: &str, key: &str, value: String| {
        if matches!(key, "sslmode" | "sslrootcert" | "sslcert" | "sslkey") {
            options.insert(key.to_owned(), value);
        } else {
            remaining.push(raw.to_owned());
        }
    };
    if is_uri {
        let (base, query) = input.split_once('?').unwrap_or((input, ""));
        for raw in query.split('&').filter(|part| !part.is_empty()) {
            let (key, value) = raw
                .split_once('=')
                .ok_or("invalid database URL parameter")?;
            let key = percent_encoding::percent_decode_str(key).decode_utf8()?;
            let value = percent_encoding::percent_decode_str(value)
                .decode_utf8()?
                .into_owned();
            keep(raw, &key, value);
        }
        Ok((format!("{base}?{}", remaining.join("&")), options))
    } else {
        let mut chars = input.char_indices().peekable();
        while chars.peek().is_some() {
            while chars.peek().is_some_and(|(_, c)| c.is_whitespace()) {
                chars.next();
            }
            let Some(&(start, _)) = chars.peek() else {
                break;
            };
            let mut key = String::new();
            while let Some(&(_, c)) = chars.peek() {
                if c == '=' || c.is_whitespace() {
                    break;
                }
                key.push(c);
                chars.next();
            }
            while chars.peek().is_some_and(|(_, c)| c.is_whitespace()) {
                chars.next();
            }
            if chars.next().map(|(_, c)| c) != Some('=') {
                return Err("invalid database connection parameter".into());
            }
            while chars.peek().is_some_and(|(_, c)| c.is_whitespace()) {
                chars.next();
            }
            let quoted = chars.peek().is_some_and(|(_, c)| *c == '\'');
            if quoted {
                chars.next();
            }
            let mut value = String::new();
            loop {
                match chars.next() {
                    Some((_, '\\')) => {
                        value.push(chars.next().ok_or("unterminated connection escape")?.1)
                    }
                    Some((_, '\'')) if quoted => break,
                    Some((_, c)) if !quoted && c.is_whitespace() => break,
                    Some((_, c)) => value.push(c),
                    None if quoted => return Err("unterminated connection quote".into()),
                    None => break,
                }
            }
            let end = chars.peek().map(|(index, _)| *index).unwrap_or(input.len());
            keep(&input[start..end], &key, value);
        }
        Ok((remaining.join(" "), options))
    }
}

#[cfg(test)]
mod tests;
