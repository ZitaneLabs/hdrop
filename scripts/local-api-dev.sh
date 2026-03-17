#!/usr/bin/env bash
set -euo pipefail

source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/common.sh"

require_cmd cargo
require_cmd diesel

cleanup() {
  compose_for_env local down --remove-orphans
}
trap cleanup EXIT SIGINT

compose_for_env local up --build --wait -d postgres localstack prometheus
"${ROOT_DIR}/scripts/local-migrate.sh"

load_env_file "local.api.env"

pushd "${ROOT_DIR}/backend" >/dev/null
cargo build --release -p hdrop-server
cargo run --release -p hdrop-server
popd >/dev/null
