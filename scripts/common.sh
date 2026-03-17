#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"

is_truthy() {
  local value="${1,,}"
  case "$value" in
    1|true|yes|on) return 0 ;;
    *) return 1 ;;
  esac
}

read_env_var_from_file() {
  local file="$1"
  local key="$2"
  local line
  local value

  line="$(grep -E "^[[:space:]]*${key}[[:space:]]*=" "$file" | tail -n 1 || true)"
  [[ -n "$line" ]] || return 1

  value="${line#*=}"
  value="${value%%#*}"
  value="${value#"${value%%[![:space:]]*}"}"
  value="${value%"${value##*[![:space:]]}"}"
  value="${value%\"}"
  value="${value#\"}"
  value="${value%\'}"
  value="${value#\'}"

  printf '%s\n' "$value"
}

vps_extra_compose_files() {
  local env_file="$1"
  local ipv6_only
  local overlay_file

  ipv6_only="$(read_env_var_from_file "$env_file" "VPS_IPV6_ONLY" || true)"
  if ! is_truthy "$ipv6_only"; then
    return
  fi

  overlay_file="${ROOT_DIR}/infra/compose.vps.ipv6.yml"
  if [[ ! -f "$overlay_file" ]]; then
    echo "Error: missing IPv6 compose overlay: ${overlay_file}" >&2
    exit 1
  fi

  printf '%s\n' "$overlay_file"
}

require_cmd() {
  local cmd="$1"
  if ! command -v "$cmd" >/dev/null 2>&1; then
    echo "Error: required command '$cmd' is not installed." >&2
    exit 1
  fi
}

has_docker_daemon_access() {
  docker info >/dev/null 2>&1
}

is_user_listed_in_docker_group() {
  local group_line
  local members

  group_line="$(getent group docker 2>/dev/null || true)"
  [[ -n "$group_line" ]] || return 1

  members="${group_line##*:}"
  [[ ",${members}," == *",${USER},"* ]]
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
  local compose_files
  local compose_args
  local command_line
  local arg

  env_file="$(resolve_config_file "${env}.compose.env")"

  require_cmd docker
  compose_files=(
    "${ROOT_DIR}/infra/compose.base.yml"
    "${ROOT_DIR}/infra/compose.${env}.yml"
  )

  if [[ "$env" == "vps" ]]; then
    while IFS= read -r extra_file; do
      compose_files+=("$extra_file")
    done < <(vps_extra_compose_files "$env_file")
  fi

  compose_args=()
  for arg in "${compose_files[@]}"; do
    compose_args+=(-f "$arg")
  done
  compose_args=(
    "${compose_args[@]}" \
    --env-file "$env_file" \
    "$@"
  )

  if has_docker_daemon_access; then
    docker compose "${compose_args[@]}"
    return
  fi

  # Fallback for shells that have not picked up group membership yet.
  if command -v sg >/dev/null 2>&1 && is_user_listed_in_docker_group; then
    command_line="docker compose"
    for arg in "${compose_args[@]}"; do
      command_line+=" $(printf '%q' "$arg")"
    done
    sg docker -c "$command_line"
    return
  fi

  cat <<'EOF' >&2
Error: cannot access Docker daemon.
Try one of:
1. Start a new shell session after adding your user to the `docker` group.
2. Run `sudo usermod -aG docker $USER` and log out/in.
3. Run this command through `sg docker -c '<command>'`.
EOF
  exit 1
}
