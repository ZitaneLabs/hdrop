#!/usr/bin/env bash
set -euo pipefail

source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/common.sh"

require_cmd docker
load_env_file "prod.compose.env"

if [[ -z "${API_IMAGE:-}" || -z "${WEB_IMAGE:-}" ]]; then
  echo "Error: API_IMAGE and WEB_IMAGE must be set in config/prod.compose.env(.example)." >&2
  exit 1
fi

docker push "${API_IMAGE}"
docker push "${WEB_IMAGE}"

echo "Published images: ${API_IMAGE}, ${WEB_IMAGE}"
