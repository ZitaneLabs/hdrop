#!/usr/bin/env bash
# Run after: docker build -t hdrop-web:test frontend
set -euo pipefail

repo_root=$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)
image=${1:-hdrop-web:test}
test_dir=$(mktemp -d)
prefix="hdrop-static-test-$$"
web="$prefix-web"
api="$prefix-api"
caddy="$prefix-caddy"
network="$prefix-network"

cleanup() {
    status=$?
    if (( status != 0 )); then
        for container in "$web" "$api" "$caddy"; do
            docker logs "$container" >&2 2>/dev/null || true
        done
    fi
    docker rm -f "$caddy" "$api" "$web" >/dev/null 2>&1 || true
    docker network rm "$network" >/dev/null 2>&1 || true
    rm -rf "$test_dir"
}
trap cleanup EXIT

fail() {
    echo "$*" >&2
    exit 1
}

wait_ready() {
    for _ in {1..60}; do
        if curl --silent --fail --max-time 1 "$base/" >/dev/null; then
            return
        fi
        sleep 0.5
    done
    fail "Server did not start: $base"
}

request() {
    local expected=$1 path=$2
    shift 2
    local actual
    actual=$(curl --silent --show-error --max-time 10 --dump-header "$test_dir/headers" \
        --output "$test_dir/body" --write-out '%{http_code}' "$@" "$base$path")
    [[ "$actual" == "$expected" ]] || fail "$path: expected HTTP $expected, got $actual"
}

header() {
    grep -Eiq "^$1: $2" "$test_dir/headers" || fail "Missing header $1: $2"
}

check_page_redirects() {
    for path in /privacy /download; do
        request 301 "$path/?source=slash-test"
        header Location "$path[?]source=slash-test"
        request 200 "$path/" --location
        header Content-Type text/html
        header Cache-Control no-cache
    done
    for path in /not-a-page/ /%5Cexample.org/ /%2F%5Cexample.org/; do
        request 404 "$path"
        if grep -Eiq '^Location:' "$test_dir/headers"; then
            fail "$path: unknown paths must not redirect"
        fi
    done
}

docker network create "$network" >/dev/null
docker run -d --name "$web" --network "$network" --network-alias web \
    --read-only --cap-drop ALL --security-opt no-new-privileges:true \
    --tmpfs /tmp:rw,noexec,nosuid,nodev --sysctl net.ipv4.ip_unprivileged_port_start=0 \
    --pids-limit 256 --publish 127.0.0.1::80 "$image" >/dev/null
base="http://$(docker port "$web" 80/tcp)"
wait_ready
docker exec "$web" nginx -t
docker exec "$web" sh -c 'test "$(id -u)" -ne 0 && ! command -v node && ! command -v npm && test ! -e /app/server.js'

for path in / /privacy /favicon.ico; do
    request 200 "$path"
    header Cache-Control no-cache
done
check_page_redirects

request 200 /download
cp "$test_dir/body" "$test_dir/download"
for path in /abcde /abcde/ /0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef; do
    request 200 "$path"
    header Content-Type text/html
    header Cache-Control no-cache
    cmp -s "$test_dir/body" "$test_dir/download" || fail "$path: download shell differs"
    if grep -Eiq '^Location:' "$test_dir/headers"; then
        fail "$path: token routes must not redirect"
    fi
done

for path in /missing.js /_next/static/missing.js /abcde/nested /abc /not-a-token /status /v1/files; do
    request 404 "$path"
    header Cache-Control no-cache
    if grep -qi immutable "$test_dir/headers"; then
        fail "$path: missing resources must not be cached immutably"
    fi
done

request 200 /
asset=$(grep -oE '/_next/static/[^" ]+\.js' "$test_dir/body" | sed -n '1p')
[[ -n "$asset" ]] || fail 'No built JavaScript asset found'
request 200 "$asset"
header Cache-Control 'public, max-age=31536000, immutable'
request 200 / --compressed
header Content-Encoding gzip
header Vary Accept-Encoding

# Exercise the production Caddyfile against a tiny deterministic backend fixture.
cat > "$test_dir/api.conf" <<'EOF'
worker_processes 1;
pid /tmp/nginx.pid;
error_log /dev/stderr warn;
events { worker_connections 32; }
http {
    access_log /dev/stdout;
    client_body_temp_path /tmp/client_temp;
    proxy_temp_path /tmp/proxy_temp;
    fastcgi_temp_path /tmp/fastcgi_temp;
    uwsgi_temp_path /tmp/uwsgi_temp;
    scgi_temp_path /tmp/scgi_temp;
    server {
        listen 3000;
        location = /status { return 200 "backend-status"; }
        location /v1/ { return 200 "backend-api"; }
        location / { return 404; }
    }
}
EOF
docker run -d --name "$api" --network "$network" --network-alias api \
    --read-only --cap-drop ALL --security-opt no-new-privileges:true \
    --tmpfs /tmp:rw,noexec,nosuid,nodev \
    --mount "type=bind,src=$test_dir/api.conf,dst=/etc/nginx/nginx.conf,readonly" \
    "$image" >/dev/null
docker run -d --name "$caddy" --network "$network" \
    --read-only --cap-drop ALL --cap-add NET_BIND_SERVICE \
    --security-opt no-new-privileges:true --tmpfs /tmp:rw,noexec,nosuid,nodev \
    --tmpfs /data --tmpfs /config --publish 127.0.0.1::80 \
    --env HDROP_SITE_ADDRESS=:80 --env HDROP_MAX_UPLOAD_SIZE=500MB \
    --mount "type=bind,src=$repo_root/caddy/Caddyfile,dst=/etc/caddy/Caddyfile,readonly" \
    caddy:2-alpine >/dev/null
base="http://$(docker port "$caddy" 80/tcp)"
wait_ready

for path in / /privacy /abcde /abcde/; do
    request 200 "$path"
    header X-Content-Type-Options nosniff
    header X-Frame-Options DENY
    header Strict-Transport-Security 'max-age=31536000'
    header Content-Security-Policy "base-uri 'self'"
    if grep -Eiq '^Server:' "$test_dir/headers"; then
        fail 'Caddy must remove the Server header'
    fi
done
check_page_redirects
request 200 /status
[[ $(cat "$test_dir/body") == backend-status ]] || fail '/status did not reach the backend'
request 200 /v1/files/abcde/challenge
[[ $(cat "$test_dir/body") == backend-api ]] || fail '/v1/* did not reach the backend'
request 404 /_next/static/missing.js
header Cache-Control no-cache
request 200 "$asset"
header Cache-Control 'public, max-age=31536000, immutable'
request 200 / --compressed
header Content-Encoding 'gzip|zstd'

echo 'Static frontend runtime, routes, caching, compression and Caddy checks passed.'
