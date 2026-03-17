#!/usr/bin/env bash
set -euo pipefail

source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/common.sh"

profile_args=()
if [[ "${1:-}" == "--with-monitoring" ]]; then
  profile_args+=(--profile monitoring)
  shift
fi

compose_for_env vps "${profile_args[@]}" up --build --wait -d "$@"
