use std::{env, path::PathBuf, process::Command, time::Duration};

use diesel::sql_types::Bool;
use diesel_async::{
    pooled_connection::{deadpool::Pool, AsyncDieselConnectionManager},
    RunQueryDsl,
};
use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};

#[test]
fn parses_uri_and_keyword_settings() {
    let settings = super::settings(
        "postgresql://user:pa%2Bss%FF@db-a:5432,db-b:5433/app?application_name=old&application_name=a%26b+c&sslrootcert=C%3A%2FCA%20certs%2Froot%2Bca.pem&sslmode=verify-full",
    ).unwrap();
    let endpoints = settings.endpoints;
    assert_eq!(endpoints.len(), 2);
    assert_eq!(endpoints[0].get_ports(), &[5432]);
    assert_eq!(endpoints[1].get_ports(), &[5433]);
    assert_eq!(endpoints[0].get_password(), Some(b"pa+ss\xff".as_slice()));
    assert_eq!(endpoints[0].get_application_name(), Some("a&b+c"));
    assert_eq!(settings.options["sslrootcert"], "C:/CA certs/root+ca.pem");
    assert_eq!(settings.options["sslmode"], "verify-full");

    let settings = super::settings(
        r#"host=old host=db-a,db-b port=1 port=5433 application_name='worker one' password='it\'s safe' sslrootcert='C:\\CA certs\\root.crt' sslkey=client\ key.pem sslmode=verify-ca"#,
    ).unwrap();
    let config = &settings.endpoints[0];
    assert_eq!(settings.endpoints.len(), 2);
    assert_eq!(config.get_ports(), &[5433]);
    assert_eq!(config.get_password(), Some(b"it's safe".as_slice()));
    assert_eq!(config.get_application_name(), Some("worker one"));
    assert_eq!(settings.options["sslrootcert"], r"C:\CA certs\root.crt");
    assert_eq!(settings.options["sslkey"], "client key.pem");
    assert_eq!(settings.options["sslmode"], "verify-ca");
}

#[test]
fn rejects_invalid_settings_at_pool_construction() {
    for input in [
        "host=localhost sslmode=unknown",
        "postgres://localhost/db?sslrootcert",
        "postgres://localhost/db?sslrootcert=%FF",
        "postgres://localhost/db?%FF=value",
        "host localhost",
        "sslrootcert='unterminated",
        "sslrootcert=unterminated\\",
        "postgres://localhost/db?sslmode=require&sslunknown=value",
        "host=localhost sslmode=require sslunknown=value",
    ] {
        assert!(super::config(input).is_err(), "accepted {input:?}");
    }
}

fn fixture(name: &str) -> String {
    env::var(name).unwrap_or_else(|_| panic!("set {name} to run PostgreSQL fixture tests"))
}

fn encoded_path(name: &str) -> String {
    let path = PathBuf::from(fixture("TEST_TLS_DIR")).join(name);
    utf8_percent_encode(path.to_str().unwrap(), NON_ALPHANUMERIC).to_string()
}

async fn uses_tls(url: &str) -> super::SetupResult<bool> {
    let manager = AsyncDieselConnectionManager::new_with_config(url, super::config(url)?);
    let pool = Pool::builder(manager).max_size(1).build()?;
    tokio::time::timeout(Duration::from_secs(10), async {
        let mut conn = pool.get().await?;
        let ssl =
            diesel::dsl::sql::<Bool>("(SELECT ssl FROM pg_stat_ssl WHERE pid = pg_backend_pid())");
        Ok(diesel::select(ssl).get_result(&mut conn).await?)
    })
    .await?
}

const TLS: Option<bool> = Some(true);
const PLAIN: Option<bool> = Some(false);

#[tokio::test]
#[ignore = "requires configure-postgres-tls.sh and the plaintext PostgreSQL fixture"]
async fn tls_connection_modes() {
    let tls = fixture("TEST_TLS_DATABASE_URL");
    let plain = fixture("TEST_PLAIN_DATABASE_URL");
    let mismatch = tls.replacen("localhost", "127.0.0.1", 1);
    assert_ne!(mismatch, tls, "the TLS certificate is for localhost");
    let plain_only = tls.replacen("://postgres:", "://hdrop_plain_client:", 1);
    let client = tls.replacen("://postgres:", "://hdrop_tls_client:", 1);
    let ca = format!("sslrootcert={}", encoded_path("ca.crt"));
    let wrong_ca = format!("sslrootcert={}", encoded_path("wrong-ca.crt"));
    let identity = format!(
        "sslcert={}&sslkey={}",
        encoded_path("client.crt"),
        encoded_path("client.key")
    );

    for (base, query, expected) in [
        (&tls, "", TLS),
        (&tls, "sslmode=prefer", TLS),
        (&tls, "sslmode=require", TLS),
        (&tls, &format!("sslmode=prefer&{ca}"), TLS),
        (&tls, &format!("sslmode=require&{ca}"), TLS),
        (&tls, &format!("sslmode=verify-ca&{ca}"), TLS),
        (&tls, &format!("sslmode=verify-full&{ca}"), TLS),
        (&tls, "sslmode=disable", PLAIN),
        (&tls, "sslmode=allow", PLAIN),
        (&plain, "", PLAIN),
        (&plain, "sslmode=prefer", PLAIN),
        (&plain, "sslmode=require", None),
        (&tls, &format!("sslmode=require&{wrong_ca}"), None),
        (&tls, &format!("sslmode=verify-ca&{wrong_ca}"), None),
        (&tls, &format!("sslmode=verify-full&{wrong_ca}"), None),
        (&mismatch, &format!("sslmode=verify-full&{ca}"), None),
        (&mismatch, &format!("sslmode=verify-ca&{ca}"), TLS),
        (&tls, &format!("sslmode=require&{ca}.missing"), None),
        (&plain_only, "sslmode=require", None),
        (&plain_only, "sslmode=prefer", PLAIN),
        (&client, &format!("sslmode=verify-full&{ca}"), None),
        (
            &client,
            &format!("sslmode=verify-full&{ca}&{identity}"),
            TLS,
        ),
        (&client, &format!("sslmode=allow&{ca}&{identity}"), TLS),
    ] {
        let separator = if base.contains('?') { '&' } else { '?' };
        let result = uses_tls(&format!("{base}{separator}{query}")).await;
        assert_eq!(
            result.as_ref().ok().copied(),
            expected,
            "{query}: {result:?}"
        );
    }

    #[cfg(unix)]
    {
        let cert = encoded_path("client.crt");
        let key = encoded_path("key-permissions/client.key");
        let url = format!("{client}?sslmode=verify-full&{ca}&sslcert={cert}&sslkey={key}");
        let error = uses_tls(&url).await.unwrap_err();
        assert!(
            error.to_string().contains("group or world access"),
            "{error}"
        );
    }
}

#[tokio::test]
#[ignore = "requires configure-postgres-tls.sh and the plaintext PostgreSQL fixture"]
async fn authentication_environment() {
    // A child inherits just this case's PG settings, independent of concurrent Cargo tests.
    if let Ok(url) = env::var("HDROP_TEST_AUTH_URL") {
        let result = uses_tls(&url).await;
        let actual = match &result {
            Ok(true) => "tls",
            Ok(false) => "plain",
            Err(_) => "error",
        };
        assert_eq!(actual, fixture("HDROP_TEST_AUTH_EXPECTED"), "{result:?}");
        return;
    }
    let tls = fixture("TEST_TLS_DATABASE_URL");
    let plain = fixture("TEST_PLAIN_DATABASE_URL");
    let dir = PathBuf::from(fixture("TEST_TLS_DIR"));
    let check = |url: &str, expected: &str, settings: &[(&str, &str)]| {
        let mut child = Command::new(env::current_exe().unwrap());
        for (name, _) in env::vars_os() {
            if name
                .to_string_lossy()
                .to_ascii_uppercase()
                .starts_with("PG")
            {
                child.env_remove(name);
            }
        }
        let output = child
            .args([
                "--ignored",
                "--exact",
                "connection::tests::authentication_environment",
            ])
            .env("HDROP_TEST_AUTH_URL", url)
            .env("HDROP_TEST_AUTH_EXPECTED", expected)
            .env("HOME", &dir)
            .env("APPDATA", &dir)
            .envs(settings.iter().copied())
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    };

    let config: tokio_postgres::Config = tls.parse().unwrap();
    let password = std::str::from_utf8(config.get_password().unwrap()).unwrap();
    let (_, address) = tls.split_once('@').unwrap();
    let passwordless = format!("postgres://postgres@{address}");
    let passfile = dir.join("pgpass");
    check(&passwordless, "error", &[]);
    check(&passwordless, "tls", &[("PGPASSWORD", password)]);
    check(&tls, "tls", &[("PGPASSWORD", "wrong-password")]);
    check(
        &passwordless,
        "tls",
        &[("PGPASSFILE", passfile.to_str().unwrap())],
    );
    check(
        &format!("{passwordless}?passfile={}", encoded_path("pgpass")),
        "tls",
        &[("PGPASSFILE", "missing")],
    );

    // The default passfile uses a different password for the first host. Only a TLS refusal should advance to the next host.
    let (_, plain_address) = plain.split_once('@').unwrap();
    let (plain_endpoint, _) = plain_address.split_once('/').unwrap();
    let first = plain_endpoint.replace("localhost", "127.0.0.1");
    let multi = format!("postgres://postgres@{first},{address}");
    let home = dir.join("home");
    let home = home.to_str().unwrap();
    let default_home = [("HOME", home), ("APPDATA", home)];
    check(&format!("{multi}?sslmode=require"), "tls", &default_home);
    check(&format!("{multi}?sslmode=disable"), "error", &default_home);

    check(&tls, "tls", &[("PGCHANNELBINDING", "require")]);
    check(&plain, "error", &[("PGCHANNELBINDING", "require")]);
    check(
        &format!("{plain}?channel_binding=disable"),
        "plain",
        &[("PGCHANNELBINDING", "require")],
    );
    check(&plain, "error", &[("PGGSSENCMODE", "require")]);
    check(
        &format!("{plain}?gssencmode=disable"),
        "plain",
        &[("PGGSSENCMODE", "require")],
    );
    check(&plain, "error", &[("PGREQUIREAUTH", "scram-sha-256")]);
    check(&plain, "error", &[("PGSERVICE", "missing-service")]);
    check(&plain, "error", &[("PGTARGETSESSIONATTRS", "read-only")]);
    check(
        &format!("{plain}?target_session_attrs=any"),
        "plain",
        &[("PGTARGETSESSIONATTRS", "read-only")],
    );
}
