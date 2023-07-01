use crate::Result;
use async_trait::async_trait;

#[async_trait]
pub trait StorageProvider {
    // String => impl as ref str
    /// Stores the file to the specified StorageProvider. E.g., uploads it to s3 or stores it local directly on the disk.
    async fn store_file(&mut self, ident: String, content: &[u8]) -> Result<Option<String>>;
    /// Deletes the file from the specified StorageProvider.
    async fn delete_file(&mut self, ident: String) -> Result<()>;
    /// Gets the file from the specified StorageProvider. E.g., fetches a download link or gets a datastream directly.
    async fn get_file(&self, ident: String) -> Result<Fetchtype>;
    /// Check if a file exists.
    async fn file_exists(&self, ident: String) -> Result<bool>;
}

pub enum Fetchtype {
    FileData(Vec<u8>),
    FileUrl(String),
}
