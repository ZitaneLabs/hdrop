use async_trait::async_trait;
use regex::Regex;
use s3::{creds::Credentials, region::Region, Bucket};
use std::env;

use super::provider::{Fetchtype, StorageProvider};
use crate::Result;

pub struct S3Provider {
    pub bucket: Bucket,
    public_url: String,
}

impl S3Provider {
    pub fn try_from_env() -> Result<Self> {
        let region_custom = Region::Custom {
            region: env::var("S3_REGION")?,
            endpoint: env::var("S3_ENDPOINT")?,
        };
        let credentials = Credentials::new(
            Some(env::var("S3_ACCESS_KEY_ID")?.as_ref()),
            Some(env::var("S3_SECRET_ACCESS_KEY")?.as_ref()),
            None,
            None,
            None,
        )?;

        let bucket = Bucket::new(
            env::var("S3_BUCKET_NAME")?.as_ref(),
            region_custom,
            // Credentials are collected from environment, config, profile or instance metadata
            credentials,
        )?
        .with_path_style();

        // -- Done
        let regex = Regex::new(r"(?m)/+$")?;
        let public_url = regex.replace(&env::var("S3_PUBLIC_URL")?, "").to_string();
        Ok(S3Provider { bucket, public_url })
    }
}

#[async_trait]
impl StorageProvider for S3Provider {
    async fn store_file(&self, ident: String, content: &[u8]) -> Result<String> {
        let _response_data = self.bucket.put_object(&ident, content).await?;
        //let response_data = bucket.get_object(s3_path).await?;
        //assert_eq!(test, response_data.as_slice());

        Ok(format!("{s3_host}/{ident}", s3_host = self.public_url))
    }

    async fn delete_file(&self, ident: String) -> Result<()> {
        let s3_path = ident.as_str();

        let _response_data = self.bucket.delete_object(s3_path).await?;

        Ok(())
    }

    async fn get_file(&self, ident: String) -> Result<Fetchtype> {
        let url = format!("{}/{}", self.public_url, ident);
        println!("URL: {}", url);

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
