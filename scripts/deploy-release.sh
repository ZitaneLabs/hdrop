#!/usr/bin/env bash
set -euo pipefail

source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/common.sh"

"${ROOT_DIR}/scripts/deploy-build-images.sh"
"${ROOT_DIR}/scripts/deploy-publish-images.sh"
