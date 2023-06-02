use super::AppState;
use crate::error::{Error, Result};
use axum::{
    body::Bytes,
    extract::{Multipart, Path, Query, State},
    Json,
};
use chrono::Utc;
use hdrop_db::{Database, InsertFile};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

/* API body equivalent structs */
#[derive(Debug, Deserialize, Clone)]
pub struct ExpiryData {
    expiry: i64,
}
#[derive(Debug, Deserialize, Clone)]
pub struct ChallengeData {
    challenge: String,
}
#[derive(Debug, Serialize)]
pub struct FileMetaData {
    file_url: Option<String>,
    file_name_data: String,
    iv: String,
    salt: String,
}

#[derive(Debug, Serialize)]
pub struct Response<T: Serialize> {
    status: &'static str,
    data: ResponseData<T>,
}

impl<T: Serialize> Response<T> {
    pub fn new(data: ResponseData<T>) -> Self {
        let status = match data {
            ResponseData::Success(_) => "success",
            ResponseData::Error(_) => "error",
        };
        Self { status, data }
    }
}

#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum ResponseData<T: Serialize> {
    Success(T),
    Error(ErrorData),
}

#[derive(Debug, Serialize)]
pub struct ErrorData {
    reason: String,
}

#[derive(Debug, Serialize)]
pub struct UploadFileResponseData {
    access_token: String,
    update_token: String,
}
#[derive(Debug, Serialize)]
pub struct GetChallengeData {
    challenge: String,
    iv: String,
    salt: String,
}
// ToDo Idea: Squash GetChallengeData & ChallengeFinishData together with Option<String> on challenge and serde skip serialize if None
#[derive(Debug, Serialize)]
pub struct ChallengeFinishData {
    iv: String,
    salt: String,
}

#[derive(Debug, Default)]
struct PartialUploadedFileData {
    file_data: Option<Bytes>,
    file_name_data: Option<String>,
    file_name_hash: Option<String>,
    salt: Option<String>,
    iv: Option<String>,
}

impl PartialUploadedFileData {
    async fn from_multipart(mut multipart_formdata: Multipart) -> PartialUploadedFileData {
        let mut partial_data: PartialUploadedFileData = PartialUploadedFileData::default();

        while let Some(field) = multipart_formdata.next_field().await.ok().flatten() {
            let Some(field_name) = field.name() else {
                continue;
            };
            match field_name {
                "file_data" => {
                    partial_data.file_data = field.bytes().await.ok();
                }
                "file_name_data" => {
                    partial_data.file_name_data = field.text().await.ok();
                }
                "file_name_hash" => {
                    partial_data.file_name_hash = field.text().await.ok();
                }
                "iv" => {
                    partial_data.iv = field.text().await.ok();
                }
                "salt" => {
                    partial_data.salt = field.text().await.ok();
                }
                _ => (),
            }
        }

        partial_data
    }
}

#[derive(Debug)]
struct UploadedFileData {
    file_data: Bytes,
    file_name_data: String,
    file_name_hash: String,
    iv: String,
    salt: String,
}

impl TryFrom<PartialUploadedFileData> for UploadedFileData {
    type Error = Error;
    fn try_from(data: PartialUploadedFileData) -> Result<Self> {
        // ToDo: Async?
        Ok(Self {
            file_data: data
                .file_data
                .ok_or(Error::FileDataConversionError("file_data"))?,
            file_name_data: data
                .file_name_data
                .ok_or(Error::FileDataConversionError("file_name_data"))?,
            file_name_hash: data
                .file_name_hash
                .ok_or(Error::FileDataConversionError("file_name_hash"))?,
            iv: data.iv.ok_or(Error::FileDataConversionError("iv"))?,
            salt: data.salt.ok_or(Error::FileDataConversionError("salt"))?,
        })
    }
}

/* Routes */
// #[axum::debug_handler]
pub async fn upload_file(
    State(state): State<Arc<AppState>>,
    multipart_formdata: Multipart,
) -> Json<Response<UploadFileResponseData>> {
    // Multipart Upload to Server
    let data: UploadedFileData = match PartialUploadedFileData::from_multipart(multipart_formdata)
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
        UploadFileResponseData {
            access_token,
            update_token,
        },
    )))
}

pub async fn access_file(
    State(state): State<Arc<AppState>>,
    Path(access_token): Path<String>,
) -> Json<Response<FileMetaData>> {
    let Ok((file_url, file_name_data, iv, salt)) = state.database.get_file_metadata(access_token).await else {
        return Json(Response::new(ResponseData::Error(ErrorData { reason: "No file found for given access token".to_string() })))   
    };

    Json(Response::new(ResponseData::Success(FileMetaData {
        file_url,
        file_name_data,
        iv,
        salt,
    })))
}

// #[axum::debug_handler]
pub async fn delete_file(
    State(state): State<Arc<AppState>>,
    Path(access_token): Path<String>,
    Query(update_token): Query<String>,
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

    if file.updateToken == update_token.to_string() {
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
    Query(update_token): Query<String>,
    Json(expiry_data): Json<ExpiryData>,
) -> Json<Response<()>> {
    let mut file = state
        .database
        .get_file_by_access_token(access_token)
        .await
        .unwrap();

    if file.updateToken == update_token.to_string() {
        file.expiresAt = Utc::now() + chrono::Duration::seconds(expiry_data.expiry);
        state.database.update_file_expiry(file).await.unwrap();
        Json(Response::new(ResponseData::Success(())))
    } else {
        Json(Response::new(ResponseData::Error(ErrorData {
            reason: "Wrong update token".to_string(),
        })))
    }
}

pub async fn get_raw_file_bytes(
    Path(access_token): Path<String>,
) -> Json<Response<UploadFileResponseData>> {
    todo!();
}

pub async fn get_challenge(
    State(state): State<Arc<AppState>>,
    Path(access_token): Path<String>,
) -> Json<Response<GetChallengeData>> {
    let Ok((challenge, iv, salt)) = state.database.get_challenge(access_token).await else {
        return Json(Response::new(ResponseData::Error(ErrorData { reason: "No file found for given access token".to_string() })))   
    };

    Json(Response::new(ResponseData::Success(GetChallengeData {
        challenge,
        iv,
        salt,
    })))
}

pub async fn verify_challenge(
    State(state): State<Arc<AppState>>,
    Path(access_token): Path<String>,
    Json(json_data): Json<ChallengeData>,
) -> Json<Response<ChallengeFinishData>> {
    let Ok((file_name_hash, iv, salt)) = state.database.get_hash(access_token).await else {
        return Json(Response::new(ResponseData::Error(ErrorData { reason: "No file found for given access token".to_string() })))   
    };

    if file_name_hash == json_data.challenge {
        // ToDo: Check if iv and salt needs to be sent here again
        return Json(Response::new(ResponseData::Success(ChallengeFinishData {
            iv,
            salt,
        })));
    } else {
        return Json(Response::new(ResponseData::Error(ErrorData {
            reason: "Challenge failed".to_string(),
        })));
    }
}
