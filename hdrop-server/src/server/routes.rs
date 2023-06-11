use super::{
    multipart::{PartialUploadedFile, UploadedFile},
    AppState,
};
use crate::background_workers::background_workers::ProviderSyncEntry;
use crate::core::Fetchtype;
use axum::{
    body::{Bytes, StreamBody},
    extract::{Multipart, Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use chrono::Utc;
use hdrop_db::{Database, InsertFile};
use hdrop_shared::requests::{ChallengeData, ExpiryData};
use hdrop_shared::{responses, ErrorData, Response, ResponseData};
use std::{borrow::Cow, sync::Arc};
use uuid::Uuid;

#[derive(Debug, serde::Deserialize)]
pub struct UpdateTokenQuery {
    #[serde(rename = "updateToken")]
    update_token: String,
}

/* Routes */
// #[axum::debug_handler]
pub async fn upload_file(
    State(state): State<Arc<AppState>>,
    multipart_formdata: Multipart,
) -> Json<Response<responses::UploadFileData>> {
    // Multipart Upload to Server
    let data: UploadedFile = match PartialUploadedFile::from_multipart(multipart_formdata)
        .await
        .try_into()
    {
        Ok(x) => x,
        Err(reason) => {
            return Json(Response::new(ResponseData::Error(ErrorData {
                reason: format!("{reason}"),
            })))
        }
    };

    // Upload to StorageProvider & update DB (S3 etc.)
    let uuid = Uuid::new_v4();

    let access_token = state
        .database
        .generate_access_token()
        .await
        .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))
        .unwrap();
    let update_token = Database::generate_update_token();
    let time = Utc::now();
    let file = InsertFile {
        id: None,
        uuid,
        accessToken: access_token.clone(),
        updateToken: update_token.clone(),
        dataUrl: None,
        fileNameData: data.file_name_data,
        fileNameHash: data.file_name_hash,
        salt: data.salt,
        iv: data.iv,
        createdAt: time,
        expiresAt: time + chrono::Duration::seconds(86400),
    };

    // Inser Partial File into DB
    let _ = state
        .database
        .insert_file(file)
        .await
        .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))
        .unwrap();

    // Cache to ensure instant availability after upload
    state
        .cache
        .write()
        .await
        .put(uuid, data.file_data.to_vec())
        .await
        .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))
        .unwrap();

    // S3
    let provider_sync_entry = ProviderSyncEntry {
        provider: state.provider.clone(),
        database: state.database.clone(),
        uuid: uuid,
        file_data: data.file_data,
        cache: state.cache.clone(),
    };
    // Send file for upload, db update & cache clearance to S3 Synchronization thread
    state
        .tx
        .send(provider_sync_entry)
        .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))
        .unwrap();

    // Test, remove later
    let _response_data = state
        .provider
        .read()
        .await
        .get_file(uuid.to_string())
        .await
        .expect("S3 Download FAIL");

    Json(Response::new(ResponseData::Success(
        responses::UploadFileData {
            access_token,
            update_token,
        },
    )))
}

pub async fn get_file(
    // rename: get_file_metadata or something else
    State(state): State<Arc<AppState>>,
    Path(access_token): Path<String>,
) -> Json<Response<responses::FileMetaData>> {
    let Ok(file_metadata) = state.database.get_file_metadata(access_token).await else {
        return Json(Response::new(ResponseData::Error(ErrorData { reason: "No file found for given access token".to_string() })))
    };
    Json(Response::new(ResponseData::Success(file_metadata)))
}

pub async fn delete_file(
    State(state): State<Arc<AppState>>,
    Path(access_token): Path<String>,
    Query(query): Query<UpdateTokenQuery>,
) -> Json<Response<()>> {
    let file = state
        .database
        .get_file_by_access_token(&access_token)
        .await
        .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))
        .unwrap();

    if file.updateToken == query.update_token {
        // Delete file
        if let Ok(()) = state
            .provider
            .read()
            .await
            .delete_file(file.uuid.to_string())
            .await
        {
            state
                .database
                .delete_file_by_uuid(file.uuid)
                .await
                .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))
                .unwrap();
            Json(Response::new(ResponseData::Success(())))
        } else {
            Json(Response::new(ResponseData::Error(ErrorData {
                reason: "File deletion Failed".to_string(),
            })))
        }
    } else {
        Json(Response::new(ResponseData::Error(ErrorData {
            reason: "Wrong update token".to_string(),
        })))
    }
}

pub async fn update_file_expiry(
    State(state): State<Arc<AppState>>,
    Path(access_token): Path<String>,
    Query(query): Query<UpdateTokenQuery>,
    Json(expiry_data): Json<ExpiryData>,
) -> Json<Response<()>> {
    let mut file = state
        .database
        .get_file_by_access_token(access_token)
        .await
        .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))
        .unwrap();

    if expiry_data.expiry > 86400 {
        return Json(Response::new(ResponseData::Error(ErrorData {
            reason: "Expiry time above max allowed expiry time".to_string(),
        })));
    }

    if file.updateToken == query.update_token {
        file.expiresAt = file.createdAt + chrono::Duration::seconds(expiry_data.expiry);
        state
            .database
            .update_file_expiry(file)
            .await
            .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))
            .unwrap();
        Json(Response::new(ResponseData::Success(())))
    } else {
        Json(Response::new(ResponseData::Error(ErrorData {
            reason: "Wrong update token".to_string(),
        })))
    }
}

pub async fn get_raw_file_bytes(
    State(state): State<Arc<AppState>>,
    Path(access_token): Path<String>,
) -> Result<axum::response::Response, (StatusCode, String)> {
    let file_entry = state
        .database
        .get_file_by_access_token(access_token)
        .await
        .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?;


    let provider = state.provider.read().await;
    let cache = state.cache.read().await;
    let Ok(cached_file) = cache.get(file_entry.uuid).await
    else {
        match provider
            .get_file(file_entry.uuid.to_string())
            .await
        {
            Ok(fetch_type) => match fetch_type {
                Fetchtype::FileData(data) => return Ok(data.clone().into_response()),
                Fetchtype::FileUrl(_url) => {
                    // ToDo: Overthink access_file & get_raw_files_bytes and their interplay together with storageprovider
                    return Ok(Json(Response::new(ResponseData::Error::<String>(ErrorData {
                        reason: "Wrong update token".to_string(),
                    })))
                    .into_response());
                }
            },
            Err(reason) => {
                return Err((StatusCode::INTERNAL_SERVER_ERROR, reason.to_string()));
                /* Ok(Json(Response::new(ResponseData::Error::<String>(ErrorData {
                    reason: format!("{reason}"),
                })))
                .into_response()) */
            }
        }
    };
    let data = cached_file.into_owned(); // ToDo: Think about possibility to remove this copy
    let response = data.into_response();

    Ok(response)
}

pub async fn get_challenge(
    State(state): State<Arc<AppState>>,
    Path(access_token): Path<String>,
) -> Json<Response<responses::GetChallengeData>> {
    let Ok(get_challenge_data) = state.database.get_challenge(access_token).await else {
        return Json(Response::new(ResponseData::Error(ErrorData { reason: "No file found for given access token".to_string() })))
    };

    Json(Response::new(ResponseData::Success(get_challenge_data)))
}

pub async fn verify_challenge(
    State(state): State<Arc<AppState>>,
    Path(access_token): Path<String>,
    Json(json_data): Json<ChallengeData>,
) -> Json<Response<responses::VerifyChallengeData>> {
    let Ok(mut verify_challenge_data) = state.database.get_hash(access_token).await else {
        return Json(Response::new(ResponseData::Error(ErrorData { reason: "No file found for given access token".to_string() })))
    };

    if verify_challenge_data.file_name_hash == Some(json_data.challenge) {
        verify_challenge_data.file_name_hash = None;
        // ToDo: Check if iv and salt needs to be sent here again
        return Json(Response::new(ResponseData::Success(verify_challenge_data)));
    } else {
        return Json(Response::new(ResponseData::Error(ErrorData {
            reason: "Challenge failed".to_string(),
        })));
    }
}
