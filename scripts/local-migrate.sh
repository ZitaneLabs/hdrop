#!/usr/bin/env bash
set -euo pipefail

source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/common.sh"

require_cmd diesel
load_env_file "local.api.env"

pushd "${ROOT_DIR}/backend/hdrop-db" >/dev/null
diesel migration run --database-url "${DATABASE_URL}"
popd >/dev/null
