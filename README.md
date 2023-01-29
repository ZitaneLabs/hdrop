# hdrop
> Simple, self-hosted encrypted file transfer.

## Docs

| Name                   | Link                              |
| ---------------------- | --------------------------------- |
| Security               | [security.md](./docs/security.md) |
| API Spec v1 (OpenAPI)  | [api_v1.yml](./docs/api_v1.yml)   |
| API Spec v1 (Markdown) | [api_v1.md](./docs/api_v1.md)     |

## Development

Run `docker-compose up` to start the stack.

| Service    | Host      | Port |
| ---------- | --------- | ---- |
| Web        | localhost | 3000 |
| API        | localhost | 8080 |
| LocalStack | localhost | 4566 |

The frontend has hot reloading enabled and will react to changes.

## Deployment
> You can deploy to most popular cloud providers with ease.

### Frontend
> The frontend is a static site that can be hosted anywhere.

No matter where you deploy, you'll need to configure the following environment variables:

| Name                     | Example Value             | Description                          |
| ------------------------ | ------------------------- | ------------------------------------ |
| `REACT_APP_BASE_URL`     | `https://example.org`     | The URL where the frontend is hosted |
| `REACT_APP_API_ENDPOINT` | `https://api.example.org` | The URL where the API is hosted      |

#### Docker (Recommended)
> We provide a preconfigured `Dockerfile` with nginx.

Simply deploy the `Dockerfile` in `hdrop-web` to your favorite cloud provider.

#### Vercel

1. Fork the repo
2. Create a new Vercel project and link it to your fork
3. Set the root directory to `hdrop-web`
4. Deploy and enjoy!

#### Other Providers

Deploying to other providers should be similarly simple. Just be sure to configure the environment variables listed above.

### Backend

No matter where you deploy, you'll need to configure the following environment variables:

| Name                   | Example Value                 | Description          |
| ---------------------- | ----------------------------- | -------------------- |
| `S3_REGION`            | `example-region-1`            | S3 region            |
| `S3_ENDPOINT`          | `https://foo.s3.example.org`  | S3 endpoint          |
| `S3_ACCESS_KEY_ID`     | ``                            | S3 Access Key ID     |
| `S3_SECRET_ACCESS_KEY` | ``                            | S3 Secret Access Key |
| `S3_PUBLIC_URL`        | `https://storage.example.org` | S3 Public Bucket URL |
| `S3_BUCKET_NAME`       | `hdrop`                       | S3 Bucket Name       |
| `CORS_ORIGIN`          | '*'                           | Allowed CORS Origin  |

#### Docker (Recommended)
We provide a preconfigured `Dockerfile` with pm2.

Simply deploy the `Dockerfile` in `hdrop-server` to your favorite cloud provider.
