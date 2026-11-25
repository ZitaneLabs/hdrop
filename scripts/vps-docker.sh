#!/usr/bin/env bash

# The installer sources this file, then uses "${docker_cmd[@]}" for every Docker command.
vps_package_installed() {
    [[ $(dpkg-query -W -f='${db:Status-Status}' "$1" 2>/dev/null) == installed ]]
}

vps_install_docker() (
    local -a elevate=("$@")
    local package tmp repo codename
    # shellcheck disable=SC1091
    . /etc/os-release || return 1
    case "$ID" in
        debian|ubuntu) ;;
        *) echo "Install Docker Engine with the Compose and Buildx plugins on $ID, then rerun the installer." >&2; return 1 ;;
    esac

    for package in docker.io docker-compose docker-compose-v2 docker-doc docker-buildx podman-docker containerd runc; do
        if vps_package_installed "$package"; then
            echo "Installed package $package conflicts with Docker's packages. Finish setting up your existing runtime before retrying." >&2
            return 1
        fi
    done

    "${elevate[@]}" apt-get update || return 1
    "${elevate[@]}" apt-get install -y --no-remove ca-certificates curl || return 1
    if [[ -z $(apt-cache madison docker-ce) ]]; then
        if [[ -e /etc/apt/sources.list.d/docker.sources ]]; then
            echo "The existing docker.sources does not provide docker-ce. Check the configured Docker repository." >&2
            return 1
        fi
        repo="https://download.docker.com/linux/$ID"
        codename=${UBUNTU_CODENAME:-${VERSION_CODENAME:?Missing release codename in /etc/os-release}}
        tmp=$(mktemp -d) || return 1
        trap 'rm -rf -- "$tmp"' EXIT
        curl -fsSL --retry 3 "$repo/gpg" -o "$tmp/docker.asc" || return 1
        cat > "$tmp/docker.sources" <<EOF
Types: deb
URIs: $repo
Suites: $codename
Components: stable
Architectures: $(dpkg --print-architecture)
Signed-By: /etc/apt/keyrings/docker.asc
EOF
        "${elevate[@]}" install -d -m 0755 /etc/apt/keyrings || return 1
        "${elevate[@]}" install -m 0644 "$tmp/docker.asc" /etc/apt/keyrings/docker.asc || return 1
        "${elevate[@]}" install -m 0644 "$tmp/docker.sources" /etc/apt/sources.list.d/docker.sources || return 1
        "${elevate[@]}" apt-get update || return 1
    fi
    "${elevate[@]}" apt-get install -y --no-remove docker-ce docker-ce-cli containerd.io docker-buildx-plugin docker-compose-plugin || return 1
    "${elevate[@]}" systemctl start docker
)

vps_compose_supported() {
    local version=${2#*:}  # Ignore the distribution's optional package epoch.
    [[ $version =~ ^v?[0-9]+\.[0-9]+\.[0-9]+ ]] || return 1
    printf '%s\n' "$1" "${version#v}" | sort -V -C
}

vps_apt_candidate() {
    LC_ALL=C apt-cache policy "$1" | awk '$1 == "Candidate:" && $2 != "(none)" {print $2}'
}

vps_ensure_plugins() {
    local minimum_compose=2.24.0 need_compose=0 need_buildx=0 package buildx
    local -a compose_packages install_packages=()
    vps_compose_supported "$minimum_compose" "$("${docker_cmd[@]}" compose version --short 2>/dev/null)" || need_compose=1
    "${docker_cmd[@]}" buildx version >/dev/null 2>&1 || need_buildx=1
    if (( !need_compose && !need_buildx )); then return 0; fi

    if vps_package_installed docker-ce-cli; then
        compose_packages=(docker-compose-plugin)
        buildx=docker-buildx-plugin
    elif vps_package_installed docker.io; then
        compose_packages=(docker-compose-v2 docker-compose)
        buildx=docker-buildx
    else
        echo "Install Docker Compose $minimum_compose or newer and the Buildx plugin for your Docker installation, then retry." >&2
        return 1
    fi
    "$@" apt-get update || return 1
    if (( need_compose )); then
        for package in "${compose_packages[@]}"; do
            if vps_compose_supported "$minimum_compose" "$(vps_apt_candidate "$package")"; then
                install_packages+=("$package")
                break
            fi
        done
        if (( ${#install_packages[@]} == 0 )); then
            echo "The configured apt repositories do not offer Docker Compose $minimum_compose or newer for this engine." >&2
            return 1
        fi
    fi
    if (( need_buildx )); then
        if [[ -z $(vps_apt_candidate "$buildx") ]]; then
            echo "The configured apt repositories do not offer the $buildx package for this engine." >&2
            return 1
        fi
        install_packages+=("$buildx")
    fi
    "$@" apt-get install -y --no-remove "${install_packages[@]}" || return 1
    if ! vps_compose_supported "$minimum_compose" "$("${docker_cmd[@]}" compose version --short)" || ! "${docker_cmd[@]}" buildx version >/dev/null; then
        echo "Docker could not load the required Compose and Buildx plugins after installation." >&2
        return 1
    fi
}

ensure_docker() {
    local endpoint
    local -a elevate=()
    if (( EUID != 0 )); then elevate=(sudo); fi
    docker_cmd=(docker)
    if ! command -v docker >/dev/null; then
        vps_install_docker "${elevate[@]}" || return 1
    fi
    # The wizard's bind mount and the public listeners must be on this VPS.
    endpoint=$(docker context inspect --format '{{.Endpoints.docker.Host}}') || return 1
    if [[ $endpoint != unix:///* ]]; then
        echo "Select a local Docker Unix socket to install hdrop on this VPS (current endpoint: $endpoint)." >&2
        return 1
    fi
    if ! docker info >/dev/null 2>&1; then
        case "$endpoint" in
            unix:///var/run/docker.sock|unix:///run/docker.sock) ;;
            *) echo "The selected Docker endpoint is unreachable: $endpoint" >&2; return 1 ;;
        esac
        # sudo has its own Docker context. Keep it on the same system socket.
        docker_cmd=("${elevate[@]}" docker --host "$endpoint")
        if ! "${docker_cmd[@]}" info >/dev/null 2>&1; then
            if [[ $(systemctl show docker.service --property=LoadState --value 2>/dev/null) == loaded ]]; then
                "${elevate[@]}" systemctl start docker || return 1
                if docker info >/dev/null 2>&1; then docker_cmd=(docker); fi
            fi
            "${docker_cmd[@]}" info >/dev/null || return 1
        fi
    fi
    vps_ensure_plugins "${elevate[@]}" || return 1
    if ! command -v curl >/dev/null; then
        if ! command -v apt-get >/dev/null; then
            echo "Install curl to verify hdrop's public HTTPS endpoint, then retry." >&2
            return 1
        fi
        "${elevate[@]}" apt-get update || return 1
        "${elevate[@]}" apt-get install -y --no-remove ca-certificates curl || return 1
    fi
}

vps_check_rootless_ports() {
    local first_unprivileged
    first_unprivileged=$(cat /proc/sys/net/ipv4/ip_unprivileged_port_start) || return 1
    if (( first_unprivileged <= 80 )); then return 0; fi

    # Exercise the selected daemon's port driver. Docker picks a free privileged
    # loopback port so an existing Caddy can keep serving on 80 and 443.
    if ! "${docker_cmd[@]}" run --rm --publish 127.0.0.1:1-79:8080 --entrypoint /bin/true caddy:2-alpine; then
        echo 'Rootless Docker could not complete the port check for host TCP 80 and 443.' >&2
        echo 'For a privileged-port error, set net.ipv4.ip_unprivileged_port_start to 80 or lower on the host.' >&2
        echo "With RootlessKit's built-in port driver, you can also grant CAP_NET_BIND_SERVICE to rootlesskit and restart the user Docker service." >&2
        echo 'See https://docs.docker.com/engine/security/rootless/tips/#exposing-privileged-ports' >&2
        return 1
    fi
}
