# Operations

This repository supports two deployment tracks:

- Single VPS: all services on one host (`web`, `api`, `postgres`, local storage, `caddy`)
- Cloud-native: existing `local`, `staging`, and `prod` workflows

## Single VPS (Domain + TLS)

This path is intended for self-hosters running on one VPS.

1. Clone the repository on your VPS.
2. Generate config interactively:

```bash
make vps-install
```

3. Start services:

```bash
make vps-up
```

Note: on a first-ever Postgres bootstrap, logs may show `received fast shutdown request`
before `PostgreSQL init process complete; ready for start up.` This is expected for the
official Postgres image initialization sequence.

4. Optional smoke check:

```bash
make vps-smoke
```

5. Optional monitoring profile:

```bash
make vps-up -- --with-monitoring
```

6. Logs and shutdown:

```bash
make vps-logs
make vps-down
```

7. Rebuild and roll forward from current checkout:

```bash
make vps-upstall
```

## Single VPS (IP Bootstrap)

Use this when domain/DNS is not ready yet.

1. Run `make vps-install` and choose IP bootstrap mode.
2. Start with `make vps-up`.
3. Access via `http://<server-ip>` and API via `http://<server-ip>/api`.
4. When domain is ready, edit `config/vps.compose.env`:
   - `SITE_ADDRESS=<your-domain>`
   - `NEXT_PUBLIC_WEB_BASE_URL=https://<your-domain>`
   - `NEXT_PUBLIC_API_BASE_URL=https://<your-domain>/api`
   - `CORS_ORIGIN=https://<your-domain>`
   - `ACME_EMAIL=<you@example.com>`
5. Run `make vps-upstall` to switch to domain + TLS.

## Cloud-native Local

```bash
cp config/local.compose.env.example config/local.compose.env
cp config/local.api.env.example config/local.api.env
cp config/local.web.env.example config/local.web.env
make local-up
make db-migrate
make local-logs
make local-down
```

## Cloud-native Staging (local/CI simulation)

```bash
cp config/staging.compose.env.example config/staging.compose.env
cp config/staging.api.env.example config/staging.api.env
cp config/staging.web.env.example config/staging.web.env
make staging-up
make staging-smoke
make staging-down
```

## Cloud-native Image Build/Publish

```bash
cp config/prod.compose.env.example config/prod.compose.env
cp config/prod.api.env.example config/prod.api.env
cp config/prod.web.env.example config/prod.web.env
make deploy-build
make deploy-publish
make deploy-release
```

## CI Helper Commands

```bash
make ci-api
make ci-web
```
