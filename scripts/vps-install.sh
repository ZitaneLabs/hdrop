#!/usr/bin/env bash
set -euo pipefail

usage() {
    echo 'Usage: ./scripts/vps-install.sh [--start | --update]'
    echo 'Configure a single VPS. Add --start to build, start and check its public HTTPS endpoint.'
    echo 'Use --update with an existing .env to pull service images, refresh build bases and deploy this checkout.'
}

action=${1:-}
if (( $# > 1 )); then usage >&2; exit 2; fi
case "$action" in
    --start|--update|'') ;;
    -h|--help) usage; exit 0 ;;
    *) usage >&2; exit 2 ;;
esac

cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.."
source scripts/vps-docker.sh

if [[ $action == --update && ! -f .env ]]; then
    echo 'Update requires an existing .env. Run ./scripts/vps-install.sh --start for first-time setup.' >&2
    exit 1
fi
if [[ ! -f .env && ( ! -t 0 || ! -t 1 ) ]]; then
    echo 'First-time setup needs an interactive terminal. Run this command in your SSH session.' >&2
    exit 1
fi

ensure_docker
umask 077
setup_dir=$(mktemp -d "$PWD/.hdrop-setup.XXXXXXXX")
cleanup() {
    rm -f -- "$setup_dir/.env" "$setup_dir/network.env"
    rmdir -- "$setup_dir"
}
trap cleanup EXIT

# Apply VPS networking to older .env files too. Explicit settings in .env take precedence.
printf '%s\n' HDROP_IPV6=true HDROP_BUILD_NETWORK=host > "$setup_dir/network.env"
compose=("${docker_cmd[@]}" compose -f docker-compose.yml --env-file "$setup_dir/network.env")
if [[ -f .env ]]; then
    echo 'Reusing .env and the existing service volumes.'
else
    echo 'Use a public hostname pointing to this VPS. Allow inbound TCP 80 and 443 for HTTPS.'
    while true; do
        read -r -p 'Enable IPv6 VPS networking? [Y/n] ' answer
        case "${answer,,}" in
            ''|y|yes) ipv6=true; build_network=host; break ;;
            n|no) ipv6=false; build_network=default; break ;;
            *) echo 'Enter y or n.' ;;
        esac
    done

    echo 'Building the configuration wizard...'
    "${docker_cmd[@]}" buildx build --load --network "$build_network" -f backend/hdrop-env/Dockerfile \
        -t hdrop-env:vps backend
    wizard_user="$(id -u):$(id -g)"
    if [[ $("${docker_cmd[@]}" info --format '{{json .SecurityOptions}}') == *rootless* ]]; then
        wizard_user=0:0  # Root inside rootless Docker maps to the invoking user on the host.
    fi
    "${docker_cmd[@]}" run --rm -it --pull never --network none --user "$wizard_user" \
        -v "$setup_dir:/output" hdrop-env:vps --vps --output /output/.env
    printf '\nHDROP_IPV6=%s\nHDROP_BUILD_NETWORK=%s\n' "$ipv6" "$build_network" >> "$setup_dir/.env"
    "${compose[@]}" --env-file "$setup_dir/.env" config --quiet
    # Publish a complete config without replacing credentials from another concurrent setup.
    ln -- "$setup_dir/.env" .env
    echo 'Saved the configuration to .env.'
fi

compose+=(--env-file .env)
"${compose[@]}" config --quiet
if [[ -z $action ]]; then
    echo 'To start hdrop, run ./scripts/vps-install.sh --start'
    exit 0
fi

up_pull=()
if [[ $action == --update ]]; then
    "${compose[@]}" pull --ignore-buildable --policy always
    "${compose[@]}" build --pull
    up_pull=(--pull never)
else
    "${compose[@]}" build
fi
# Compose --wait fails immediately on an old unhealthy status. Start those
# containers again in dependency order after the new images are available.
unhealthy=$("${compose[@]}" ps --orphans=false --format '{{if eq .Health "unhealthy"}}{{.ID}}{{end}}' | sed '/^$/d')
if [[ -n $unhealthy ]]; then
    echo 'Restarting previously unhealthy hdrop containers.'
    mapfile -t containers <<< "$unhealthy"
    "${docker_cmd[@]}" stop "${containers[@]}"
fi
if ! "${compose[@]}" up -d --no-build "${up_pull[@]}" --wait --wait-timeout 300; then
    echo 'Startup failed. Check docker compose ps -a and docker compose logs, then rerun --start.' >&2
    exit 1
fi
# Read only the deployed public URL and network setting. Compose sets CORS_ORIGIN to HDROP_PUBLIC_URL.
url=$("${compose[@]}" exec -T api printenv CORS_ORIGIN)
ipv6=$("${compose[@]}" exec -T api printenv HDROP_IPV6)
url=${url%/}
curl_args=(--noproxy '*' --proto '=https' --fail --silent --show-error --connect-timeout 5 --max-time 10)
case "${ipv6,,}" in true|1|t|yes|on) curl_args+=(-6) ;; esac
echo "Waiting for trusted HTTPS at $url ..."
if ! status=$(curl "${curl_args[@]}" --retry 18 --retry-all-errors --retry-delay 4 --retry-max-time 90 "$url/status") ||
   [[ $status != OK ]] ||
   ! home_code=$(curl "${curl_args[@]}" --output /dev/null --write-out '%{http_code}' "$url/") ||
   [[ $home_code != 200 ]]; then
    echo 'Services started, but the public check failed. Check DNS, TCP 80/443 and docker compose logs caddy, then rerun --start.' >&2
    exit 1
fi
echo "hdrop is ready at $url"
