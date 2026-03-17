#!/usr/bin/env bash
set -euo pipefail

source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/common.sh"

require_cmd curl
load_env_file "vps.compose.env"

web_url="${SMOKE_WEB_URL:-${NEXT_PUBLIC_WEB_BASE_URL:-}}"
api_status_url="${SMOKE_API_STATUS_URL:-${NEXT_PUBLIC_API_BASE_URL:-}/status}"

if [[ -z "${web_url}" || -z "${api_status_url}" ]]; then
  echo "Error: SMOKE_WEB_URL and SMOKE_API_STATUS_URL (or NEXT_PUBLIC_* URLs) must be set." >&2
  exit 1
fi

curl_args=(--fail --silent --show-error)
if [[ "${SMOKE_INSECURE:-0}" == "1" ]]; then
  curl_args+=(--insecure)
fi

curl "${curl_args[@]}" "${web_url}" >/dev/null
curl "${curl_args[@]}" "${api_status_url}" >/dev/null

echo "VPS smoke check passed."
