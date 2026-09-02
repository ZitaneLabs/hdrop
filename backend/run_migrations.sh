#!/usr/bin/env bash

set -euo pipefail

DATABASE_URL=postgres://postgres:postgres@localhost:5432/hdrop
script_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"

pushd "$script_dir/hdrop-db"
diesel migration run --database-url "$DATABASE_URL"
popd
