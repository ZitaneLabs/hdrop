.PHONY: local-up local-down local-logs db-migrate api-dev \
	staging-up staging-down staging-smoke \
	deploy-build deploy-publish deploy-release \
	ci-api ci-web

local-up:
	./scripts/local-up.sh

local-down:
	./scripts/local-down.sh

local-logs:
	./scripts/local-logs.sh

db-migrate:
	./scripts/local-migrate.sh

api-dev:
	./scripts/local-api-dev.sh

staging-up:
	./scripts/staging-up.sh

staging-down:
	./scripts/staging-down.sh

staging-smoke:
	./scripts/staging-smoke.sh

deploy-build:
	./scripts/deploy-build-images.sh

deploy-publish:
	./scripts/deploy-publish-images.sh

deploy-release:
	./scripts/deploy-release.sh

ci-api:
	cd backend && cargo +nightly fmt --all --check
	cd backend && cargo clippy --all-targets --all-features

ci-web:
	cd frontend/web && npm ci
	cd frontend/web && npm run test -- --runInBand
	cd frontend/web && npm run lint
	cd frontend/web && ./check_licenses.sh
