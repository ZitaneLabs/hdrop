#!/usr/bin/env bash

# This script is used to run the server locally.

set -euo pipefail

trap 'docker compose down' EXIT SIGINT
docker compose up --build --wait -d

pushd hdrop-db
DATABASE_URL="postgres://postgres:postgres@localhost:5432/hdrop" diesel migration run
popd

cargo build --release -p hdrop-server
export RUST_BACKTRACE=1
export PORT=8080
export CORS_ORIGIN="*"
export S3_ACCESS_KEY_ID="dev"
export S3_BUCKET_NAME="hdrop"
export S3_ENDPOINT="http://localhost:4566"
export S3_PUBLIC_URL="http://localhost:4566/hdrop"
export S3_REGION="eu-west-1"
export S3_SECRET_ACCESS_KEY="dev"
export DATABASE_URL="postgres://postgres:postgres@localhost:5432/hdrop"
export CACHE_STRATEGY="memory"
export CACHE_MEMORY_LIMIT_MB="100"
export STORAGE_PROVIDER="s3"
#export CACHE_DISK_LIMIT_MB="100"

cargo run --release -p hdrop-server
