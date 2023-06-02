mod core;
mod error;
mod server;
pub(crate) use crate::error::{Error, Result};
use server::start_server;

#[tokio::main]
async fn main() {
    start_server().await;
}
