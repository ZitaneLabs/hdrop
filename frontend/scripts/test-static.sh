#!/usr/bin/env bash
# Requires frontend dependencies, Playwright Chromium, and built web images.
set -euo pipefail

repo_root=$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)
web="hdrop-frontend-test-$$"
runtime=static

cleanup() {
    status=$?
    if (( status != 0 )); then
        docker logs "$web" >&2 2>/dev/null || true
    fi
    docker rm -f "$web" >/dev/null 2>&1 || true
}
trap cleanup EXIT

for image in "${1:-hdrop-web:test}" "${2:-}"; do
    [[ -n "$image" ]] || continue
    docker run -d --name "$web" \
        --read-only --cap-drop ALL --security-opt no-new-privileges:true \
        --tmpfs /tmp:rw,noexec,nosuid,nodev --sysctl net.ipv4.ip_unprivileged_port_start=0 \
        --pids-limit 256 --publish 127.0.0.1::80 "$image" >/dev/null
    base="http://$(docker port "$web" 80/tcp)"
    curl --silent --show-error --fail --retry 30 --retry-all-errors \
        --retry-delay 1 --retry-max-time 30 --max-time 2 "$base/" >/dev/null
    docker exec "$web" sh -c 'test "$(id -u)" -ne 0'
    if [[ "$runtime" == static ]]; then
        docker exec "$web" nginx -t
        docker exec "$web" sh -c '! command -v node && ! command -v npm && test ! -e /app/server.js'
    else
        docker exec "$web" node --version
    fi
    STATIC_BASE_URL="$base" FRONTEND_RUNTIME="$runtime" npm --prefix "$repo_root/frontend" run test:static
    docker rm -f "$web" >/dev/null
    runtime=nextjs
done
