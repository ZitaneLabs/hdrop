#!/usr/bin/env sh

set -euo pipefail

LICENSE_ALLOWLIST="$(cat license_allowlist.txt | tr '\n' ';')"
LICENSE_EXCLUDE_PACKAGES="$(cat license_exclude_packages.txt| tr '\n' ';')"
npx license-checker-rseidelsohn --onlyAllow "$LICENSE_ALLOWLIST" --excludePackages "$LICENSE_EXCLUDE_PACKAGES"
