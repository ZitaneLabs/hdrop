#!/usr/bin/env bash
set -euo pipefail

source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/common.sh"

require_cmd docker
if ! docker compose version >/dev/null 2>&1; then
  echo "Error: docker compose plugin is required." >&2
  exit 1
fi

start_after=0
force_overwrite=0
for arg in "$@"; do
  case "$arg" in
    --start)
      start_after=1
      ;;
    --force)
      force_overwrite=1
      ;;
    *)
      echo "Usage: $0 [--start] [--force]" >&2
      exit 1
      ;;
  esac
done

CONFIG_FILE="${ROOT_DIR}/config/vps.compose.env"

require_no_whitespace() {
  local label="$1"
  local value="$2"
  if [[ "$value" =~ [[:space:]] ]]; then
    echo "Error: ${label} must not contain whitespace." >&2
    exit 1
  fi
}

sanitize_host_input() {
  local host="$1"
  host="${host#http://}"
  host="${host#https://}"
  host="${host%%/*}"
  echo "$host"
}

is_ipv6_literal() {
  local host="$1"
  local colons
  host="${host#[}"
  host="${host%]}"
  colons="${host//[^:]}"
  [[ "${#colons}" -ge 2 ]]
}

normalize_url_host() {
  local host="$1"

  if is_ipv6_literal "$host"; then
    host="${host#[}"
    host="${host%]}"
    echo "[${host}]"
    return
  fi

  if [[ "$host" == *:* ]]; then
    echo "Error: host '${host}' appears to include a port. Use host/IP only." >&2
    exit 1
  fi

  echo "$host"
}

prompt_with_default() {
  local prompt="$1"
  local default_value="$2"
  local value
  read -r -p "${prompt} [${default_value}]: " value
  if [[ -z "$value" ]]; then
    echo "$default_value"
  else
    echo "$value"
  fi
}

prompt_required() {
  local prompt="$1"
  local value
  while true; do
    read -r -p "${prompt}: " value
    if [[ -n "$value" ]]; then
      echo "$value"
      return
    fi
    echo "Value is required."
  done
}

generate_password() {
  if command -v openssl >/dev/null 2>&1; then
    openssl rand -hex 16
  else
    tr -dc 'A-Za-z0-9' </dev/urandom | head -c 32
  fi
}

if [[ -f "${CONFIG_FILE}" && "${force_overwrite}" -ne 1 ]]; then
  echo "Found existing config/vps.compose.env"
  read -r -p "Reuse it? [Y/n]: " reuse
  reuse="${reuse:-Y}"
  case "$reuse" in
    [Yy]*)
      if [[ "$start_after" -eq 1 ]]; then
        "${ROOT_DIR}/scripts/vps-up.sh"
      else
        echo "Using existing config. Run: make vps-up"
      fi
      exit 0
      ;;
    *)
      ;;
  esac
fi

echo "Select setup mode:"
echo "1) Domain mode (automatic TLS via Caddy)"
echo "2) IP bootstrap mode (HTTP only)"
mode="$(prompt_with_default "Mode number" "1")"
if [[ "${mode}" != "1" && "${mode}" != "2" ]]; then
  echo "Error: mode must be 1 (domain) or 2 (IP bootstrap)." >&2
  exit 1
fi

ipv6_mode_answer="$(prompt_with_default "Enable IPv6-only compatibility mode? (y/N)" "N")"
ipv6_mode_answer="${ipv6_mode_answer,,}"
case "${ipv6_mode_answer}" in
  y|yes)
    vps_ipv6_only=1
    vps_build_network="host"
    ;;
  n|no|"")
    vps_ipv6_only=0
    vps_build_network="default"
    ;;
  *)
    echo "Error: please answer y/yes or n/no for IPv6-only compatibility mode." >&2
    exit 1
    ;;
esac

if [[ "${vps_ipv6_only}" -eq 1 ]]; then
  docker_ipv6="$(docker info --format '{{.IPv6}}' 2>/dev/null || true)"
  if [[ "${docker_ipv6}" != "true" ]]; then
    echo "Warning: Docker daemon reports IPv6='${docker_ipv6:-unknown}'."
    echo "The generated config will enable host-network builds for better IPv6-only compatibility."
  fi
fi

app_name="$(prompt_with_default "App display name" "hdrop")"
storage_dir="$(prompt_with_default "Container storage path" "/var/lib/hdrop/storage")"
local_storage_limit_mb="$(prompt_with_default "Local storage limit MB (empty=unlimited)" "")"
single_file_limit_mb="$(prompt_with_default "Single file upload limit MB" "500")"
cache_strategy="$(prompt_with_default "Cache strategy (memory|disk|hybrid)" "memory")"
cache_memory_limit_mb="$(prompt_with_default "Cache memory limit MB" "2000")"
cache_disk_limit_mb="$(prompt_with_default "Cache disk limit MB" "2000")"
cache_dir="$(prompt_with_default "Cache directory" "/tmp/hdrop-cache")"

db_user="$(prompt_with_default "Postgres user" "hdrop")"
default_db_password="$(generate_password)"
read -r -p "Postgres password [generated]: " db_password
if [[ -z "${db_password}" ]]; then
  db_password="${default_db_password}"
fi
db_name="$(prompt_with_default "Postgres database" "hdrop")"

require_no_whitespace "App display name" "${app_name}"
require_no_whitespace "Container storage path" "${storage_dir}"
require_no_whitespace "Cache strategy" "${cache_strategy}"
require_no_whitespace "Cache directory" "${cache_dir}"
require_no_whitespace "Postgres user" "${db_user}"
require_no_whitespace "Postgres database" "${db_name}"

domain=""
server_ip=""
server_url_host=""
site_address=""
acme_email=""
web_base_url=""
api_base_url=""
cors_origin=""
smoke_web_url=""
smoke_api_status_url=""

if [[ "${mode}" == "1" ]]; then
  domain="$(prompt_required "Domain (example.com)")"
  domain="$(sanitize_host_input "${domain}")"
  require_no_whitespace "Domain" "${domain}"

  if [[ ! "${domain}" =~ ^[A-Za-z0-9.-]+$ ]]; then
    echo "Error: invalid domain '${domain}'." >&2
    exit 1
  fi

  acme_email="$(prompt_required "ACME email")"
  if [[ ! "${acme_email}" =~ ^[^[:space:]@]+@[^[:space:]@]+\.[^[:space:]@]+$ ]]; then
    echo "Error: invalid email '${acme_email}'." >&2
    exit 1
  fi

  site_address="${domain}"
  web_base_url="https://${domain}"
  api_base_url="https://${domain}/api"
  cors_origin="https://${domain}"
  smoke_web_url="${web_base_url}"
  smoke_api_status_url="${api_base_url}/status"
else
  server_ip="$(prompt_required "Server IP or hostname")"
  server_ip="$(sanitize_host_input "${server_ip}")"
  require_no_whitespace "Server IP or hostname" "${server_ip}"
  server_url_host="$(normalize_url_host "${server_ip}")"
  site_address=":80"
  acme_email="admin@example.invalid"
  web_base_url="http://${server_url_host}"
  api_base_url="http://${server_url_host}/api"
  cors_origin="http://${server_url_host}"
  smoke_web_url="${web_base_url}"
  smoke_api_status_url="${api_base_url}/status"
fi

database_url="postgres://${db_user}:${db_password}@postgres:5432/${db_name}"

cat >"${CONFIG_FILE}" <<EOF2
# Generated by scripts/vps-install.sh
WEB_IMAGE=hdrop-web:vps
API_IMAGE=hdrop-api:vps
CADDY_IMAGE=caddy:2-alpine
HTTP_PORT=80
HTTPS_PORT=443
PROMETHEUS_PORT=9090
VPS_IPV6_ONLY=${vps_ipv6_only}
VPS_BUILD_NETWORK=${vps_build_network}

SITE_ADDRESS=${site_address}
ACME_EMAIL=${acme_email}

NEXT_PUBLIC_APP_NAME=${app_name}
NEXT_PUBLIC_WEB_BASE_URL=${web_base_url}
NEXT_PUBLIC_API_BASE_URL=${api_base_url}
NEXT_PUBLIC_PBKDF2_ITERATIONS=600000
NEXT_PUBLIC_PASSWORD_BYTES=32
NEXT_PUBLIC_CHALLENGE_BYTES=32

HDROP_PORT=80
HDROP_PROMETHEUS_PORT=3001
CORS_ORIGIN=${cors_origin}
SINGLE_FILE_LIMIT_MB=${single_file_limit_mb}

STORAGE_PROVIDER=local
LOCAL_STORAGE_DIR=${storage_dir}
LOCAL_STORAGE_LIMIT_MB=${local_storage_limit_mb}

POSTGRES_USER=${db_user}
POSTGRES_PASSWORD=${db_password}
POSTGRES_DB=${db_name}
DATABASE_URL=${database_url}

CACHE_STRATEGY=${cache_strategy}
CACHE_MEMORY_LIMIT_MB=${cache_memory_limit_mb}
CACHE_DISK_LIMIT_MB=${cache_disk_limit_mb}
CACHE_DIR=${cache_dir}

SMOKE_WEB_URL=${smoke_web_url}
SMOKE_API_STATUS_URL=${smoke_api_status_url}
EOF2

echo "Wrote config/vps.compose.env"

if [[ "$start_after" -eq 1 ]]; then
  "${ROOT_DIR}/scripts/vps-up.sh"
else
  echo "Next step: make vps-up"
fi
