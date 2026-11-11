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

`npm run build` exports static files to `out/`. For the Next.js server, keep
`HDROP_WEB_RUNTIME=nextjs` set for both `npm run build` and `npm start`.
Docker uses the `static` target by default, or use `--target nextjs`.

[`NEXT_PUBLIC_*` settings](./.env.example) are public and compiled into JavaScript.
Set them in `.env.local` or Docker `--build-arg` values, then rebuild. Changing env
vars on a running container won't update the frontend. Don't put secrets here.
Leave URL settings unset to use the browser origin for share links and `/v1/*` requests.
A separate API origin needs matching CORS configuration.

Keep the [Nginx routing rules](./infra/nginx/nginx.conf) if you use another static host:
`/` serves `index.html`, `/privacy` serves `privacy.html`, and access-token paths
(5 to 64 lowercase hex characters) serve `download.html`. Passwords remain in URL
fragments. Caddy routes `/v1/*` and `/status` to the API.

See [deployment](../README.md#production-environment) and the
[security model](../docs/security.md). Upload and download clients must use the
same PBKDF2 iteration count, including for existing files.

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
