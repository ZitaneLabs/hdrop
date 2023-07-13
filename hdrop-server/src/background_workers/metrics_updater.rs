use hdrop_shared::metrics::names;

use crate::core::monitoring::SystemMetrics;

pub struct MetricsUpdater {
    system: SystemMetrics,
}

impl MetricsUpdater {
    pub fn new() -> Self {
        Self {
            system: SystemMetrics::new(),
        }
    }

    /// Time-based update of all metrics except for the self updating ones (requests)
    pub async fn run(mut self) {
        loop {
            // Update RAM
            let ram_status = self.system.ram_status();

            metrics::gauge!(names::system::RAM_USAGE_B, ram_status.used() as f64);

            // Update CPU
            let cpu_status = self.system.cpu_status();
            let len = cpu_status.len() as f64;
            let mut added_up_usage = 0.;
            for cpu in cpu_status {
                added_up_usage += cpu.utilization();
            }

            let average = added_up_usage / len;

            metrics::gauge!(names::system::AVG_CPU_USAGE, average);
        }
    }
}

pub mod metrics_middleware {
    use std::time::Instant;

    use axum::{extract::MatchedPath, http::Request, middleware::Next, response::IntoResponse};
    use hdrop_shared::metrics::names;

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

        metrics::increment_counter!(names::network::HTTP_REQUESTS_TOTAL, &labels);
        metrics::histogram!(
            names::network::HTTP_REQUESTS_DURATION_SECONDS,
            latency,
            &labels
        );

        response
    }
}
