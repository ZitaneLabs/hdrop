#!/usr/bin/env bash
set -euo pipefail

source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/common.sh"

require_cmd docker
load_env_file "prod.compose.env"

if [[ -z "${API_IMAGE:-}" || -z "${WEB_IMAGE:-}" ]]; then
  echo "Error: API_IMAGE and WEB_IMAGE must be set in config/prod.compose.env(.example)." >&2
  exit 1
fi

docker build \
  -f "${ROOT_DIR}/backend/hdrop-server/Dockerfile" \
  -t "${API_IMAGE}" \
  "${ROOT_DIR}/backend"

docker build \
  -f "${ROOT_DIR}/frontend/web/Dockerfile" \
  --build-arg "NEXT_PUBLIC_APP_NAME=${NEXT_PUBLIC_APP_NAME:-hdrop}" \
  --build-arg "NEXT_PUBLIC_WEB_BASE_URL=${NEXT_PUBLIC_WEB_BASE_URL:-https://hdrop.example.com}" \
  --build-arg "NEXT_PUBLIC_API_BASE_URL=${NEXT_PUBLIC_API_BASE_URL:-https://api.hdrop.example.com}" \
  --build-arg "NEXT_PUBLIC_PBKDF2_ITERATIONS=${NEXT_PUBLIC_PBKDF2_ITERATIONS:-600000}" \
  --build-arg "NEXT_PUBLIC_PASSWORD_BYTES=${NEXT_PUBLIC_PASSWORD_BYTES:-32}" \
  --build-arg "NEXT_PUBLIC_CHALLENGE_BYTES=${NEXT_PUBLIC_CHALLENGE_BYTES:-32}" \
  -t "${WEB_IMAGE}" \
  "${ROOT_DIR}/frontend/web"

echo "Built images: ${API_IMAGE}, ${WEB_IMAGE}"
