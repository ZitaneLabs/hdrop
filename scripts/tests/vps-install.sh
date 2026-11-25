#!/usr/bin/env bash
# Run on Linux with util-linux's script(1) to exercise the installer's first-run TTY.
set -euo pipefail

scripts=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
work=$(mktemp -d)
trap 'rm -rf -- "$work"' EXIT
mkdir -p "$work/repo/scripts" "$work/bin"
cp "$scripts"/{vps-install,vps-docker}.sh "$work/repo/scripts/"
cd "$work/repo"
export PATH="$work/bin:$PATH" VPS_TEST_LOG="$work/calls"
touch "$VPS_TEST_LOG"
ln -s /bin/false "$work/bin/sudo"
ln -s /bin/false "$work/bin/apt-get"

cat > "$work/bin/docker" <<'MOCK'
#!/usr/bin/env bash
set -eu
echo "$*" >> "$VPS_TEST_LOG"
case "$1" in
    context) echo "${TEST_DOCKER_ENDPOINT:-unix:///var/run/docker.sock}" ;;
    info)
        if [[ ${2:-} == --format ]]; then
            if [[ ${ROOTLESS:-} == true ]]; then echo '["name=rootless"]'; else echo '[]'; fi
        fi ;;
    buildx|build) ;;
    stop) [[ ${2:-} == stale-proxy ]] ;;
    run)
        if [[ $* == *' --publish '* ]]; then
            [[ ${ROOTLESS_PUBLISH:-} == allowed ]]
            exit
        fi
        for arg in "$@"; do
            if [[ $arg == *:/output ]]; then
                printf '%s\n' HDROP_PUBLIC_URL=https://files.example POSTGRES_PASSWORD=test-secret > "${arg%:/output}/.env"
                exit 0
            fi
        done
        exit 1 ;;
    compose)
        case "${*:2}" in
            'version --short') echo 2.27.1 ;;
            *'exec -T api printenv CORS_ORIGIN') sed -n 's/^HDROP_PUBLIC_URL=//p' .env ;;
            *'exec -T api printenv HDROP_IPV6') sed -n 's/^HDROP_IPV6=//p' .env ;;
            *'ps --orphans=false --format '*) printf '%s\n' "${UNHEALTHY_ID:-}" ;;
            *'pull --ignore-buildable --policy always') [[ ${FAIL_REFRESH:-} != pull ]] ;;
            *'build --pull') [[ ${FAIL_REFRESH:-} != build ]] ;;
            *'config --quiet'|*' build'|*'up -d --no-build --wait --wait-timeout 300'|*'up -d --no-build --pull never --wait --wait-timeout 300') ;;
            *) exit 1 ;;
        esac ;;
    *) exit 1 ;;
esac
MOCK
cat > "$work/bin/curl" <<'MOCK'
#!/usr/bin/env bash
echo "curl $*" >> "$VPS_TEST_LOG"
[[ ${!#} != "${FAIL_URL:-}" ]] || exit 7
case "${!#}" in
    https://files.example/status) echo OK ;;
    https://files.example/) echo "${HOME_CODE:-200}" ;;
    *) exit 1 ;;
esac
MOCK
cat > "$work/bin/cat" <<'MOCK'
#!/usr/bin/env bash
if [[ ${1:-} == /proc/sys/net/ipv4/ip_unprivileged_port_start ]]; then
    echo "${FIRST_UNPRIVILEGED_PORT:-1024}"
else
    exec /bin/cat "$@"
fi
MOCK
chmod +x "$work/bin/"{docker,curl,cat}

installer=(bash scripts/vps-install.sh)
"${installer[@]}" --help > "$work/output"
grep -q -- '--start' "$work/output"
grep -q -- '--update' "$work/output"
if "${installer[@]}" --unknown 2>/dev/null; then exit 1; fi
if "${installer[@]}" --start extra 2>/dev/null; then exit 1; fi
if "${installer[@]}" --update --start 2>/dev/null; then exit 1; fi
if "${installer[@]}" --update > "$work/output" 2>&1; then exit 1; fi
grep -q 'Update requires an existing .env' "$work/output"
if "${installer[@]}" </dev/null 2>/dev/null; then exit 1; fi
[[ ! -s $VPS_TEST_LOG && ! -e .env ]]

export ROOTLESS=true TEST_DOCKER_ENDPOINT=unix:///run/user/1000/docker.sock
if script -qec 'bash scripts/vps-install.sh --start' /dev/null </dev/null > "$work/output"; then exit 1; fi
grep -q 'CAP_NET_BIND_SERVICE' "$work/output"
if grep -Eq '^(buildx build|stop) | up | pull ' "$VPS_TEST_LOG"; then exit 1; fi
[[ ! -e .env ]]
: > "$VPS_TEST_LOG"
printf '\n' | script -qec 'bash scripts/vps-install.sh' /dev/null > "$work/output"
grep -q '^buildx build --load --network host ' "$VPS_TEST_LOG"
grep -q '^run .* --user 0:0 ' "$VPS_TEST_LOG"
if grep -q ' --publish ' "$VPS_TEST_LOG"; then exit 1; fi
grep -Fxq HDROP_IPV6=true .env
grep -Fxq HDROP_BUILD_NETWORK=host .env
grep -Fxq POSTGRES_PASSWORD=test-secret .env
[[ $(stat -c %a .env) == 600 ]]
if grep -Eq ' up |^curl ' "$VPS_TEST_LOG"; then exit 1; fi
cp .env "$work/original.env"

: > "$VPS_TEST_LOG"
if FIRST_UNPRIVILEGED_PORT=81 "${installer[@]}" --update > "$work/output" 2>&1; then exit 1; fi
grep -q 'net.ipv4.ip_unprivileged_port_start' "$work/output"
if grep -Eq '^stop | (build|up|pull) ' "$VPS_TEST_LOG"; then exit 1; fi
: > "$VPS_TEST_LOG"
FIRST_UNPRIVILEGED_PORT=80 "${installer[@]}" --start > "$work/output"
grep -q 'hdrop is ready' "$work/output"
if grep -q ' --publish ' "$VPS_TEST_LOG"; then exit 1; fi
: > "$VPS_TEST_LOG"
ROOTLESS_PUBLISH=allowed "${installer[@]}" --update > "$work/output"
grep -q 'hdrop is ready' "$work/output"
probe_image=$(sed -n 's/^run --rm --publish 127\.0\.0\.1:1-79:8080 --entrypoint \/bin\/true //p' "$VPS_TEST_LOG")
[[ -n $probe_image ]]
grep -Fxq "    image: $probe_image" <(tr -d '\r' < "$scripts/../docker-compose.yml")
unset ROOTLESS TEST_DOCKER_ENDPOINT

: > "$VPS_TEST_LOG"
"${installer[@]}" --start > "$work/output"
grep -q '^hdrop is ready at https://files.example' "$work/output"
grep -q 'up .*--wait' "$VPS_TEST_LOG"
grep -Eq '^curl .* -6 .*https://files.example/status$' "$VPS_TEST_LOG"
grep -Eq '^curl .* -6 .*https://files.example/$' "$VPS_TEST_LOG"
if grep -Eq '^(buildx build|run) ' "$VPS_TEST_LOG"; then exit 1; fi
if grep -q '^stop ' "$VPS_TEST_LOG"; then exit 1; fi
UNHEALTHY_ID=stale-proxy "${installer[@]}" --start > "$work/output"
grep -Fxq 'stop stale-proxy' "$VPS_TEST_LOG"

for path in /status /; do
    if FAIL_URL="https://files.example$path" "${installer[@]}" --start > "$work/output" 2>&1; then exit 1; fi
    grep -q 'public check failed' "$work/output"
done
if HOME_CODE=302 "${installer[@]}" --start > "$work/output" 2>&1; then exit 1; fi

for step in pull build; do
    : > "$VPS_TEST_LOG"
    if FAIL_REFRESH=$step UNHEALTHY_ID=stale-proxy "${installer[@]}" --update > "$work/output" 2>&1; then exit 1; fi
    grep -q ' pull --ignore-buildable --policy always$' "$VPS_TEST_LOG"
    if [[ $step == build ]]; then grep -q ' build --pull$' "$VPS_TEST_LOG"; fi
    if grep -Eq '^stop | up |^curl ' "$VPS_TEST_LOG"; then exit 1; fi
done

: > "$VPS_TEST_LOG"
UNHEALTHY_ID=stale-proxy "${installer[@]}" --update > "$work/output"
grep -Fxq 'stop stale-proxy' "$VPS_TEST_LOG"
grep -q ' up -d --no-build --pull never --wait --wait-timeout 300$' "$VPS_TEST_LOG"
grep -Eq '^curl .* -6 .*https://files.example/status$' "$VPS_TEST_LOG"
grep -Eq '^curl .* -6 .*https://files.example/$' "$VPS_TEST_LOG"
grep -q '^hdrop is ready at https://files.example' "$work/output"
if grep -Eq '^(buildx build|run) ' "$VPS_TEST_LOG"; then exit 1; fi
cmp "$work/original.env" .env
echo 'VPS installer regression checks passed.'
