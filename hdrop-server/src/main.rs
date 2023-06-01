mod core;
mod error;
mod server;
use crate::core::{S3Provider, StorageProvider};
use hdrop_db::{Database, InsertFile};
pub(crate) use crate::error::{Error, Result};
//use crate::schema::files::dataUrl;
use axum::{
    body::Bytes,
    extract::{Extension, Multipart, Path, Query, State},
    routing::{delete, get, post},
    Json, Router,
};
use chrono::Utc;
use dotenvy;
use serde::{Deserialize, Serialize};
use std::{env, sync::Arc};
use tokio::sync::RwLock;
use uuid::Uuid;
#[derive(Clone)]
struct StateInfos<T: StorageProvider> {
    pub provider: T,
}

impl<T: StorageProvider> StateInfos<T> {
    pub fn new(provider: T) -> Self {
        Self { provider: provider }
    }
}

#[tokio::main]
async fn main() {
    // Load environment variables from .env file.
    // Fails if .env file not found, not readable or invalid.
    dotenvy::from_filename_override(".env.example").expect(".env.example file could not be loaded");

    for (key, value) in env::vars() {
        println!("{key}: {value}");
    }

    let storageprovider = S3Provider::new().unwrap(); // ToDo: Add env var to decide which storageprovider
    let stateInfo = Arc::new(StateInfos::new(storageprovider));

    let app = Router::new()
        .route("/test", get(|| async { "Hello, World!" }))
        .route("/v1/files", post(uploadFile))
        .route("/v1/files/:access_token", get(accessFile))
        .route("/v1/files/:access_token", delete(deleteFile))
        .route("/v1/files/:access_token/expiry", post(updateFileExpiry))
        .route("/v1/files/:access_token/raw", get(getRawFileBytes))
        .route("/v1/files/:access_token/challenge", get(getChallenge))
        .route("/v1/files/:access_token/challenge", post(verifyChallenge))
        //.with_state(stateInfo)
        .layer(Extension(stateInfo));

    // run it with hyper on localhost:3000
    axum::Server::bind(&"0.0.0.0:8080".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

/* handlers */
#[derive(Debug, Deserialize, Clone)]
pub struct Expiry {
    expiry: u64,
}
/*
// Responses
#[derive(Debug, Serialize)]
enum ResponesType<T> {
    Success(T),
    Error(ErrorData)
}

#[derive(Debug, Serialize)]
pub struct Response<T> {
    status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<T>,
}

#[derive(Debug, Serialize)]
pub struct ErrorData {
    reason: String,
}

impl<T> Response<T>
where
    T: Serialize,
{
    fn success() -> Response<()> {
        Response {
            status: "success".to_string(),
            data: None,
        }
    }

    fn octet_steam(octet_stream: Vec<u8>) -> Response<Vec<u8>> {
        Response {
            status: "success".to_string(),
            data: Some(octet_stream),
        }
    }

    fn missing_update_token() -> Response<ErrorData> {
        Response {
            status: "error".to_string(),
            data: Some(ErrorData {
                reason: "Missing update token".to_string(),
            }),
        }
    }
}
*/

#[derive(Debug, Serialize)]
struct Response<T: Serialize> {
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
enum ResponseData<T: Serialize> {
    Success(T),
    Error(ErrorData),
}

#[derive(Debug, Serialize)]
struct ErrorData {
    reason: String,
}

#[derive(Debug, Serialize)]
struct UploadFileResponseData {
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
        &mut self,
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

async fn uploadFile(
    Extension(stateInfo): Extension<Arc<StateInfos<S3Provider>>>,
    mut multipart_formdata: Multipart,
) -> Json<Response<UploadFileResponseData>> {
    // Multipart Upload to Server
    let data: UploadedFileData = match PartialUploadedFileData::default()
        .from_multipart(multipart_formdata)
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
    let s3provider = &stateInfo.provider;
    let uuid = Uuid::new_v4();

    // DB update
    let mut db = Database::new().unwrap();
    let access_token = db.generate_access_token();
    let update_token = Database::generate_update_token();
    let time = Utc::now();
    let mut file = InsertFile {
        id: None,
        uuid: uuid,
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
    let Ok(dataurl) = s3provider.uploadFile(uuid.to_string().clone(), &data.file_data).await else {
        return Json(Response::new(ResponseData::Error(ErrorData {
            reason: "S3 upload failed".to_string(),
        })));
    };
    // DB Update FileUrl here
    file.dataUrl = Some(dataurl); // ToDo: change to dataUrl
    db.update_file(&file);

    // Test, remove later
    let response_data = s3provider
        .bucket
        .get_object(uuid.to_string())
        .await
        .expect("S3 Download FAIL");
    assert_eq!(data.file_data, response_data.as_slice());

    Json(Response::new(ResponseData::Success(
        UploadFileResponseData {
            access_token: access_token,
            update_token: update_token,
        },
    )))
}

async fn updateFileExpiry(
    Path(access_token): Path<String>,
    update_token: Query<String>,
    Json(expiry): Json<Expiry>,
) -> Json<Response<()>> {
    todo!();
}

async fn deleteFile(
    Extension(stateInfo): Extension<Arc<StateInfos<S3Provider>>>,
    Path(access_token): Path<String>,
    update_token: Query<String>,
) -> Json<Response<()>> {
    let s3provider = &stateInfo.provider;
    let uuid = Uuid::new_v4().to_string();

    if let Ok(()) = s3provider.deleteFile(uuid).await {
        Json(Response::new(ResponseData::Success(())))
    } else {
        Json(Response::new(ResponseData::Error(ErrorData {
            reason: "File download Failed".to_string(),
        })))
    }
}

async fn accessFile(Path(access_token): Path<String>) -> Json<Response<()>> {
    todo!();
}

async fn getRawFileBytes(
    Path(access_token): Path<String>,
) -> Json<Response<UploadFileResponseData>> {
    todo!();
}

async fn getChallenge(Path(access_token): Path<String>) -> Json<Response<()>> {
    todo!();
}

async fn verifyChallenge(Path(access_token): Path<String>) -> Json<Response<()>> {
    todo!();
}
