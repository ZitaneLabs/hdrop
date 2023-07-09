use super::{
    app_state::AppState,
    multipart::{PartialUploadedFile, UploadedFile},
};
use axum::{
    extract::{Multipart, Path, Query, State},
    headers::{authorization::Bearer, Authorization},
    http::StatusCode,
    response::IntoResponse,
    Json, TypedHeader,
};
use chrono::Utc;
use hdrop_db::{Database, File, InsertFile};
use hdrop_shared::{
    requests as request,
    responses::{FileMetaData, GetChallengeData, UploadFileData, VerifyChallengeData},
};
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    background_workers::{
        expiration_worker::ExpirationWorker, storage_synchronizer::ProviderSyncEntry,
    },
    core::Fetchtype,
    error::Error,
    Result,
};

#[derive(Debug, serde::Deserialize)]
pub struct UpdateTokenQuery {
    update_token: String,
}

/* Routes */
pub async fn upload_file(
    State(state): State<Arc<AppState>>,
    multipart_formdata: Multipart,
) -> Result<Json<UploadFileData>> {
    // Parse multipart formdata
    let data: UploadedFile = PartialUploadedFile::from_multipart(multipart_formdata)
        .await
        .try_into()?;

    // Upload to StorageProvider & update DB (S3 etc.)
    let uuid = Uuid::new_v4();

    let access_token = state.database.generate_access_token().await?;
    let update_token = Database::generate_update_token();
    let time = Utc::now();
    let file = InsertFile {
        uuid,
        accessToken: access_token.clone(),
        updateToken: update_token.clone(),
        dataUrl: None,
        fileNameData: data.file_name_data,
        challengeData: data.challenge_data,
        challengeHash: data.challenge_hash,
        salt: data.salt,
        iv: data.iv,
        createdAt: time,
        expiresAt: time + chrono::Duration::seconds(86400),
    };

    // Inser Partial File into DB
    let _ = state.database.insert_file(file).await?;

    // Cache to ensure instant availability after upload
    state
        .cache
        .write()
        .await
        .put(uuid, data.file_data.to_vec())
        .await?;

    // S3
    let provider_sync_entry = ProviderSyncEntry {
        provider: state.provider.clone(),
        database: state.database.clone(),
        uuid,
        file_data: data.file_data,
        cache: state.cache.clone(),
    };

    // Send file for upload, db update & cache clearance to storage synchronization thread
    state
        .get_provider_sync_tx()
        .send(provider_sync_entry)
        .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))
        .expect("Unable to send data to the storage synchronizer");

    Ok(Json(UploadFileData {
        access_token,
        update_token,
    }))
}

pub async fn get_file(
    State(state): State<Arc<AppState>>,
    TypedHeader(bearer): TypedHeader<Authorization<Bearer>>,
    Path(access_token): Path<String>,
) -> Result<impl IntoResponse> {
    // Check bearer token to restrict access
    let challenge_hash = state
        .database
        .get_verification_data(&access_token)
        .await?
        .challenge_hash
        .unwrap_or("".to_string());

    if bearer.token() != challenge_hash {
        return Err(Error::InvalidChallenge);
    }

    let file_entry = state
        .database
        .get_file_by_access_token(&access_token)
        .await?;

    match file_entry.dataUrl {
        Some(_) => Ok(Json(FileMetaData {
            file_url: file_entry.dataUrl,
        })
        .into_response()),
        None => Ok(get_raw_file_bytes(State(state), file_entry)
            .await?
            .into_response()),
    }
}

pub async fn get_raw_file_bytes(
    State(state): State<Arc<AppState>>,
    file_entry: File,
) -> Result<impl IntoResponse> {
    // Check for file in cache
    let cache = state.cache.read().await;
    if let Ok(data) = cache.get(file_entry.uuid).await {
        return Ok(data.into_owned().into_response());
    }

    // Check for file in provider
    let provider = state.provider.read().await;
    let fetched_data = provider.get_file(file_entry.uuid.to_string()).await?;
    match fetched_data {
        Fetchtype::FileData(data) => Ok(data.to_vec().into_response()),
        Fetchtype::FileUrl(_) => Err(Error::InvalidFile),
    }
}

pub async fn delete_file(
    State(state): State<Arc<AppState>>,
    Path(access_token): Path<String>,
    Query(query): Query<UpdateTokenQuery>,
) -> Result<Json<()>> {
    let file = state
        .database
        .get_file_by_access_token(&access_token)
        .await?;

    if file.updateToken == query.update_token {
        // Delete file
        let deletion_worker = ExpirationWorker::new(
            state.provider.clone(),
            state.database.clone(),
            state.cache.clone(),
        );

        deletion_worker.delete_file(file.uuid).await?;
        Ok(Json(()))
    } else {
        Err(Error::UpdateToken)
    }
}

pub async fn update_file_expiry(
    State(state): State<Arc<AppState>>,
    Path(access_token): Path<String>,
    Query(query): Query<UpdateTokenQuery>,
    Json(expiry_data): Json<request::ExpiryData>,
) -> Result<Json<()>> {
    let mut file = state
        .database
        .get_file_by_access_token(access_token)
        .await?;

    if expiry_data.expiry > 86400 {
        return Err(Error::InvalidExpiry);
    }

    if file.updateToken == query.update_token {
        file.expiresAt = file.createdAt + chrono::Duration::seconds(expiry_data.expiry);
        state.database.update_file_expiry(file).await?;

        Ok(Json(()))
    } else {
        Err(Error::UpdateToken)
    }
}

pub async fn get_challenge(
    State(state): State<Arc<AppState>>,
    Path(access_token): Path<String>,
) -> Result<Json<GetChallengeData>> {
    let get_challenge_data = state.database.get_challenge(access_token).await?;

    Ok(Json(get_challenge_data))
}

pub async fn verify_challenge(
    State(state): State<Arc<AppState>>,
    Path(access_token): Path<String>,
    Json(json_data): Json<request::ChallengeData>,
) -> Result<Json<VerifyChallengeData>> {
    let mut verify_challenge_data = state.database.get_verification_data(access_token).await?;

    if verify_challenge_data.challenge_hash == Some(json_data.challenge) {
        verify_challenge_data.challenge_hash = None;
        let response = Json(verify_challenge_data);

        Ok(response)
    } else {
        Err(Error::InvalidChallenge)
    }
}
