use crate::Result;
use async_trait::async_trait;

#[async_trait]
pub trait StorageProvider {
    // String => impl as ref str
    async fn uploadFile(&self, ident: String, content: &[u8]) -> Result<String>;
    async fn deleteFile(&self, ident: String) -> Result<()>;
    async fn getFile(&self, ident: String) -> Result<Fetchtype>;
}

pub type Buffer = String;

pub enum Fetchtype {
    FileData(Buffer),
    FileUrl(String),
}
