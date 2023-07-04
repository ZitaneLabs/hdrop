use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod background_workers;
mod core;
mod error;
mod server;
mod utils;

use crate::server::start_metrics_server;

pub(crate) use self::{error::Result, server::Server};

// Initialize global tracing subscriber
fn setup_tracing() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "hdrop_server=debug,hdrop_db=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing as early as possible
    setup_tracing();

    // Load environment variables from .env file (for development)
    match dotenvy::dotenv() {
        Ok(_) => tracing::info!("Loaded environment variables from .env file"),
        Err(err) => {
            tracing::warn!("Failed to load environment variables from .env file: {err}. \
                You can safely ignore this warning if you're explicitly exporting the environment variables.")
        }
    }

    // Start the server
    // The `/metrics` endpoint should not be publicly available. If behind a reverse proxy, this
    // can be achieved by rejecting requests to `/metrics`. In this example, a second server is
    // started on another port to expose `/metrics`.
    let server = Server::new().await?;
    let (_main_server, _metrics_server) = tokio::join!(server.run(), start_metrics_server());

    Ok(())
}
