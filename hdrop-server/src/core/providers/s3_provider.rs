use async_trait::async_trait;
use hdrop_shared::env;
use regex::Regex;
use s3::{creds::Credentials, region::Region, Bucket};

use super::provider::{Fetchtype, StorageProvider};
use crate::Result;

#[derive(Debug)]
pub struct S3Provider {
    pub bucket: Bucket,
    public_url: String,
}

impl S3Provider {
    pub fn try_from_env() -> Result<Self> {
        let region_custom = Region::Custom {
            region: env::s3_region()?,
            endpoint: env::s3_endpoint()?,
        };
        let credentials = Credentials::new(
            Some(&env::s3_access_key_id()?),
            Some(&env::s3_secret_access_key()?),
            None,
            None,
            None,
        )?;

        let bucket = Bucket::new(
            &env::s3_bucket_name()?,
            region_custom,
            // Credentials are collected from environment, config, profile or instance metadata
            credentials,
        )?
        .with_path_style();

        let regex = Regex::new(r"(?m)/+$")?;
        let public_url = regex.replace(&env::s3_public_url()?, "").to_string();
        Ok(S3Provider { bucket, public_url })
    }
}

#[async_trait]
impl StorageProvider for S3Provider {
    async fn store_file(&mut self, ident: String, content: &[u8]) -> Result<Option<String>> {
        let _response_data = self.bucket.put_object(&ident, content).await?;

        Ok(Some(format!(
            "{s3_host}/{ident}",
            s3_host = self.public_url
        )))
    }

    async fn delete_file(&mut self, ident: String) -> Result<()> {
        let s3_path = ident.as_str();

        // ToDo: Add tracing at failable calls like below throughout the code
        let _response_data = self.bucket.delete_object(s3_path).await?;

        Ok(())
    }

    async fn get_file(&self, ident: String) -> Result<Fetchtype> {
        let url = format!("{}/{}", self.public_url, ident);

        Ok(Fetchtype::FileUrl(url))
    }

    async fn file_exists(&self, ident: String) -> Result<bool> {
        let s3_path = ident.as_str();

        Ok(self.bucket.object_exists(s3_path).await?)
    }
}
