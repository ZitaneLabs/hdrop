mod providers;

pub use self::providers::{
    provider::{Fetchtype, StorageProvider},
    s3_provider::S3Provider,
};
// Todo change ::* to reasonable use
