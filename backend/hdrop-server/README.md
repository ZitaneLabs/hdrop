# hdrop Rust API

## Run the API locally

Install stable Rust, a C build toolchain and the PostgreSQL client development library (`libpq-dev` on Debian/Ubuntu, `libpq-devel` on Fedora/RHEL). On macOS, run `brew install libpq` and add `$(brew --prefix libpq)/bin` to your `PATH` so the build finds `pg_config`.

From the repository root, install Diesel CLI with PostgreSQL support, start only the development database and run migrations. The helper uses the database's local development credentials.

```sh
cargo install --locked diesel_cli --no-default-features --features postgres
docker compose -f docker-compose.dev.yml up -d --wait postgres
bash backend/run_migrations.sh
```

Build and run the API in debug mode. Encrypted uploads persist in your user's development data directory.

```sh
export DATABASE_URL=postgres://postgres:postgres@localhost:5432/hdrop
export STORAGE_PROVIDER=local
export LOCAL_STORAGE_DIR="${XDG_DATA_HOME:-$HOME/.local/share}/hdrop-dev/files"
export CACHE_STRATEGY=memory
export CORS_ORIGIN=http://localhost:3000
mkdir -p "$LOCAL_STORAGE_DIR"
cargo run --locked --manifest-path backend/Cargo.toml -p hdrop-server
```

In another terminal, `curl -fsS http://localhost:8080/status` should print `OK`. Start the [frontend development server](../../frontend/README.md#development) for browser uploads.

Press Ctrl+C to stop the API. From the repository root, `docker compose -f docker-compose.dev.yml stop postgres` stops the database and keeps its data.

## Database migrations

From `backend/hdrop-db`, run `diesel migration generate <name>` and edit the generated `up.sql` and `down.sql`. From the repository root, `bash backend/run_migrations.sh` applies the migrations to the development database. Diesel updates `src/schema.rs` using the [database configuration](../hdrop-db/diesel.toml). Update the [Rust models](../hdrop-db/src/models.rs) when the schema changes.

## Standalone API settings

The standalone Rust binary reads environment variables and can load a local `.env`. The VPS installer defaults to local storage, a disk cache and a 500 MB upload limit. In production Compose, Caddy handles public HTTPS and the API listens on container port 3000.

| Setting | Standalone default | Purpose |
| --- | --- | --- |
| `DATABASE_URL` | Required | PostgreSQL connection URL for file metadata |
| `HDROP_PORT`, `PROMETHEUS_PORT` | `8080`, `3001` | HTTP API and Prometheus `/metrics` listener ports |
| `CORS_ORIGIN` | `*` | One allowed browser origin or a comma-separated list of full origins |
| `SINGLE_FILE_LIMIT_MB` | `100` | Maximum upload request size in decimal MB |
| `STORAGE_PROVIDER` | Required | `local` for disk or `s3` for object storage |
| `LOCAL_STORAGE_DIR`, `LOCAL_STORAGE_LIMIT_MB` | `./files`, no limit | Local file directory and capacity |
| `CACHE_STRATEGY` | `memory` | `memory`, `disk` or `hybrid` |
| `CACHE_DIR` | `./file_cache` | Cache directory for disk and hybrid strategies |
| `CACHE_MEMORY_LIMIT_MB`, `CACHE_DISK_LIMIT_MB` | No limit | Capacity per cache tier in decimal MB |

S3 requires `S3_REGION`, `S3_ENDPOINT`, `S3_BUCKET_NAME`, `S3_ACCESS_KEY_ID`, `S3_SECRET_ACCESS_KEY` and a browser-accessible `S3_PUBLIC_URL`. For `S3_ADDRESSING_STYLE`, use `path` (default) or `virtual`. To enable an overall request timeout, set `S3_REQUEST_TIMEOUT_SECS` to a positive integer. See the [S3-compatible configuration example](../../docs/installation.md#configure-and-start-the-droplet).
