use crate::error::{Error, Result};
use axum::{body::Bytes, extract::Multipart};

/// Initial struct which allows file data to be incomplete.
#[derive(Debug, Default)]
pub struct PartialUploadedFile {
    file_data: Option<Bytes>,
    file_name_data: Option<String>,
    file_name_hash: Option<String>,
    salt: Option<String>,
    iv: Option<String>,
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

/// Struct which must include complete file data.
#[derive(Debug)]
pub struct UploadedFile {
    pub file_data: Bytes,
    pub file_name_data: String,
    pub file_name_hash: String,
    pub iv: String,
    pub salt: String,
}

/// Convert initial struct into finalized struct. Fails if a field is missing.
impl TryFrom<PartialUploadedFile> for UploadedFile {
    type Error = Error;
    fn try_from(data: PartialUploadedFile) -> Result<Self> {
        if let Some(reason) = data.error {
            return Err(Error::FileUpload { reason });
        }

        Ok(Self {
            file_data: { 
            let data =  
                data
                .file_data
                .ok_or(Error::FileDataConversionError("file_data"))?;

            if Bytes::len(&data) > 200 {
                return Err(Error::FileLimitExceeded("200", "200"));
            }

            data
            },
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
