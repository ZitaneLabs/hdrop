use super::StateInfos;
use crate::core::{S3Provider, StorageProvider};
use crate::error::{Error, Result};
use axum::{
    body::Bytes,
    extract::{Extension, Multipart, Path, Query, State},
    Json,
};
use chrono::Utc;
use hdrop_db::{Database, InsertFile};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

/* API body equivalent structs */
#[derive(Debug, Deserialize, Clone)]
pub struct Expiry {
    expiry: u64,
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

#[derive(Debug, Default)]
struct PartialUploadedFileData {
    file_data: Option<Bytes>,
    file_name_data: Option<String>,
    file_name_hash: Option<String>,
    salt: Option<String>,
    iv: Option<String>,
}

impl PartialUploadedFileData {
    async fn from_multipart(
        mut multipart_formdata: Multipart,
    ) -> PartialUploadedFileData {
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
pub async fn upload_file(
    Extension(state_info): Extension<Arc<StateInfos<S3Provider>>>,
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
    let storage_provider = &state_info.provider;
    let uuid = Uuid::new_v4();

    // DB update
    let mut db = Database::try_from_env().unwrap();
    let access_token = db.generate_access_token();
    let update_token = Database::generate_update_token();
    let time = Utc::now();
    let mut file = InsertFile {
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
    db.insert_file(&file);

    // S3
    let dataurl = match storage_provider
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

    db.update_file(&file);

    // Test, remove later
    let response_data = storage_provider
        .bucket
        .get_object(uuid.to_string())
        .await
        .expect("S3 Download FAIL");
    assert_eq!(data.file_data, response_data.as_slice());

    Json(Response::new(ResponseData::Success(
        UploadFileResponseData {
            access_token,
            update_token,
        },
    )))
}

pub async fn access_file(Path(access_token): Path<String>) -> Json<Response<()>> {
    todo!();
}

pub async fn delete_file(
    Extension(state_info): Extension<Arc<StateInfos<S3Provider>>>,
    Path(access_token): Path<String>,
    update_token: Query<String>,
) -> Json<Response<()>> {
    let storage_provider = &state_info.provider;
    let uuid = Uuid::new_v4().to_string();

    if let Ok(()) = storage_provider.delete_file(uuid).await {
        Json(Response::new(ResponseData::Success(())))
    } else {
        Json(Response::new(ResponseData::Error(ErrorData {
            reason: "File download Failed".to_string(),
        })))
    }
}

pub async fn update_file_expiry(
    Path(access_token): Path<String>,
    update_token: Query<String>,
    Json(expiry): Json<Expiry>,
) -> Json<Response<()>> {
    todo!();
}

pub async fn get_raw_file_bytes(
    Path(access_token): Path<String>,
) -> Json<Response<UploadFileResponseData>> {
    todo!();
}

pub async fn get_challenge(Path(access_token): Path<String>) -> Json<Response<()>> {
    todo!();
}

pub async fn verify_challenge(Path(access_token): Path<String>) -> Json<Response<()>> {
    todo!();
}
