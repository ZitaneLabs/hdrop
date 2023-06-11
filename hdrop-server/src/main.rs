mod background_workers;
mod core;
mod error;
mod server;

pub(crate) use crate::error::Result;
use server::start_server;

#[tokio::main]
async fn main() -> Result<()> {
    start_server().await
}
