use crate::core::monitoring::SystemMonitoring;
use hdrop_shared::metrics::names;
use std::time::Duration;
pub struct MetricsUpdater {
    system: SystemMonitoring,
}

impl MetricsUpdater {
    pub fn new() -> Self {
        Self {
            system: SystemMonitoring::new(),
        }
    }

    /// Time-based update of all metrics except for the self updating ones (requests)
    pub async fn update_metrics(mut self) {
        loop {
            // Update RAM
            let ram_status = self.system.ram_status();

            metrics::gauge!(names::system::RAM_USAGE, ram_status.used() as f64);

            // Update CPU
            let cpu_status = self.system.cpu_status();
            let len = cpu_status.len() as f64;
            let mut added_up_usage = 0.;
            for cpu in cpu_status {
                added_up_usage += cpu.utilization();
            }

            let average = added_up_usage / len;

            metrics::gauge!(names::system::AVG_CPU_USAGE, average);

            // Update Network
            let mut network_status = self.system.network_status().into_iter();
            let (received, transmitted) = match network_status.next() {
                Some(network) => (network.received(), network.transmitted()),
                None => (0, 0),
            };

            metrics::gauge!(names::network::NETWORK_OCTETS_RECEIVED, received as f64);

            metrics::gauge!(
                names::network::NETWORK_OCTETS_TRANSMITTED,
                transmitted as f64
            );

            tokio::time::sleep(Duration::from_secs(60)).await;
        }
    }
}

pub mod metrics_middleware {
    use axum::{extract::MatchedPath, http::Request, middleware::Next, response::IntoResponse};
    use std::time::Instant;

    /// Middleware which is plugged in to track everything related to requests.
    pub async fn track_requests<B>(req: Request<B>, next: Next<B>) -> impl IntoResponse {
        let start = Instant::now();
        let path = if let Some(matched_path) = req.extensions().get::<MatchedPath>() {
            matched_path.as_str().to_owned()
        } else {
            req.uri().path().to_owned()
        };
        let method = req.method().clone();

        let response = next.run(req).await;

        let latency = start.elapsed().as_secs_f64();
        let status = response.status().as_u16().to_string();

        let labels = [
            ("method", method.to_string()),
            ("path", path),
            ("status", status),
        ];

        metrics::increment_counter!("http_requests_total", &labels);
        metrics::histogram!("http_requests_duration_seconds", latency, &labels);

        response
    }
}
