use super::provider::Fetchtype;
use super::provider::StorageProvider;
use crate::Result;
use async_trait::async_trait;
use bincache::compression::Noop;
use bincache::Cache;
use bincache::CacheBuilder;
use bincache::DiskStrategy;
use std::borrow::Cow;
use std::path::Path;

#[derive(Debug)]
pub struct OnPremiseProvider<'a> {
    path: Cow<'a, Path>,
    storage: Cache<String, DiskStrategy, Noop>,
}

impl OnPremiseProvider<'_> {
    pub fn new<'a>(
        path: Cow<'a, Path>,
        disk_byte_limit: Option<usize>,
        disk_entry_limit: Option<usize>,
    ) -> Self {
        OnPremiseProvider {
            path,
            storage: CacheBuilder::default()
                .with_strategy(DiskStrategy::new(path, disk_byte_limit, disk_entry_limit))
                .build()
                .unwrap(),
        }
        //OnPremiseProvider { path }
    }
}

impl Default for OnPremiseProvider<'_> {
    fn default() -> Self {
        let path = Path::new("uploaded_files").into();
        OnPremiseProvider {
            path,
            storage: CacheBuilder::default()
                .with_strategy(DiskStrategy::new(path, None, None))
                .build()
                .unwrap(),
        }
    }
}

#[async_trait]
impl StorageProvider for OnPremiseProvider<'_> {
    async fn store_file(&self, ident: String, content: &[u8]) -> Result<String> {
        self.storage.put(ident, content).await?;
        Ok(format!("{path}/{ident}", path = self.path.display()))
    }

    async fn delete_file(&self, ident: String) -> Result<()> {
        self.storage.delete(ident).await?;
        Ok(())
    }

    async fn get_file(&self, ident: String) -> Result<Fetchtype> {
        let mut data = self.storage.get(ident).await?;
        Ok(Fetchtype::FileData(data.as_ref()))
    }

    async fn file_exists(&self, ident: String) -> Result<bool> {
        //self.storage.
        //ToDo: bincache add exists
        let file_path = format!("{path}/{ident}", path = self.path.display());

        Ok(Path::new(&file_path).exists())
    }
}
