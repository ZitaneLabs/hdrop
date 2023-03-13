# Metrics
> Prometheus metrics are exposed on the `/v1/metrics` endpoint.

| Metric                             | Type        | Description                                      |
| ---------------------------------- | ----------- | ------------------------------------------------ |
| `hdrop_db_file_count`              | `Gauge`     | Number of files currently stored in the database |
| `hdrop_cached_file_count`          | `Gauge`     | Number of files currently stored in the cache    |
| `hdrop_cached_file_bytes`          | `Gauge`     | Number of bytes currently stored in the cache    |
| `hdrop_storage_stored_files_total` | `Counter`   | Total number of files stored                     |
| `hdrop_storage_stored_bytes_total` | `Counter`   | Total number of bytes stored                     |
| `http_request_duration_seconds`    | `Histogram` | Duration histogram of http responses             |
