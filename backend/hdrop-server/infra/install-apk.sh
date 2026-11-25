#!/bin/sh
set -eu

if ! apk add --no-cache "$@"; then
    # Some IPv6 routes cannot reach the default CDN. Keep the same signed release repositories.
    sed 's/dl-cdn\.alpinelinux\.org/mirrors.edge.kernel.org/g' /etc/apk/repositories >/tmp/hdrop-apk-repositories
    apk add --no-cache --repositories-file /tmp/hdrop-apk-repositories "$@"
    rm /tmp/hdrop-apk-repositories
fi
