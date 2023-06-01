use super::provider::StorageProvider;
use super::provider::{Buffer, Fetchtype};
use async_trait::async_trait;
use s3::creds::Credentials;
use s3::region::Region;
use s3::Bucket;
//use anyhow::Result;
use crate::Result;
use regex::Regex;
use std::env;

pub struct S3Provider {
    pub bucket: Bucket,
    public_url: String,
}

impl S3Provider {
    pub fn new() -> Result<Self> {
        let region_custom = Region::Custom {
            region: env::var("S3_REGION").unwrap(),
            endpoint: env::var("S3_ENDPOINT").unwrap(),
        };
        let credentials = Credentials::new(
            Some(env::var("S3_ACCESS_KEY_ID").unwrap().as_ref()),
            Some(env::var("S3_SECRET_ACCESS_KEY").unwrap().as_ref()),
            None,
            None,
            None,
        )?;

        let bucket = Bucket::new(
            env::var("S3_BUCKET_NAME").unwrap().as_ref(),
            region_custom,
            // Credentials are collected from environment, config, profile or instance metadata
            credentials,
        )?;
        // -- Done
        let regex = Regex::new(&r"\/+$").unwrap();
        let public_url = env::var("S3_PUBLIC_URL").unwrap();
        let sanitizedPublicUrl = format!("{}", regex.replace(&public_url, ""));
        Ok(S3Provider {
            bucket: bucket,
            public_url: public_url,
        })
    }
}

#[async_trait]
impl StorageProvider for S3Provider {
    async fn uploadFile(&self, ident: String, content: &[u8]) -> Result<String> {
        let s3_path = ident.as_str();

        let response_data = self.bucket.put_object(s3_path, content).await?;
        //let response_data = bucket.get_object(s3_path).await?;
        //assert_eq!(test, response_data.as_slice());
        let url = format!("{}/{}", self.public_url, s3_path);
        Ok(url)
    }

    async fn deleteFile(&self, ident: String) -> Result<()> {
        let s3_path = ident.as_str();

        let response_data = self.bucket.delete_object(s3_path).await?;

        Ok(())
    }

    async fn getFile(&self, ident: String) -> Result<Fetchtype> {
        let url = format!("{}/{}", self.public_url, ident);

        Ok(Fetchtype::FileUrl(url))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_upload() {
        /*
        let bucket = self.bucket;
        let s3_path = ident.as_str();

        let test = b"I'm going to S3!";

        let response_data = bucket.put_object(s3_path, content).await?;

        let response_data = bucket.get_object(s3_path).await?;
        //assert_eq!(response_data.status_code(), 200);
        */
    }
}
