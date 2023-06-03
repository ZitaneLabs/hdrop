use super::{
    multipart::{PartialUploadedFile, UploadedFile},
    AppState,
};
use crate::error::{Error, Result};
use axum::{
    body::Bytes,
    extract::{Multipart, Path, Query, State},
    Json,
};
use chrono::Utc;
use hdrop_db::{Database, InsertFile};
use hdrop_shared::requests::{ChallengeData, ExpiryData};
use hdrop_shared::{responses, ErrorData, Response, ResponseData};
use std::sync::Arc;
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

    let access_token = state.database.generate_access_token().await.unwrap();
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
    let mut file = state.database.insert_file(file).await.unwrap();

    // S3
    let dataurl = match state
        .provider
        .read()
        .await
        .store_file(uuid.to_string().clone(), &data.file_data)
        .await
    {
        Ok(url) => url,
        Err(err) => {
            return Json(Response::new(ResponseData::Error(ErrorData {
                reason: format!("{:?}", err),
            })))
        }
    };
    // DB Update FileUrl here
    file.dataUrl = Some(dataurl); // ToDo: change to dataUrl

    state.database.update_file(file).await.unwrap();

    // Test, remove later
    let response_data = state
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

pub async fn access_file(
    State(state): State<Arc<AppState>>,
    Path(access_token): Path<String>,
) -> Json<Response<responses::FileMetaData>> {
    let Ok((file_url, file_name_data, iv, salt)) = state.database.get_file_metadata(access_token).await else {
        return Json(Response::new(ResponseData::Error(ErrorData { reason: "No file found for given access token".to_string() })))
    };

    Json(Response::new(ResponseData::Success(
        responses::FileMetaData {
            file_url,
            file_name_data,
            iv,
            salt,
        },
    )))
}

// #[axum::debug_handler]
pub async fn delete_file(
    State(state): State<Arc<AppState>>,
    Path(access_token): Path<String>,
    Query(query): Query<UpdateTokenQuery>,
) -> Json<Response<()>> {
    /* let uuid = Uuid::new_v4().to_string();

    if let Ok(()) = state.provider.read().await.delete_file(uuid).await {
        Json(Response::new(ResponseData::Success(())))
    } else {

    */
    // ToDo: Remove and get this from state later

    let file = state
        .database
        .get_file_by_access_token(&access_token)
        .await
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
            state.database.delete_file(file).await.unwrap();
            Json(Response::new(ResponseData::Success(())))
        } else {
            Json(Response::new(ResponseData::Error(ErrorData {
                reason: "File download Failed".to_string(),
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
        .unwrap();

    if file.updateToken == query.update_token {
        file.expiresAt = Utc::now() + chrono::Duration::seconds(expiry_data.expiry);
        state.database.update_file_expiry(file).await.unwrap();
        Json(Response::new(ResponseData::Success(())))
    } else {
        Json(Response::new(ResponseData::Error(ErrorData {
            reason: "Wrong update token".to_string(),
        })))
    }
}

pub async fn get_raw_file_bytes(Path(access_token): Path<String>) -> Json<Response<()>> {
    todo!();
}

pub async fn get_challenge(
    State(state): State<Arc<AppState>>,
    Path(access_token): Path<String>,
) -> Json<Response<responses::GetChallengeData>> {
    let Ok((challenge, iv, salt)) = state.database.get_challenge(access_token).await else {
        return Json(Response::new(ResponseData::Error(ErrorData { reason: "No file found for given access token".to_string() })))
    };

    Json(Response::new(ResponseData::Success(
        responses::GetChallengeData {
            challenge,
            iv,
            salt,
        },
    )))
}

pub async fn verify_challenge(
    State(state): State<Arc<AppState>>,
    Path(access_token): Path<String>,
    Json(json_data): Json<ChallengeData>,
) -> Json<Response<responses::VerifyChallengeData>> {
    let Ok((file_name_hash, iv, salt)) = state.database.get_hash(access_token).await else {
        return Json(Response::new(ResponseData::Error(ErrorData { reason: "No file found for given access token".to_string() })))
    };

    if file_name_hash == json_data.challenge {
        // ToDo: Check if iv and salt needs to be sent here again
        return Json(Response::new(ResponseData::Success(
            responses::VerifyChallengeData { iv, salt },
        )));
    } else {
        return Json(Response::new(ResponseData::Error(ErrorData {
            reason: "Challenge failed".to_string(),
        })));
    }
}
