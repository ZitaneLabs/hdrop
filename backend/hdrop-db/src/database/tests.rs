use diesel::result::{DatabaseErrorKind, Error as DieselError};
use diesel_async::{AsyncConnection, SimpleAsyncConnection};

use super::*;
use crate::error::Error;

// Each test uses its own schema; never migrate or clear the caller's tables.
async fn test_database(pool_size: usize) -> (Database, AsyncPgConnection, String) {
    let url =
        std::env::var("TEST_DATABASE_URL").expect("set TEST_DATABASE_URL to run PostgreSQL tests");
    let mut admin = AsyncPgConnection::establish(&url).await.unwrap();
    let schema = format!("hdrop_test_{}", Uuid::new_v4().simple());
    admin
        .batch_execute(&format!("CREATE SCHEMA {schema}"))
        .await
        .unwrap();
    let separator = if url.contains('?') { '&' } else { '?' };
    let manager = AsyncDieselConnectionManager::new(format!(
        "{url}{separator}options=-csearch_path%3D{schema}"
    ));
    let database = Database {
        pool: Pool::builder(manager).max_size(pool_size).build().unwrap(),
    };
    let mut conn = database.pool.get().await.unwrap();
    for migration in [
        include_str!("../../migrations/2023-03-01-223630_create_File/up.sql"),
        include_str!("../../migrations/2023-06-22-150526_api_v1_0/up.sql"),
    ] {
        conn.batch_execute(migration).await.unwrap();
    }
    drop(conn);
    (database, admin, schema)
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
        createdAt: Utc::now(),
        expiresAt: Utc::now() + chrono::Duration::hours(1),
        updateToken: "update".into(),
        dataUrl: None,
    }
}

#[tokio::test]
#[ignore = "requires PostgreSQL and TEST_DATABASE_URL"]
async fn async_queries_and_token_uniqueness() {
    let (db, mut admin, schema) = test_database(1).await;
    assert!(db
        .get_file_by_access_token("taken")
        .await
        .unwrap_err()
        .is_not_found());
    let file = db.insert_file(new_file("taken")).await.unwrap();
    let uuid = file.uuid;
    assert_eq!(
        db.get_file_by_access_token("taken").await.unwrap().uuid,
        uuid
    );
    assert_eq!(db.get_file_rows().await.unwrap(), 1);

    // Bypassing the insert retry still cannot store duplicate tokens.
    let error = diesel::insert_into(files_table::files)
        .values(new_file("taken"))
        .execute(&mut db.pool.get().await.unwrap())
        .await
        .unwrap_err();
    assert!(
        matches!(error, DieselError::DatabaseError(DatabaseErrorKind::UniqueViolation, ref info)
        if info.constraint_name() == Some("files_accessToken_key"))
    );
    let retried = db.insert_file(new_file("taken")).await.unwrap();
    assert_ne!(retried.accessToken, "taken");
    assert_eq!(
        db.get_file_by_access_token(&retried.accessToken)
            .await
            .unwrap()
            .uuid,
        retried.uuid
    );

    let mut duplicate_uuid = new_file("different");
    duplicate_uuid.uuid = uuid;
    assert!(matches!(
        db.insert_file(duplicate_uuid).await,
        Err(Error::Diesel(DieselError::DatabaseError(
            DatabaseErrorKind::UniqueViolation,
            _
        )))
    ));
    assert_eq!(db.get_file_rows().await.unwrap(), 2);

    // Force every retry to collide and verify the retry budget returns an error.
    db.pool
        .get()
        .await
        .unwrap()
        .batch_execute(
            "CREATE FUNCTION force_collision() RETURNS trigger LANGUAGE plpgsql AS $$
         BEGIN NEW.\"accessToken\" := 'taken'; RETURN NEW; END $$;
         CREATE TRIGGER force_collision BEFORE INSERT ON files
         FOR EACH ROW EXECUTE FUNCTION force_collision();",
        )
        .await
        .unwrap();
    let error = tokio::time::timeout(
        std::time::Duration::from_secs(5),
        db.insert_file(new_file("forced")),
    )
    .await
    .unwrap()
    .unwrap_err();
    assert!(
        matches!(error, Error::Diesel(DieselError::DatabaseError(DatabaseErrorKind::UniqueViolation, ref info))
        if info.constraint_name() == Some("files_accessToken_key"))
    );
    db.pool
        .get()
        .await
        .unwrap()
        .batch_execute("DROP TRIGGER force_collision ON files")
        .await
        .unwrap();

    db.update_data_url(uuid, Some("https://example.com/file"))
        .await
        .unwrap();
    assert_eq!(
        db.get_file_by_access_token("taken")
            .await
            .unwrap()
            .dataUrl
            .as_deref(),
        Some("https://example.com/file")
    );
    db.update_data_url(uuid, None).await.unwrap();
    assert!(db
        .get_file_by_access_token("taken")
        .await
        .unwrap()
        .dataUrl
        .is_none());
    let file = db.get_file_by_access_token("taken").await.unwrap();
    assert_eq!(file.challengeHash, "hash");
    assert_eq!(file.fileNameData, "name");
    assert_eq!(
        (file.challengeData, file.salt, file.iv),
        ("challenge".into(), "salt".into(), "iv".into())
    );
    assert!(db.get_files_to_flush().await.unwrap().is_empty());
    let expires_at = Utc::now() - chrono::Duration::hours(1);
    db.update_file_expiry(uuid, expires_at).await.unwrap();
    let file = db.get_file_by_uuid(uuid).await.unwrap();
    assert_eq!(file.challengeHash, "hash");
    assert_eq!(
        file.expiresAt.timestamp_micros(),
        expires_at.timestamp_micros()
    );
    assert_eq!(db.get_files_to_flush().await.unwrap(), vec![uuid]);
    db.delete_file(uuid).await.unwrap();
    db.delete_file(uuid).await.unwrap();
    assert!(db.get_file_by_uuid(uuid).await.unwrap_err().is_not_found());
    db.delete_file(retried.uuid).await.unwrap();
    assert_eq!(db.get_file_rows().await.unwrap(), 0);

    // Retry generation preserves longer tokens too, not just the default length.
    for length in [8, 32, 64] {
        let token = "a".repeat(length);
        let original = db.insert_file(new_file(&token)).await.unwrap();
        let retried = db.insert_file(new_file(&token)).await.unwrap();
        assert_eq!(retried.accessToken.len(), length);
        assert_ne!(retried.accessToken, token);
        db.delete_file(original.uuid).await.unwrap();
        db.delete_file(retried.uuid).await.unwrap();
    }

    // Query and connection failures still propagate through the production API.
    admin
        .batch_execute(&format!("DROP SCHEMA {schema} CASCADE"))
        .await
        .unwrap();
    assert!(matches!(
        db.get_file_by_access_token("missing").await,
        Err(Error::Diesel(_))
    ));
    assert!(matches!(
        db.insert_file(new_file("failed")).await,
        Err(Error::Diesel(_))
    ));
    db.pool.close();
    assert_eq!(Database::generate_access_token().len(), 5);
    assert!(matches!(
        db.get_file_by_access_token("missing").await,
        Err(Error::DeadpoolPool(_))
    ));
}

#[tokio::test]
#[ignore = "requires PostgreSQL and TEST_DATABASE_URL"]
async fn concurrent_inserts_retry_token_conflicts() {
    let (db, mut admin, schema) = test_database(2).await;
    // Both clients attempt the same token without a preflight query.
    let (left, right) = tokio::join!(
        db.insert_file(new_file("shared")),
        db.insert_file(new_file("shared"))
    );
    let (left, right) = (left.unwrap(), right.unwrap());
    assert_ne!(left.accessToken, right.accessToken);
    assert!(left.accessToken == "shared" || right.accessToken == "shared");
    assert_eq!(db.get_file_rows().await.unwrap(), 2);
    admin
        .batch_execute(&format!("DROP SCHEMA {schema} CASCADE"))
        .await
        .unwrap();
}

#[tokio::test]
#[ignore = "requires PostgreSQL and TEST_DATABASE_URL"]
async fn file_count_metrics_track_mutations_and_reconcile() {
    use metrics_util::debugging::{DebugValue, DebuggingRecorder};

    let recorder = DebuggingRecorder::new();
    // This test's current-thread runtime keeps the recorder isolated from other tests.
    let _guard = metrics::set_default_local_recorder(&recorder);
    let snapshotter = recorder.snapshotter();
    let assert_count = |expected: f64| {
        let values = snapshotter.snapshot().into_vec();
        assert_eq!(values.len(), 1);
        assert_eq!(values[0].3, DebugValue::Gauge(expected.into()));
        // DebuggingRecorder snapshots reset gauges; restore the checked value.
        metrics::gauge!(names::storage::DATABASE_FILE_COUNT).set(expected);
    };
    let (db, mut admin, schema) = test_database(1).await;
    let external = new_file("external");
    diesel::insert_into(files_table::files)
        .values(&external)
        .execute(&mut db.pool.get().await.unwrap())
        .await
        .unwrap();
    db.update_metrics().await;
    assert_count(1.0);

    let inserted = db.insert_file(new_file("inserted")).await.unwrap();
    assert_count(2.0);
    let mut duplicate_uuid = new_file("duplicate");
    duplicate_uuid.uuid = inserted.uuid;
    assert!(db.insert_file(duplicate_uuid).await.is_err());
    assert_count(2.0);
    let retried = db.insert_file(new_file("inserted")).await.unwrap();
    assert_count(3.0);
    db.delete_file(retried.uuid).await.unwrap();
    assert_count(2.0);
    db.delete_file(retried.uuid).await.unwrap();
    assert_count(2.0);

    // External changes are picked up by reconciliation, not by each mutation.
    diesel::delete(files_table::files.filter(files_table::uuid.eq(external.uuid)))
        .execute(&mut db.pool.get().await.unwrap())
        .await
        .unwrap();
    db.delete_file(inserted.uuid).await.unwrap();
    assert_count(1.0);
    db.update_metrics().await;
    assert_count(0.0);

    let file = db.insert_file(new_file("retained")).await.unwrap();
    assert_count(1.0);
    admin
        .batch_execute(&format!("DROP SCHEMA {schema} CASCADE"))
        .await
        .unwrap();
    assert!(db.insert_file(new_file("failed")).await.is_err());
    assert!(db.delete_file(file.uuid).await.is_err());
    db.update_metrics().await;
    assert_count(1.0);
}
