#!/usr/bin/env bash
set -euo pipefail

source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/common.sh"

# Cloudflare public IPv6 DNS resolvers
CLOUDFLARE_DNS_V6_PRIMARY="2606:4700:4700::1111"
CLOUDFLARE_DNS_V6_SECONDARY="2606:4700:4700::1001"
# Google public IPv6 DNS resolvers
GOOGLE_DNS_V6_PRIMARY="2001:4860:4860::8888"
GOOGLE_DNS_V6_SECONDARY="2001:4860:4860::8844"
# systemd-resolved local stub on the host
RESOLVED_STUB_DNS="127.0.0.53"

DOCKER_FIXED_CIDR_V6_DEFAULT="fd00:dead:beef::/48"
RESOLVED_DROPIN_FILE="/etc/systemd/resolved.conf.d/99-hdrop-ipv6-dns.conf"
DOCKER_DAEMON_FILE="/etc/docker/daemon.json"

dry_run=0
fixed_cidr_v6="${DOCKER_FIXED_CIDR_V6_DEFAULT}"

usage() {
  cat <<'EOF'
Usage: ./scripts/vps-ipv6.sh [--dry-run] [--fixed-cidr-v6 <cidr>]

Configure an Ubuntu VPS host for IPv6-only Docker + DNS operation:
- writes systemd-resolved DNS drop-in
- writes/merges /etc/docker/daemon.json with IPv6 options
  and points Docker DNS to the host systemd-resolved stub
- restarts systemd-resolved and docker
- performs basic host/docker DNS checks

Options:
  --dry-run                Print planned changes without applying.
  --fixed-cidr-v6 <cidr>  Docker fixed-cidr-v6 (default: fd00:dead:beef::/48).
  -h, --help               Show this help.
EOF
}

for arg in "$@"; do
  case "$arg" in
    --dry-run)
      dry_run=1
      ;;
    --fixed-cidr-v6)
      echo "Error: --fixed-cidr-v6 requires a value." >&2
      exit 1
      ;;
    --fixed-cidr-v6=*)
      fixed_cidr_v6="${arg#*=}"
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      # Handle split args: --fixed-cidr-v6 <cidr>
      if [[ "${prev_arg:-}" == "--fixed-cidr-v6" ]]; then
        fixed_cidr_v6="$arg"
        unset prev_arg
      else
        prev_arg="$arg"
        if [[ "$prev_arg" != "--fixed-cidr-v6" ]]; then
          echo "Error: unknown argument '$arg'." >&2
          usage >&2
          exit 1
        fi
      fi
      ;;
  esac
done

if [[ "${prev_arg:-}" == "--fixed-cidr-v6" ]]; then
  echo "Error: --fixed-cidr-v6 requires a value." >&2
  exit 1
fi

require_cmd docker
require_cmd getent
require_cmd systemctl

if [[ "$dry_run" -eq 0 ]] && [[ "$EUID" -ne 0 ]] && ! command -v sudo >/dev/null 2>&1; then
  echo "Error: root privileges are required (run as root or install sudo)." >&2
  exit 1
fi

run_as_root() {
  if [[ "$dry_run" -eq 1 ]]; then
    printf '+ '
    printf '%q ' "$@"
    printf '\n'
    return
  fi

  if [[ "$EUID" -eq 0 ]]; then
    "$@"
  else
    sudo "$@"
  fi
}

read_root_file() {
  local file="$1"
  if [[ "$EUID" -eq 0 ]]; then
    cat "$file"
  else
    sudo cat "$file"
  fi
}

write_root_file_from_tmp() {
  local src="$1"
  local dest="$2"

  run_as_root mkdir -p "$(dirname "$dest")"
  if [[ "$dry_run" -eq 1 ]]; then
    echo "--- ${dest} (planned)"
    cat "$src"
    return
  fi

  if [[ "$EUID" -eq 0 ]]; then
    cat "$src" > "$dest"
  else
    sudo tee "$dest" < "$src" >/dev/null
  fi

  run_as_root chmod 644 "$dest"
}

backup_root_file_if_exists() {
  local file="$1"
  local backup
  if [[ "$dry_run" -eq 1 || ! -f "$file" ]]; then
    return
  fi

  backup="${file}.bak.$(date +%Y%m%d%H%M%S)"
  run_as_root cp "$file" "$backup"
  echo "Backup created: ${backup}"
}

has_resolved_stub_listener() {
  if ! command -v ss >/dev/null 2>&1; then
    return 1
  fi
  ss -lntup 2>/dev/null | grep -F "127.0.0.53:53" >/dev/null
}

collect_resolved_nameservers() {
  local source_file="/run/systemd/resolve/resolv.conf"
  if [[ ! -f "$source_file" ]]; then
    source_file="/etc/resolv.conf"
  fi
  awk '/^nameserver[[:space:]]+/ {print $2}' "$source_file" | awk '!seen[$0]++'
}

resolved_tmp="$(mktemp)"
docker_tmp="$(mktemp)"
trap 'rm -f "$resolved_tmp" "$docker_tmp"' EXIT

cat >"$resolved_tmp" <<EOF
# Managed by hdrop scripts/vps-ipv6.sh
[Resolve]
# Cloudflare IPv6 DNS
DNS=${CLOUDFLARE_DNS_V6_PRIMARY} ${CLOUDFLARE_DNS_V6_SECONDARY}
# Google IPv6 fallback DNS
FallbackDNS=${GOOGLE_DNS_V6_PRIMARY} ${GOOGLE_DNS_V6_SECONDARY}
# Keep local DNS stub listener active for Docker upstream forwarding.
DNSStubListener=yes
EOF

echo "Preparing systemd-resolved drop-in:"
echo "  - ${CLOUDFLARE_DNS_V6_PRIMARY} (Cloudflare)"
echo "  - ${CLOUDFLARE_DNS_V6_SECONDARY} (Cloudflare)"
echo "  - ${GOOGLE_DNS_V6_PRIMARY} (Google)"
echo "  - ${GOOGLE_DNS_V6_SECONDARY} (Google)"

echo "Applying host IPv6/DNS configuration..."
backup_root_file_if_exists "$RESOLVED_DROPIN_FILE"
write_root_file_from_tmp "$resolved_tmp" "$RESOLVED_DROPIN_FILE"

if [[ "$dry_run" -eq 0 ]]; then
  run_as_root systemctl restart systemd-resolved
fi

docker_dns_targets=()
docker_dns_source=""
if [[ "$dry_run" -eq 1 ]]; then
  docker_dns_targets=("$RESOLVED_STUB_DNS")
  docker_dns_source="dry-run default (systemd-resolved stub)"
elif has_resolved_stub_listener; then
  docker_dns_targets=("$RESOLVED_STUB_DNS")
  docker_dns_source="systemd-resolved stub listener"
else
  mapfile -t docker_dns_targets < <(collect_resolved_nameservers)
  if [[ "${#docker_dns_targets[@]}" -eq 0 ]]; then
    docker_dns_targets=(
      "$CLOUDFLARE_DNS_V6_PRIMARY"
      "$CLOUDFLARE_DNS_V6_SECONDARY"
      "$GOOGLE_DNS_V6_PRIMARY"
      "$GOOGLE_DNS_V6_SECONDARY"
    )
    docker_dns_source="public IPv6 fallback list"
  else
    docker_dns_source="/run/systemd/resolve/resolv.conf nameservers"
  fi
fi

echo "Docker DNS upstream targets (${docker_dns_source}): ${docker_dns_targets[*]}"

docker_dns_json="["
for dns_server in "${docker_dns_targets[@]}"; do
  docker_dns_json+="\"${dns_server}\","
done
docker_dns_json="${docker_dns_json%,}]"

if [[ "$dry_run" -eq 1 ]]; then
  cat >"$docker_tmp" <<EOF
{
  "ipv6": true,
  "fixed-cidr-v6": "${fixed_cidr_v6}",
  "ip6tables": true,
  "dns": ${docker_dns_json}
}
EOF
elif [[ -s "$DOCKER_DAEMON_FILE" ]]; then
  require_cmd jq
  read_root_file "$DOCKER_DAEMON_FILE" | jq \
    --arg cidr "$fixed_cidr_v6" \
    --argjson dns "${docker_dns_json}" \
    '
      .ipv6 = true
      | .ip6tables = true
      | ."fixed-cidr-v6" = $cidr
      | .dns = $dns
    ' >"$docker_tmp"
else
  cat >"$docker_tmp" <<EOF
{
  "ipv6": true,
  "fixed-cidr-v6": "${fixed_cidr_v6}",
  "ip6tables": true,
  "dns": ${docker_dns_json}
}
EOF
fi

backup_root_file_if_exists "$DOCKER_DAEMON_FILE"
write_root_file_from_tmp "$docker_tmp" "$DOCKER_DAEMON_FILE"

if [[ "$dry_run" -eq 0 ]]; then
  run_as_root systemctl restart docker
fi

if [[ -f "${ROOT_DIR}/config/vps.compose.env" ]]; then
  if grep -q '^VPS_IPV6_ONLY=' "${ROOT_DIR}/config/vps.compose.env"; then
    sed -i 's/^VPS_IPV6_ONLY=.*/VPS_IPV6_ONLY=1/' "${ROOT_DIR}/config/vps.compose.env"
  else
    echo 'VPS_IPV6_ONLY=1' >> "${ROOT_DIR}/config/vps.compose.env"
  fi

  if grep -q '^VPS_BUILD_NETWORK=' "${ROOT_DIR}/config/vps.compose.env"; then
    sed -i 's/^VPS_BUILD_NETWORK=.*/VPS_BUILD_NETWORK=host/' "${ROOT_DIR}/config/vps.compose.env"
  else
    echo 'VPS_BUILD_NETWORK=host' >> "${ROOT_DIR}/config/vps.compose.env"
  fi

  echo "Updated config/vps.compose.env: VPS_IPV6_ONLY=1, VPS_BUILD_NETWORK=host"
fi

echo
echo "Verification:"
echo "Docker DNS upstream targets: ${docker_dns_targets[*]}"
docker_ipv6="$(docker_ipv6_status || true)"
echo "Docker IPv6=${docker_ipv6:-unknown}"
if command -v ss >/dev/null 2>&1; then
  ss -lntup 2>/dev/null | grep -F "127.0.0.53:53" >/dev/null && \
    echo "systemd-resolved stub listener: OK" || \
    echo "systemd-resolved stub listener: NOT FOUND"
fi
getent ahosts deb.debian.org | head -n 3 || true
if command -v curl >/dev/null 2>&1; then
  curl -6 --fail --silent --show-error --head https://deb.debian.org >/dev/null && \
    echo "curl -6 deb.debian.org: OK" || \
    echo "curl -6 deb.debian.org: FAILED"
fi

echo
if [[ "$dry_run" -eq 1 ]]; then
  echo "Dry run completed. Re-run without --dry-run to apply changes."
else
  echo "IPv6 host configuration applied."
  echo "Next steps:"
  echo "1. make vps-install  (or reuse existing config)"
  echo "2. make vps-up"
fi
