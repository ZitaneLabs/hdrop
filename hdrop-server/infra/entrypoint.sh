#!/usr/bin/env sh

# Migrate database
diesel migration run --migration-dir ./hdrop-db/migrations

# Start server
cargo run --release --bin hdrop-server