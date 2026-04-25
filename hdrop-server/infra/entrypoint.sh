#!/usr/bin/env sh

# Migrate database
diesel migration run --migration-dir ./hdrop-db/migrations

# Start server
pm2-runtime start /usr/local/bin/hdrop-server --name hdrop-server
