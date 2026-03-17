# Operations

This repository uses a flat operations layout:

- `infra/`: compose overlays + infra config files
- `config/`: environment templates (`*.env.example`)
- `scripts/`: runnable workflow scripts
- `Makefile`: canonical entrypoint for local/staging/deploy workflows

## Local

1. Optional: create runtime env files from templates:

```bash
cp config/local.compose.env.example config/local.compose.env
cp config/local.api.env.example config/local.api.env
cp config/local.web.env.example config/local.web.env
```

2. Start stack:

```bash
make local-up
```

3. Apply database migrations:

```bash
make db-migrate
```

4. Stream logs:

```bash
make local-logs
```

5. Stop stack:

```bash
make local-down
```

## Run API Outside Docker (local dev)

```bash
make api-dev
```

This starts dependency services, runs migrations, then launches the backend binary from `backend/`.

## Staging (local/CI simulation)

1. Optional: create runtime env files from templates:

```bash
cp config/staging.compose.env.example config/staging.compose.env
cp config/staging.api.env.example config/staging.api.env
cp config/staging.web.env.example config/staging.web.env
```

2. Bring staging overlay up/down:

```bash
make staging-up
make staging-down
```

3. Run smoke checks:

```bash
make staging-smoke
```

## Deploy Images

1. Optional: create runtime env files from templates:

```bash
cp config/prod.compose.env.example config/prod.compose.env
cp config/prod.api.env.example config/prod.api.env
cp config/prod.web.env.example config/prod.web.env
```

2. Build, publish, or full release flow:

```bash
make deploy-build
make deploy-publish
make deploy-release
```

## CI Helper Commands

```bash
make ci-api
make ci-web
```
