#!/usr/bin/env sh

set -eu

MIGRATION_DIR="${DB_MIGRATION_DIR:-./hdrop-db/migrations}"
MAX_ATTEMPTS="${DB_MIGRATE_MAX_ATTEMPTS:-60}"
RETRY_SECONDS="${DB_MIGRATE_RETRY_SECONDS:-2}"
ATTEMPT=1

DIESEL_BIN="${DIESEL_BIN:-diesel}"
if ! command -v "${DIESEL_BIN}" >/dev/null 2>&1; then
  if [ -x "/usr/local/cargo/bin/diesel" ]; then
    DIESEL_BIN="/usr/local/cargo/bin/diesel"
  else
    echo "Error: diesel CLI not found in PATH and /usr/local/cargo/bin/diesel does not exist." >&2
    exit 1
  fi
fi

# Migrate database, retrying until DB becomes reachable or attempts are exhausted.
until "${DIESEL_BIN}" migration run --migration-dir "${MIGRATION_DIR}"; do
  if [ "${ATTEMPT}" -ge "${MAX_ATTEMPTS}" ]; then
    echo "Error: database migrations failed after ${ATTEMPT} attempts." >&2
    exit 1
  fi
  echo "Database not ready, retrying migrations in ${RETRY_SECONDS}s (${ATTEMPT}/${MAX_ATTEMPTS})..." >&2
  ATTEMPT=$((ATTEMPT + 1))
  sleep "${RETRY_SECONDS}"
done

# Start server
exec pm2-runtime start target/release/hdrop-server --name hdrop-server
