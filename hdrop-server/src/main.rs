mod background_workers;
mod core;
mod error;
mod server;
mod utils;

pub(crate) use crate::error::Result;
pub(crate) use crate::utils::parse_and_upscale_to_mb;
use server::start_server;

#[tokio::main]
async fn main() -> Result<()> {
    start_server().await
}
