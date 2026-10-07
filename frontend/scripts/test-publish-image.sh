#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")"

# Mock both commands so these checks never contact GitHub or a registry.
docker() { printf '%s\n' "$*"; }
git() {
    printf '%s\trefs/heads/production\n' "$TEST_PRODUCTION_SHA"
    return "${TEST_GIT_STATUS:-0}"
}
export -f docker git
export GH_REPOSITORY=Example/Hdrop GITHUB_SHA=current TEST_PRODUCTION_SHA=current
image=ghcr.io/example/hdrop-web
sha_commands=$(printf 'tag hdrop-web:test %s:sha-current\npush %s:sha-current' "$image" "$image")
production_commands=$(printf 'tag hdrop-web:test %s:production\npush %s:production' "$image" "$image")

export GITHUB_REF=refs/heads/development
[[ $(bash publish-image.sh) == "$sha_commands" ]]

export GITHUB_REF=refs/heads/production
[[ $(bash publish-image.sh) == "$sha_commands"$'\n'"$production_commands" ]]

# An old run finishing or being rerun after a newer commit must only publish its SHA.
export TEST_PRODUCTION_SHA=newer
[[ $(bash publish-image.sh) == "$sha_commands"$'\nSkipping production tag for stale commit current' ]]

# Even matching output must not promote when the remote lookup fails.
export TEST_PRODUCTION_SHA=current TEST_GIT_STATUS=1
if output=$(bash publish-image.sh); then
    echo 'Expected the failed remote lookup to stop publication' >&2
    exit 1
fi
[[ "$output" == "$sha_commands" ]]

echo 'Image publication checks passed.'
