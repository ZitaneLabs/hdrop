# Metrics
> Prometheus metrics are exposed on the `/v1/metrics` endpoint.

| Metric                           | Type        | Description                                      |
| -------------------------------- | ----------- | ------------------------------------------------ |
| `database_file_count`            | `Gauge`     | Number of files currently stored in the database |
| `cache_total_capacity_bytes`     | `Gauge`     | Capacity in bytes currently set for the cache    |
| `cache_used_capacity_bytes`      | `Gauge`     | Number of bytes currently stored in the cache    |
| `used_storage_bytes`             | `Gauge`     | Total number of bytes stored in storage          |
| `http_requests_duration_seconds` | `Histogram` | Duration histogram of http responses             |
| `http_requests_total`            | `Counter`   | Total http requests (resets on restart)          |
| `network_octets_received`        | `Gauge`     | Amount of network bytes received on interface    |
| `network_octets_transmitted`     | `Gauge`     | Amount of network bytes transmitted              |
| `avg_cpu_usage`                  | `Gauge`     | Whole system: Average CPU usage across all cores (0-100)       |
| `ram_usage_bytes`                      | `Gauge`     | Whole system: Amount of bytes currently stored in RAM |
