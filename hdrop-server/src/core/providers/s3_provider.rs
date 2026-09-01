use async_trait::async_trait;
use hdrop_shared::{env, metrics::UpdateMetrics};
use object_store::{
    aws::{AmazonS3, AmazonS3Builder},
    path::Path,
    ClientOptions,
    Error as ObjectStoreError,
    ObjectStoreExt,
};

use super::provider::{Fetchtype, StorageProvider};
use crate::Result;

#[derive(Debug)]
pub struct S3Provider {
    pub store: AmazonS3,
    public_url: String,
}

impl S3Provider {
    pub fn try_from_env() -> Result<Self> {
        let region = env::s3_region()?;
        let endpoint = env::s3_endpoint()?;
        let access_key = normalize_credential(env::s3_access_key_id()?);
        let secret_key = normalize_credential(env::s3_secret_access_key()?);
        let bucket = env::s3_bucket_name()?;
        let virtual_hosted_style_request = env::s3_virtual_hosted_style_request()?;
        let client_options = match env::s3_request_timeout()? {
            Some(timeout) => ClientOptions::new().with_timeout(timeout),
            None => ClientOptions::new().with_timeout_disabled(),
        };
        let allow_http = endpoint.starts_with("http://");
        let endpoint = normalize_endpoint(&endpoint, &bucket);

        let store = AmazonS3Builder::new()
            .with_region(region)
            .with_endpoint(endpoint)
            .with_bucket_name(bucket)
            .with_access_key_id(access_key)
            .with_secret_access_key(secret_key)
            .with_client_options(client_options)
            .with_virtual_hosted_style_request(virtual_hosted_style_request)
            .with_disable_bulk_delete(true)
            .with_allow_http(allow_http)
            .build()?;

        let public_url = normalize_public_url(&env::s3_public_url()?);
        Ok(S3Provider { store, public_url })
    }
}

fn normalize_credential(credential: String) -> String {
    credential.replace('\n', "")
}

fn normalize_endpoint(endpoint: &str, bucket: &str) -> String {
    let endpoint = endpoint.trim_end_matches('/');
    endpoint
        .strip_suffix(&format!("/{bucket}"))
        .unwrap_or(endpoint)
        .to_string()
}

fn normalize_public_url(public_url: &str) -> String {
    public_url.trim_end_matches('/').to_string()
}

#[async_trait]
impl StorageProvider for S3Provider {
    async fn store_file(&mut self, ident: String, content: &[u8]) -> Result<Option<String>> {
        self.store
            .put(&Path::from(ident.as_str()), content.to_vec().into())
            .await?;

        Ok(Some(format!(
            "{s3_host}/{ident}",
            s3_host = self.public_url
        )))
    }

    async fn delete_file(&mut self, ident: String) -> Result<()> {
        self.store.delete(&Path::from(ident.as_str())).await?;

        Ok(())
    }

    async fn get_file(&self, ident: String) -> Result<Fetchtype> {
        let url = format!("{}/{}", self.public_url, ident);

        Ok(Fetchtype::FileUrl(url))
    }

    async fn file_exists(&self, ident: String) -> Result<bool> {
        match self.store.head(&Path::from(ident.as_str())).await {
            Ok(_) => Ok(true),
            Err(ObjectStoreError::NotFound { .. }) => Ok(false),
            Err(error) => Err(error.into()),
        }
    }
}

impl UpdateMetrics for S3Provider {}
