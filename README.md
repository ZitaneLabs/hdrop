<h1><img src="./hdrop-web-next/public/logo512.png" width="32" height="32" />&nbsp;hdrop</h1>

**Simple, self-hosted encrypted file transfer.**

## Features

- Easy to self-host
- Modern UI/UX
- End-to-end encrypted (E2EE)
- Automatic file deletion
- Supports S3 and simple disk storage
- No user accounts
- Metrics included

## Docs

| Name                   | Link                              |
| ---------------------- | --------------------------------- |
| Deployment             | [deployment.md](./docs/deployment.md) |
| Security               | [security.md](./docs/security.md) |
| API Spec v1 (OpenAPI)  | [api_v1.yml](./docs/api_v1.yml)     |
| Full Feature List      | [features.md](./docs/features.md) |
| Changelog              | [changelog.md](./docs/changelog.md) |


## Development

Run `docker-compose up` to start the stack.

| Service    | Host      | Port |
| ---------- | --------- | ---- |
| Web        | localhost | 3000 |
| API        | localhost | 8080 |
| LocalStack | localhost | 4566 |
| Postgres   | localhost | 5432 |

The frontend has hot reloading enabled and will react to changes.

## Deployment
> You can deploy to most popular cloud providers with ease.

### Frontend
> The frontend is a static site that can be hosted anywhere.

**Environment Variables**<br>
No matter where you deploy, you'll need to configure the following environment variables:

| Name                            | Example Value           | Description                             |
| ------------------------------- | ----------------------- | --------------------------------------- |
| `NEXT_PUBLIC_APP_NAME`          | `hdrop`                 | Display name of the instance            |
| `NEXT_PUBLIC_WEB_BASE_URL`      | `http://localhost:3000` | The URL where the frontend is hosted    |
| `NEXT_PUBLIC_API_BASE_URL`      | `http://localhost:8080` | The URL where the API is hosted         |
| `NEXT_PUBLIC_PBKDF2_ITERATIONS` | `600000`                | Number of PBKDF2 passes on the password |
| `NEXT_PUBLIC_PASSWORD_BYTES`    | `32`                    | Number of bytes in the password         |
| `NEXT_PUBLIC_CHALLENGE_BYTES`   | `32`                    | Number of bytes in the challenge        |

#### Docker (Recommended)
> We provide a preconfigured `Dockerfile` with nginx.

Simply deploy the `Dockerfile` in `hdrop-web-next` to your favorite cloud provider.

#### Vercel

1. Fork the repo
2. Create a new Vercel project and link it to your fork
3. Set the root directory to `hdrop-web-next`
4. Deploy and enjoy!

#### Other Providers

Deploying to other providers should be similarly simple. Just be sure to configure the environment variables listed above.

### Backend

**Prerequisites**<br>

- Postgres database
- S3-compatible object store or available disks<br>
  Examples of compatible providers include:
  - Amazon S3
  - Cloudflare R2
  - Backblaze B2

**Environment Variables**<br>
No matter where you deploy, you'll need to configure the following environment variables:

| Name                     | Example                        | Description                | Required | Default        |
| ------------------------ | ------------------------------ | -------------------------- | -------- | -------------- |
| `PORT`                   | `2000`                         | Listener port              | No       | `8080`         |
| `DATABASE_URL`           | `postgres://user:pass@host/db` | Postgres Connection String | Yes      | -              |
| `STORAGE_PROVIDER`       | `s3` / `local`                 | Storage provider           | Yes      | -              |
| `S3_REGION`              | `example-region-1`             | S3 region                  | Yes[^1]  | -              |
| `S3_ENDPOINT`            | `https://foo.s3.example.org`   | S3 endpoint                | Yes[^1]  | -              |
| `S3_ACCESS_KEY_ID`       | `AKIAIOSFODNN7EXAMPLE`         | S3 Access Key ID           | Yes[^1]  | -              |
| `S3_SECRET_ACCESS_KEY`   | `bPxRfiCYEXAMPLEKEY`           | S3 Secret Access Key       | Yes[^1]  | -              |
| `S3_PUBLIC_URL`          | `https://storage.example.org`  | S3 Public Bucket URL       | Yes[^1]  | -              |
| `S3_BUCKET_NAME`         | `hdrop`                        | S3 Bucket Name             | Yes[^1]  | -              |
| `LOCAL_STORAGE_DIR`      | `/var/hdrop-storage`           | Local storage path         | No[^2]   | `./files`      |
| `LOCAL_STORAGE_LIMIT_MB` | `250`                          | Local storage limit in MB  | No[^2]   | -              |
| `CORS_ORIGIN`            | `*` / `example.com,foo.bar`    | Allowed CORS origins       | No       | `*`            |
| `CACHE_STRATEGY`         | `memory` / `disk` / `hybrid`   | Cache strategy             | No       | `memory`       |
| `CACHE_MEMORY_LIMIT_MB`  | `250`                          | Cache memory limit in MB   | No[^3]   | -              |
| `CACHE_DISK_LIMIT_MB`    | `250`                          | Cache disk limit in MB     | No[^4]   | -              |
| `CACHE_DIR`              | `/var/cache`                   | Cache storage directory    | No[^4]   | `./file_cache` |
| `SINGLE_FILE_LIMIT_MB`   | `250`                          | File upload limit[^5]      | No[^6]   | `100`          |

[^1]: Only required if `STORAGE_PROVIDER` is set to `s3`
[^2]: Not required, but recommended if `STORAGE_PROVIDER` is set to `local`
[^3]: Not required, but recommended if `CACHE_STRATEGY` is set to `memory` or `hybrid`
[^4]: Not required, but recommended if `CACHE_STRATEGY` is set to `disk` or `hybrid`
[^5]: The file upload limit is the total body limit per request
[^6]: This is only `100MB` by default, so you should probably set this to something reasonable

#### Docker (Recommended)
We provide a preconfigured `Dockerfile` with pm2.

Simply deploy the `Dockerfile` in `hdrop-server` to your favorite cloud provider.
