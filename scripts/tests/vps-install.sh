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
    context) echo unix:///var/run/docker.sock ;;
    info|buildx|build) ;;
    run)
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
            *'config --quiet'|*'up -d --build --wait --wait-timeout 300') ;;
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
chmod +x "$work/bin/"{docker,curl}

installer=(bash scripts/vps-install.sh)
"${installer[@]}" --help > "$work/output"
grep -q -- '--start' "$work/output"
if "${installer[@]}" --unknown 2>/dev/null; then exit 1; fi
if "${installer[@]}" --start extra 2>/dev/null; then exit 1; fi
if "${installer[@]}" </dev/null 2>/dev/null; then exit 1; fi
[[ ! -s $VPS_TEST_LOG && ! -e .env ]]

printf '\n' | script -qec 'bash scripts/vps-install.sh' /dev/null > "$work/output"
grep -q '^buildx build --load --network host ' "$VPS_TEST_LOG"
grep -Fxq HDROP_IPV6=true .env
grep -Fxq HDROP_BUILD_NETWORK=host .env
grep -Fxq POSTGRES_PASSWORD=test-secret .env
[[ $(stat -c %a .env) == 600 ]]
if grep -Eq ' up |^curl ' "$VPS_TEST_LOG"; then exit 1; fi
cp .env "$work/original.env"

: > "$VPS_TEST_LOG"
"${installer[@]}" --start > "$work/output"
grep -q '^hdrop is ready at https://files.example' "$work/output"
grep -q 'up .*--wait' "$VPS_TEST_LOG"
grep -Eq '^curl .* -6 .*https://files.example/status$' "$VPS_TEST_LOG"
grep -Eq '^curl .* -6 .*https://files.example/$' "$VPS_TEST_LOG"
if grep -Eq '^(buildx build|run) ' "$VPS_TEST_LOG"; then exit 1; fi

for path in /status /; do
    if FAIL_URL="https://files.example$path" "${installer[@]}" --start > "$work/output" 2>&1; then exit 1; fi
    grep -q 'public check failed' "$work/output"
done
if HOME_CODE=302 "${installer[@]}" --start > "$work/output" 2>&1; then exit 1; fi
cmp "$work/original.env" .env
echo 'VPS installer regression checks passed.'
