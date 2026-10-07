#!/usr/bin/env bash
# The workflow serializes image jobs per ref before running this script.
set -euo pipefail

image="ghcr.io/${GH_REPOSITORY,,}-web"
docker tag hdrop-web:test "$image:sha-$GITHUB_SHA"
docker push "$image:sha-$GITHUB_SHA"

if [[ "$GITHUB_REF" == refs/heads/production ]]; then
    production_sha=$(git ls-remote --exit-code origin refs/heads/production | cut -f1)
    if [[ "$GITHUB_SHA" == "$production_sha" ]]; then
        docker tag hdrop-web:test "$image:production"
        docker push "$image:production"
    else
        echo "Skipping production tag for stale commit $GITHUB_SHA"
    fi
fi
