# hdrop frontend

Next.js + React, with encryption and file transfers handled in your browser.
Run `npm run dev` for the local dev server or `npm run build` to export static
files to `out/`. The production Docker image serves those files with Nginx on
port 80. Both setups use the Rust API.

## Development

From this directory:

```sh
npm ci
cp .env.example .env.local
npm run dev
```

Open `http://localhost:3000`. Start the Rust API separately at
`http://localhost:8080` and allow the frontend origin in its CORS configuration.

## Configuration and hosting

[`NEXT_PUBLIC_*` settings](./.env.example) are public and compiled into JavaScript.
Set them in `.env.local` or Docker `--build-arg` values, then rebuild. Changing env
vars on a running container won't update the frontend. Don't put secrets here.
Leave URL settings unset to use the browser origin for share links and `/v1/*` requests.
A separate API origin needs matching CORS configuration.

Keep the [Nginx routing rules](./infra/nginx/nginx.conf) if you use another static host:
`/` serves `index.html`, `/privacy` serves `privacy.html`, and access-token paths
(5 to 64 lowercase hex characters) serve `download.html`. Passwords remain in URL
fragments. Caddy routes `/v1/*` and `/status` to the API.

See [deployment and CI images](../README.md#production-environment) and the
[security model](../docs/security.md). Upload and download clients must use the
same PBKDF2 iteration count, including for existing files.

## Checks

From this directory:

```sh
npm test -- --runInBand
npm run lint
npm run build
```

For runtime and Caddy checks, from the repository root:

```sh
docker build -t hdrop-web:test frontend
bash frontend/scripts/test-static.sh hdrop-web:test
```

For browser tests, install Chromium with `npx playwright install chromium`, then
run `npm run test:static` from this directory against a running static frontend.
Set `STATIC_BASE_URL` if you're using a URL other than `http://127.0.0.1:8080`.
The API is mocked. See [CI](../.github/workflows/web.yml) for container setup.
