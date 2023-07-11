use axum::{body::Bytes, extract::Multipart};

use crate::error::{Error, Result};

/// Initial struct which allows file data to be incomplete.
#[derive(Debug, Default)]
pub struct PartialUploadedFile {
    iv: Option<String>,
    salt: Option<String>,
    file_data: Option<Bytes>,
    file_name_data: Option<String>,
    challenge_data: Option<String>,
    challenge_hash: Option<String>,
    error: Option<String>,
}

impl PartialUploadedFile {
    pub async fn from_multipart(mut multipart_formdata: Multipart) -> PartialUploadedFile {
        let mut partial_data: PartialUploadedFile = PartialUploadedFile::default();

        while let Some(field) = multipart_formdata.next_field().await.ok().flatten() {
            let Some(field_name) = field.name() else {
                continue;
            };
            match field_name {
                "iv" => {
                    partial_data.iv = field.text().await.ok();
                }
                "salt" => {
                    partial_data.salt = field.text().await.ok();
                }
                "file_data" => match field.bytes().await {
                    Ok(x) => {
                        partial_data.file_data = Some(x);
                    }
                    Err(e) => {
                        partial_data.error = Some(format!("{:?}", e));
                    }
                },
                "file_name_data" => {
                    partial_data.file_name_data = field.text().await.ok();
                }
                "challenge_data" => {
                    partial_data.challenge_data = field.text().await.ok();
                }
                "challenge_hash" => {
                    partial_data.challenge_hash = field.text().await.ok();
                }
                _ => (),
            }
        }

        partial_data
    }
}

/// Struct which must include complete file data.
#[derive(Debug)]
pub struct UploadedFile {
    pub iv: String,
    pub salt: String,
    pub file_data: Bytes,
    pub file_name_data: String,
    pub challenge_data: String,
    pub challenge_hash: String,
}

/// Convert initial struct into finalized struct. Fails if a field is missing.
impl TryFrom<PartialUploadedFile> for UploadedFile {
    type Error = Error;
    fn try_from(data: PartialUploadedFile) -> Result<Self> {
        if let Some(reason) = data.error {
            return Err(Error::FileUpload { reason });
        }

        Ok(Self {
            iv: data.iv.ok_or(Error::FileDataConversionError("iv"))?,
            salt: data.salt.ok_or(Error::FileDataConversionError("salt"))?,
            file_data: data
                .file_data
                .ok_or(Error::FileDataConversionError("file_data"))?,
            file_name_data: data
                .file_name_data
                .ok_or(Error::FileDataConversionError("file_name_data"))?,
            challenge_data: data
                .challenge_data
                .ok_or(Error::FileDataConversionError("file_name_hash"))?,
            challenge_hash: data
                .challenge_hash
                .ok_or(Error::FileDataConversionError("file_name_hash"))?,
        })
    }
}
