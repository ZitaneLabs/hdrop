use diesel_async::{
    pooled_connection::{deadpool::Pool, AsyncDieselConnectionManager},
    RunQueryDsl,
};
use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};

use super::split_tls_options;

#[test]
fn extracts_tls_options_without_changing_other_connection_settings() {
    let (url, options) = split_tls_options(
        "postgresql://user:pa%2Bss@db-a:5432,db-b:5433/app?application_name=a+b&sslrootcert=C%3A%2FCA%20certs%2Froot%2Bca.pem&sslmode=verify-full",
    ).unwrap();
    assert_eq!(
        url,
        "postgresql://user:pa%2Bss@db-a:5432,db-b:5433/app?application_name=a+b"
    );
    assert_eq!(options["sslrootcert"], "C:/CA certs/root+ca.pem");
    assert_eq!(options["sslmode"], "verify-full");
    let config: tokio_postgres::Config = url.parse().unwrap();
    assert_eq!(config.get_hosts().len(), 2);
    assert_eq!(config.get_ports(), &[5432, 5433]);
    assert_eq!(config.get_password(), Some(b"pa+ss".as_slice()));
    assert_eq!(config.get_application_name(), Some("a+b"));

    let (url, options) = split_tls_options(
        r#"host=db-a,db-b application_name='worker one' password='it\'s safe' sslrootcert='C:\\CA certs\\root.crt' sslkey=client\ key.pem sslmode=verify-ca"#,
    ).unwrap();
    assert_eq!(options["sslrootcert"], r"C:\CA certs\root.crt");
    assert_eq!(options["sslkey"], "client key.pem");
    assert_eq!(options["sslmode"], "verify-ca");
    let config: tokio_postgres::Config = url.parse().unwrap();
    assert_eq!(config.get_hosts().len(), 2);
    assert_eq!(config.get_application_name(), Some("worker one"));
    assert_eq!(config.get_password(), Some(b"it's safe".as_slice()));
}

#[test]
fn rejects_malformed_settings_and_preserves_unknown_options_for_validation() {
    for input in [
        "postgres://localhost/db?sslrootcert",
        "postgres://localhost/db?sslrootcert=%FF",
        "postgres://localhost/db?%FF=value",
        "host localhost",
        "sslrootcert='unterminated",
        "sslrootcert=unterminated\\",
    ] {
        assert!(split_tls_options(input).is_err(), "accepted {input:?}");
    }
    for input in [
        "postgres://localhost/db?sslmode=require&sslunknown=value",
        "host=localhost sslmode=require sslunknown=value",
    ] {
        let (url, options) = split_tls_options(input).unwrap();
        assert_eq!(options["sslmode"], "require");
        assert!(url.contains("sslunknown=value"));
        assert!(url.parse::<tokio_postgres::Config>().is_err());
    }
}

fn connection_url(base: &str, mode: Option<&str>, root: Option<&str>) -> String {
    let mut url = base.to_owned();
    for (key, value) in [("sslmode", mode), ("sslrootcert", root)] {
        if let Some(value) = value {
            url.push(if url.contains('?') { '&' } else { '?' });
            url.push_str(key);
            url.push('=');
            url.push_str(&utf8_percent_encode(value, NON_ALPHANUMERIC).to_string());
        }
    }
    url
}

async fn uses_tls(url: String) -> super::SetupResult<bool> {
    #[derive(diesel::QueryableByName)]
    struct TlsState {
        #[diesel(sql_type = diesel::sql_types::Bool)]
        ssl: bool,
    }

    let manager = AsyncDieselConnectionManager::new_with_config(url, super::config());
    let pool = Pool::builder(manager).max_size(1).build()?;
    let timeout = std::time::Duration::from_secs(10);
    let mut conn = tokio::time::timeout(timeout, pool.get()).await??;
    let state = tokio::time::timeout(
        timeout,
        diesel::sql_query("SELECT ssl FROM pg_stat_ssl WHERE pid = pg_backend_pid()")
            .get_result::<TlsState>(&mut conn),
    )
    .await??;
    Ok(state.ssl)
}

#[tokio::test]
#[ignore = "requires TLS/plain PostgreSQL and TEST_TLS_* / TEST_PLAIN_DATABASE_URL fixtures"]
async fn tls_connection_modes() {
    let required =
        |name| std::env::var(name).unwrap_or_else(|_| panic!("set {name} to run TLS tests"));
    let tls = required("TEST_TLS_DATABASE_URL");
    let plain = required("TEST_PLAIN_DATABASE_URL");
    let ca = required("TEST_TLS_CA_CERT");
    let wrong_ca = required("TEST_TLS_WRONG_CA_CERT");
    let client_cert = required("TEST_TLS_CLIENT_CERT");
    let client_key = required("TEST_TLS_CLIENT_KEY");

    for (base, mode, root, expected) in [
        (&tls, None, None, true),
        (&tls, Some("prefer"), None, true),
        (&tls, Some("require"), None, true),
        (&tls, Some("prefer"), Some(ca.as_str()), true),
        (&tls, Some("require"), Some(ca.as_str()), true),
        (&tls, Some("verify-ca"), Some(ca.as_str()), true),
        (&tls, Some("verify-full"), Some(ca.as_str()), true),
        (&tls, Some("disable"), None, false),
        (&tls, Some("allow"), None, false),
        (&plain, None, None, false),
        (&plain, Some("prefer"), None, false),
    ] {
        assert_eq!(
            uses_tls(connection_url(base, mode, root))
                .await
                .unwrap_or_else(|error| panic!("{mode:?}, {root:?}: {error}")),
            expected,
            "unexpected TLS state for {mode:?}",
        );
    }
    assert!(uses_tls(connection_url(&plain, Some("require"), None))
        .await
        .is_err());
    for mode in ["require", "verify-ca", "verify-full"] {
        assert!(
            uses_tls(connection_url(&tls, Some(mode), Some(&wrong_ca)))
                .await
                .is_err(),
            "{mode} accepted the wrong CA"
        );
    }
    let mismatch = tls.replacen("localhost", "127.0.0.1", 1);
    assert_ne!(mismatch, tls, "TEST_TLS_DATABASE_URL must use localhost");
    assert!(
        uses_tls(connection_url(&mismatch, Some("verify-full"), Some(&ca)))
            .await
            .is_err()
    );
    assert!(
        uses_tls(connection_url(&mismatch, Some("verify-ca"), Some(&ca)))
            .await
            .unwrap()
    );
    let missing_ca = format!("{ca}.does-not-exist");
    assert!(
        uses_tls(connection_url(&tls, Some("require"), Some(&missing_ca)))
            .await
            .is_err()
    );

    let client_base = tls.replacen("://postgres:", "://hdrop_tls_client:", 1);
    assert_ne!(client_base, tls, "TEST_TLS_DATABASE_URL must use postgres");
    let client_url = connection_url(&client_base, Some("verify-full"), Some(&ca));
    assert!(
        uses_tls(client_url.clone()).await.is_err(),
        "client certificate must be required"
    );
    let client_url = format!(
        "{client_url}&sslcert={}&sslkey={}",
        utf8_percent_encode(&client_cert, NON_ALPHANUMERIC),
        utf8_percent_encode(&client_key, NON_ALPHANUMERIC),
    );
    assert!(uses_tls(client_url.clone()).await.unwrap());
    assert!(
        uses_tls(client_url.replace("sslmode=verify%2Dfull", "sslmode=allow"))
            .await
            .unwrap()
    );
}
