use axum::{routing::get, Router};
use hdrop_shared::metrics::names;
use metrics::{register_counter, register_gauge, register_histogram};
use metrics_exporter_prometheus::{Matcher, PrometheusBuilder, PrometheusHandle};
use std::{future::ready, net::SocketAddr};

pub struct PrometheusMetricsServer {}

impl PrometheusMetricsServer {
    pub fn new() -> Self {
        Self {}
    }

    /// Metrics Recorder.
    fn metrics_app(&self) -> Router {
        let recorder_handle = self.setup_metrics_recorder();
        Router::new().route("/metrics", get(move || ready(recorder_handle.render())))
    }

    /// Metrics Setup.
    /// Set up all gauges and register them.
    fn setup_metrics_recorder(&self) -> PrometheusHandle {
        const EXPONENTIAL_SECONDS: &[f64] = &[
            0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0,
        ];

        let result = PrometheusBuilder::new()
            .set_buckets_for_metric(
                Matcher::Full(names::network::HTTP_REQUESTS_DURATION_SECONDS.to_string()),
                EXPONENTIAL_SECONDS,
            )
            .unwrap()
            .install_recorder()
            .unwrap();

        self.register_metrics();

        result
    }

    /// Register all gauges from names module.
    fn register_metrics(&self) {
        for name in names::GAUGE_NAMES {
            register_gauge!(name);
        }
        for name in names::HISTOGRAM_NAMES {
            register_histogram!(name);
        }
        for name in names::COUNTER_NAMES {
            register_counter!(name);
        }
    }

    /// Run the metrics server.
    pub async fn run(&self) {
        let app = self.metrics_app();

        let addr = SocketAddr::from(([0; 4], 3001));
        tracing::info!("Prometheus exporter listening on {}", addr);
        axum::Server::bind(&addr)
            .serve(app.into_make_service())
            .await
            .unwrap()
    }
}
