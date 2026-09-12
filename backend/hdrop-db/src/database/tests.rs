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
        generator: TokenGenerator::default(),
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

fn new_file(token: &str) -> InsertFile {
    InsertFile {
        uuid: Uuid::new_v4(),
        accessToken: token.into(),
        createdAt: Utc::now(),
        expiresAt: Utc::now() + chrono::Duration::hours(1),
        ..Default::default()
    }
}

#[tokio::test]
#[ignore = "requires PostgreSQL and TEST_DATABASE_URL"]
async fn async_queries_and_token_uniqueness() {
    let (db, mut admin, schema) = test_database(1).await;
    assert!(!db.check_access_token_collission("taken").await.unwrap());
    let file = db.insert_file(new_file("taken")).await.unwrap();
    let uuid = file.uuid;
    assert!(db.check_access_token_collission("taken").await.unwrap());
    assert_eq!(
        db.get_file_by_access_token("taken").await.unwrap().uuid,
        uuid
    );
    assert_eq!(db.get_file_rows().await.unwrap(), 1);

    // Bypassing the preflight and retry still cannot store duplicate tokens.
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
        db.get_file_metadata("taken")
            .await
            .unwrap()
            .file_url
            .as_deref(),
        Some("https://example.com/file")
    );
    db.update_data_url(uuid, None::<String>).await.unwrap();
    assert!(db
        .get_file_metadata("taken")
        .await
        .unwrap()
        .file_url
        .is_none());
    let mut file = db.get_file_by_uuid(uuid).await.unwrap();
    file.challengeHash = "hash".into();
    file.challengeData = "challenge".into();
    file.fileNameData = "name".into();
    file.salt = "salt".into();
    file.iv = "iv".into();
    db.update_file(file).await.unwrap();
    let verification = db.get_verification_data("taken").await.unwrap();
    assert_eq!(verification.challenge_hash.as_deref(), Some("hash"));
    assert_eq!(verification.file_name_data, "name");
    let challenge = db.get_challenge("taken").await.unwrap();
    assert_eq!(
        (challenge.challenge, challenge.salt, challenge.iv),
        ("challenge".into(), "salt".into(), "iv".into())
    );
    assert!(db.get_files_to_flush().await.unwrap().is_empty());
    let mut file = db.get_file_by_uuid(uuid).await.unwrap();
    file.expiresAt = Utc::now() - chrono::Duration::hours(1);
    db.update_file_expiry(file).await.unwrap();
    assert_eq!(db.get_files_to_flush().await.unwrap(), vec![uuid]);
    db.delete_file_by_uuid(uuid).await.unwrap();
    db.delete_file_by_uuid(uuid).await.unwrap();
    assert!(db.get_file_by_uuid(uuid).await.unwrap_err().is_not_found());
    assert_eq!(db.delete_file(retried).await.unwrap().accessToken.len(), 5);
    assert_eq!(db.get_file_rows().await.unwrap(), 0);

    // A real query failure must not masquerade as an unused token.
    admin
        .batch_execute(&format!("DROP SCHEMA {schema} CASCADE"))
        .await
        .unwrap();
    assert!(matches!(
        db.check_access_token_collission("missing").await,
        Err(Error::Diesel(_))
    ));
    assert!(matches!(
        db.generate_access_token().await,
        Err(Error::Diesel(_))
    ));
    db.pool.close();
    assert!(matches!(
        db.check_access_token_collission("missing").await,
        Err(Error::DeadpoolPool(_))
    ));
}

#[tokio::test]
#[ignore = "requires PostgreSQL and TEST_DATABASE_URL"]
async fn concurrent_inserts_retry_token_conflicts() {
    let (db, mut admin, schema) = test_database(2).await;
    // Both clients have passed the preflight before attempting the same token.
    assert!(!db.check_access_token_collission("shared").await.unwrap());
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
    db.delete_file_by_uuid(retried.uuid).await.unwrap();
    assert_count(2.0);
    db.delete_file_by_uuid(retried.uuid).await.unwrap();
    assert_count(2.0);
    assert!(db.delete_file(retried).await.unwrap_err().is_not_found());
    assert_count(2.0);

    // External changes are picked up by reconciliation, not by each mutation.
    diesel::delete(files_table::files.filter(files_table::uuid.eq(external.uuid)))
        .execute(&mut db.pool.get().await.unwrap())
        .await
        .unwrap();
    db.delete_file(inserted).await.unwrap();
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
    assert!(db.delete_file_by_uuid(file.uuid).await.is_err());
    assert!(db.delete_file(file).await.is_err());
    db.update_metrics().await;
    assert_count(1.0);
}
