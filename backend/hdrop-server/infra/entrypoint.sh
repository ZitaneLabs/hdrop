#!/usr/bin/env sh
set -eu

# Migrate database
diesel migration run --migration-dir ./hdrop-db/migrations

# Start server
exec /usr/local/bin/hdrop-server
