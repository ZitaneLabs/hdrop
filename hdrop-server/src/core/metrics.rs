pub mod monitoring;
pub mod names;

use crate::core::names::{network, storage, system, GAUGE_NAMES};
use axum::{routing::get, Router};
use metrics::register_gauge;
use metrics_exporter_prometheus::{Matcher, PrometheusBuilder, PrometheusHandle};
use std::{future::ready, net::SocketAddr};

/// Metrics Recorder
fn metrics_app() -> Router {
    let recorder_handle = setup_metrics_recorder();
    Router::new().route("/metrics", get(move || ready(recorder_handle.render())))
}

/// Metrics Setup.
/// Set up all gauges and register them.
fn setup_metrics_recorder() -> PrometheusHandle {
    const EXPONENTIAL_SECONDS: &[f64] = &[
        0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0,
    ];

    let result = PrometheusBuilder::new()
        .set_buckets_for_metric(
            Matcher::Full("http_requests_duration_seconds".to_string()),
            EXPONENTIAL_SECONDS,
        )
        .unwrap()
        .install_recorder()
        .unwrap();

    register_metrics();

    result
}

/// Register all gauges from names module.
fn register_metrics() {
    for name in GAUGE_NAMES {
        register_gauge!(name);
    }
}

/// Separate metrics server.
pub async fn start_metrics_server() {
    let app = metrics_app();

    let addr = SocketAddr::from(([0; 4], 3001));
    tracing::info!("Prometheus exporter listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap()
}
