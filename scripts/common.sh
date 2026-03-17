#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"

require_cmd() {
  local cmd="$1"
  if ! command -v "$cmd" >/dev/null 2>&1; then
    echo "Error: required command '$cmd' is not installed." >&2
    exit 1
  fi
}

resolve_config_file() {
  local name="$1"
  local file="${ROOT_DIR}/config/${name}"
  local example="${file}.example"

  if [[ -f "$file" ]]; then
    echo "$file"
    return
  fi

  if [[ -f "$example" ]]; then
    echo "$example"
    return
  fi

  echo "Error: missing config file '${file}' (or '${example}')." >&2
  exit 1
}

load_env_file() {
  local name="$1"
  local file
  file="$(resolve_config_file "$name")"

  set -a
  # shellcheck disable=SC1090
  source "$file"
  set +a
}

compose_for_env() {
  local env="$1"
  shift

  local env_file
  env_file="$(resolve_config_file "${env}.compose.env")"

  require_cmd docker
  docker compose \
    -f "${ROOT_DIR}/infra/compose.base.yml" \
    -f "${ROOT_DIR}/infra/compose.${env}.yml" \
    --env-file "$env_file" \
    "$@"
}
