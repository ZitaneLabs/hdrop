.PHONY: local-up local-down local-logs db-migrate api-dev \
	vps-install vps-ipv6 vps-up vps-down vps-logs vps-smoke vps-upstall \
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

vps-install:
	./scripts/vps-install.sh

vps-ipv6:
	./scripts/vps-ipv6.sh

vps-up:
	./scripts/vps-up.sh

vps-down:
	./scripts/vps-down.sh

vps-logs:
	./scripts/vps-logs.sh

vps-smoke:
	./scripts/vps-smoke.sh

vps-upstall:
	./scripts/vps-upstall.sh

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
