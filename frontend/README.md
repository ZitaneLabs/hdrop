# hdrop frontend

Next.js + React, with encryption and file transfers handled in your browser.
Run `npm run dev` locally. Production supports static Nginx (the default) or a
standalone Next.js server. Both hydrate in the browser and use the Rust API.

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

For static hosting, use `frontend` as the project root, run `npm ci` and
`npm run build`, then publish `out/`. For the Next.js server, keep
`HDROP_WEB_RUNTIME=nextjs` set for both `npm run build` and `npm start`.
Docker uses the `static` target by default, or use `--target nextjs`.

[`NEXT_PUBLIC_*` settings](./.env.example) are public and compiled into JavaScript.
Set them in `.env.local` or Docker `--build-arg` values, then rebuild. Changing env
vars on a running container won't update the frontend. Don't put secrets here.
Leave URL settings unset to use the browser origin for share links and `/v1/*` requests.
For separate hosts, build with `NEXT_PUBLIC_WEB_BASE_URL` set to the frontend's
HTTPS origin and `NEXT_PUBLIC_API_BASE_URL` to the API's HTTPS origin.
Set the API's `CORS_ORIGIN` to the full frontend origin, such as `https://drop.example.com`.
`NEXT_PUBLIC_APP_NAME` sets the navigation bar name (default `hdrop`).

Keep the [Nginx routing rules](./infra/nginx/nginx.conf) if you use another static host:
`/` serves `index.html`, `/privacy` serves `privacy.html`, and access-token paths
(5 to 64 lowercase hex characters) serve `download.html`. Passwords remain in URL
fragments. Caddy routes `/v1/*` and `/status` to the API.

See the [installation guide](../docs/installation.md) and the
[cryptography guide](../docs/crypto.md). Upload and download clients must use
the same PBKDF2 iteration count, including for existing files.

## Checks

From this directory:

```sh
npm test -- --runInBand
npm run lint
npm run build
```

Install Chromium with `npx playwright install chromium` from this directory.
Then run the container and browser checks from the repository root:

```sh
docker build -t hdrop-web:test frontend
bash frontend/scripts/test-static.sh hdrop-web:test
```

To check both runtimes, build another image with `--target nextjs` and pass it as
the script's second argument. Checks use a mocked API and run without Caddy.

To verify an installed VPS from a workstation with IPv6 access, run the following
from this directory with Chromium installed. It uploads a small file through the
public frontend, decrypts it in a second browser tab, and deletes it.

```sh
HDROP_TEST_URL=https://files.example.com npx playwright test e2e/vps.spec.ts
```
