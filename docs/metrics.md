# Metrics

The `hdrop-backend` Prometheus job scrapes `http://api:3001/metrics` every 15 seconds. Production Compose keeps the exporter and Prometheus web UI internal and stores 15 days of samples in the `prometheus_data` volume. Inspect current values from the VPS checkout.

```sh
docker compose exec -T api wget -qO- http://127.0.0.1:3001/metrics
```

| Metric | Type | What it measures |
| --- | --- | --- |
| `database_file_count` | Gauge | File metadata rows in PostgreSQL. |
| `cache_total_capacity_bytes` | Gauge | Total capacity reported by the cache. |
| `cache_used_capacity_bytes` | Gauge | Bytes the cache uses. |
| `used_storage_bytes` | Gauge | Combined size of files in the local upload directory. For S3 bucket usage, use the provider's metrics. |
| `http_requests_total` | Counter | Requests handled by API routes since startup. |
| `http_requests_duration_seconds` | Histogram | Seconds spent handling API requests. |
| `avg_cpu_usage` | Gauge | Mean utilization across visible CPU cores (0 to 1). Multiply by 100 for a percentage. |
| `ram_usage_bytes` | Gauge | Bytes of system memory in use, as seen by the API. |

The API refreshes CPU and memory gauges every 15 seconds. Both HTTP metrics group requests by `method`, `path` and `status`. Matched route templates such as `/v1/files/{access_token}` group different files into one series per route, method and status.

The bundled `InstanceDown` alert fires after five minutes of failed exporter scrapes. [Scrape configuration](../prometheus/prometheus.yml) and [alert rule](../prometheus/alert.rules.yml).
