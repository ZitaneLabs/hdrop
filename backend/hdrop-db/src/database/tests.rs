use std::time::Duration;

use diesel::result::{DatabaseErrorKind::UniqueViolation, Error as DieselError};
use diesel_async::SimpleAsyncConnection;
use metrics_exporter_prometheus::PrometheusBuilder;
use tokio::time::timeout;

use super::*;
use crate::error::Error;

async fn execute(db: &Database, sql: &str) -> Result<()> {
    Ok(db.pool.get().await?.batch_execute(sql).await?)
}

// Each test creates and drops its own schema, including when tests run concurrently.
async fn test_database(pool_size: usize) -> Result<(Database, String)> {
    let url =
        std::env::var("TEST_DATABASE_URL").expect("set TEST_DATABASE_URL to run PostgreSQL tests");
    let schema = format!("hdrop_test_{}", Uuid::new_v4().simple());
    let separator = if url.contains('?') { '&' } else { '?' };
    let url = format!("{url}{separator}options=-csearch_path%3D{schema}");
    let pool = Pool::builder(AsyncDieselConnectionManager::new(url))
        .max_size(pool_size)
        .build()?;
    let db = Database { pool };
    let migrations = concat!(
        include_str!("../../migrations/2023-03-01-223630_create_File/up.sql"),
        "\n",
        include_str!("../../migrations/2023-06-22-150526_api_v1_0/up.sql"),
    );
    execute(&db, &format!("CREATE SCHEMA {schema};\n{migrations}")).await?;
    Ok((db, format!("DROP SCHEMA {schema} CASCADE")))
}

fn new_file(token: &str) -> File {
    File {
        uuid: Uuid::new_v4(),
        accessToken: token.into(),
        challengeHash: "hash".into(),
        challengeData: "challenge".into(),
        fileNameData: "name".into(),
        salt: "salt".into(),
        iv: "iv".into(),
        createdAt: Timestamp::now().into(),
        expiresAt: (Timestamp::now() + jiff::SignedDuration::from_hours(1)).into(),
        updateToken: "update".into(),
        dataUrl: None,
    }
}

async fn insert_raw(db: &Database, file: &File) -> Result<usize> {
    Ok(diesel::insert_into(files_table::files)
        .values(file)
        .execute(&mut db.pool.get().await?)
        .await?)
}

fn violated_index(error: &Error) -> &str {
    match error {
        Error::Diesel(DieselError::DatabaseError(UniqueViolation, info)) => {
            info.constraint_name().unwrap()
        }
        _ => panic!("expected a unique-index violation, got {error}"),
    }
}

#[tokio::test]
#[ignore = "requires PostgreSQL and TEST_DATABASE_URL"]
async fn update_expire_and_delete_file() -> Result<()> {
    let (db, cleanup) = test_database(1).await?;
    let missing = db.get_file_by_access_token("new").await.unwrap_err();
    assert!(missing.is_not_found());
    let uuid = db.insert_file(new_file("new")).await?.uuid;
    assert_eq!(db.get_file_rows().await?, 1);
    let unexpired = db.insert_file(new_file("still-live")).await?;
    let url = "https://example.com/file";
    db.update_data_url(uuid, Some(url)).await?;
    let file = db.get_file_by_access_token("new").await?;
    assert_eq!(file.uuid, uuid);
    assert_eq!(file.dataUrl.as_deref(), Some(url));
    db.update_data_url(uuid, None).await?;
    assert!(db.get_files_to_flush().await?.is_empty());
    let expires_at = Timestamp::now() - jiff::SignedDuration::from_hours(1);
    db.update_file_expiry(uuid, expires_at).await?;
    let file = db.get_file_by_uuid(uuid).await?;
    assert!(file.dataUrl.is_none());
    assert_eq!(
        file.expiresAt.to_jiff().as_microsecond(),
        expires_at.as_microsecond()
    );
    assert_eq!(file.challengeHash, "hash");
    assert_eq!(file.fileNameData, "name");
    assert_eq!(file.challengeData, "challenge");
    assert_eq!(file.salt, "salt");
    assert_eq!(file.iv, "iv");
    assert_eq!(db.get_files_to_flush().await?, vec![uuid]);
    db.delete_file(uuid).await?;
    assert!(db.get_file_by_uuid(uuid).await.unwrap_err().is_not_found());
    db.delete_file(unexpired.uuid).await?;
    assert_eq!(db.get_file_rows().await?, 0);
    execute(&db, &cleanup).await
}

#[tokio::test]
#[ignore = "requires PostgreSQL and TEST_DATABASE_URL"]
async fn token_collisions_grow_then_terminate_at_the_limit() -> Result<()> {
    let (db, cleanup) = test_database(1).await?;
    db.insert_file(new_file("taken")).await?;
    let error = insert_raw(&db, &new_file("taken")).await.unwrap_err();
    assert_eq!(violated_index(&error), "files_accessToken_key");
    // Give lengths 5, 6 and the maximum the same index key as the existing row.
    let force_collisions = format!(
        r#"DROP INDEX "files_accessToken_key";
        CREATE UNIQUE INDEX "files_accessToken_key" ON files ((1))
        WHERE length("accessToken") < 7 OR length("accessToken") >= {MAX_TOKEN_LENGTH}"#
    );
    execute(&db, &force_collisions).await?;
    let file = new_file("taken");
    let uuid = file.uuid;
    let budget = Duration::from_secs(5);
    let grown = timeout(budget, db.insert_file(file))
        .await
        .expect("collision retries stalled")?;
    assert_eq!(grown.uuid, uuid);
    assert_eq!(grown.accessToken.len(), 7);
    assert_eq!(
        db.get_file_by_access_token(&grown.accessToken).await?.uuid,
        uuid
    );
    assert_eq!(db.get_file_rows().await?, 2);

    let at_limit = new_file(&"a".repeat(MAX_TOKEN_LENGTH));
    let error = timeout(budget, db.insert_file(at_limit))
        .await
        .expect("retry limit exceeded")
        .unwrap_err();
    assert_eq!(violated_index(&error), "files_accessToken_key");
    assert_eq!(db.get_file_rows().await?, 2);
    execute(&db, &cleanup).await
}

#[tokio::test]
#[ignore = "requires PostgreSQL and TEST_DATABASE_URL"]
async fn concurrent_inserts_retry_and_preserve_token_length() -> Result<()> {
    let (db, cleanup) = test_database(2).await?;
    for length in [5, 8, 32, 64] {
        let token = "a".repeat(length);
        let (left, right) = tokio::try_join!(
            db.insert_file(new_file(&token)),
            db.insert_file(new_file(&token))
        )?;
        assert_ne!(left.accessToken, right.accessToken);
        assert!(left.accessToken == token || right.accessToken == token);
        assert_eq!(db.get_file_rows().await?, 2);
        for file in [left, right] {
            assert_eq!(file.accessToken.len(), length);
            assert_eq!(
                db.get_file_by_access_token(&file.accessToken).await?.uuid,
                file.uuid
            );
            db.delete_file(file.uuid).await?;
        }
    }
    execute(&db, &cleanup).await
}

#[tokio::test]
#[ignore = "requires PostgreSQL and TEST_DATABASE_URL"]
async fn file_count_metrics_track_mutations_and_reconcile() -> Result<()> {
    let recorder = PrometheusBuilder::new().build_recorder();
    let _guard = metrics::set_default_local_recorder(&recorder);
    let handle = recorder.handle();
    let assert_count = |expected: u64| {
        let name = names::storage::DATABASE_FILE_COUNT;
        let metric = format!("# TYPE {name} gauge\n{name} {expected}");
        assert_eq!(handle.render().trim(), metric);
    };
    let (db, cleanup) = test_database(1).await?;
    let external = new_file("external");
    insert_raw(&db, &external).await?;
    db.update_metrics().await;
    assert_count(1);

    let inserted = db.insert_file(new_file("inserted")).await?;
    assert_count(2);
    let duplicate = File {
        uuid: inserted.uuid,
        ..new_file("duplicate")
    };
    let error = db.insert_file(duplicate).await.unwrap_err();
    assert!(matches!(
        violated_index(&error),
        "files_pkey" | "files_uuid_key"
    ));
    assert_eq!(db.get_file_rows().await?, 2);
    assert_count(2);
    let retried = db.insert_file(new_file("inserted")).await?;
    assert_count(3);
    db.delete_file(retried.uuid).await?;
    assert_count(2);
    db.delete_file(retried.uuid).await?;
    assert_count(2);

    diesel::delete(files_table::files.filter(files_table::uuid.eq(external.uuid)))
        .execute(&mut db.pool.get().await?)
        .await?;
    db.delete_file(inserted.uuid).await?;
    assert_count(1);
    db.update_metrics().await;
    assert_count(0);

    let retained = db.insert_file(new_file("retained")).await?;
    assert_count(1);
    execute(&db, &cleanup).await?;
    let lookup = db.get_file_by_access_token("missing").await.unwrap_err();
    let insert = db.insert_file(new_file("failed")).await.unwrap_err();
    let delete = db.delete_file(retained.uuid).await.unwrap_err();
    for error in [lookup, insert, delete] {
        assert!(matches!(error, Error::Diesel(_)), "{error}");
    }
    db.update_metrics().await;
    assert_count(1);
    db.pool.close();
    assert_eq!(Database::generate_access_token().len(), 5);
    assert!(matches!(
        db.get_file_by_access_token("missing").await,
        Err(Error::DeadpoolPool(_))
    ));
    Ok(())
}
