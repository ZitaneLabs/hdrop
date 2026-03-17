# Environment Reference

This file lists the environment variables used by hdrop runtime components.

## Backend (`hdrop-server`)

| Variable | Required | Description |
| --- | --- | --- |
| `HDROP_PORT` | No | API listen port in container. |
| `PROMETHEUS_PORT` | No | Metrics endpoint port in container. |
| `CORS_ORIGIN` | No | Allowed CORS origin(s), `*` or comma-separated list. |
| `SINGLE_FILE_LIMIT_MB` | No | Max upload size in MB. |
| `STORAGE_PROVIDER` | Yes | `s3` or `local`. |
| `DATABASE_URL` | Yes | PostgreSQL connection URL. |
| `CACHE_STRATEGY` | No | `memory`, `disk`, or `hybrid`. |
| `CACHE_MEMORY_LIMIT_MB` | No | Memory cache size limit in MB. |
| `CACHE_DISK_LIMIT_MB` | No | Disk cache size limit in MB. |
| `CACHE_DIR` | No | Cache directory for disk/hybrid modes. |
| `S3_REGION` | Yes for `s3` | S3 region identifier. |
| `S3_ENDPOINT` | Yes for `s3` | S3 endpoint URL. |
| `S3_ACCESS_KEY_ID` | Yes for `s3` | S3 access key ID. |
| `S3_SECRET_ACCESS_KEY` | Yes for `s3` | S3 secret key. |
| `S3_BUCKET_NAME` | Yes for `s3` | S3 bucket name. |
| `S3_PUBLIC_URL` | Yes for `s3` | Public base URL used for direct download links. |
| `LOCAL_STORAGE_DIR` | Yes for `local` | Filesystem path for local storage provider. |
| `LOCAL_STORAGE_LIMIT_MB` | No for `local` | Local storage size limit in MB; empty means unlimited. |

Source: `backend/hdrop-shared/src/env.rs`.

## Frontend (`frontend/web`)

| Variable | Required | Description |
| --- | --- | --- |
| `NEXT_PUBLIC_APP_NAME` | No | UI app name shown in header. |
| `NEXT_PUBLIC_WEB_BASE_URL` | Yes | Public base URL for download link creation. |
| `NEXT_PUBLIC_API_BASE_URL` | Yes | Public API base URL used by the browser client. |
| `NEXT_PUBLIC_PBKDF2_ITERATIONS` | No | PBKDF2 iterations used client-side. |
| `NEXT_PUBLIC_PASSWORD_BYTES` | No | Generated password byte length. |
| `NEXT_PUBLIC_CHALLENGE_BYTES` | No | Generated challenge byte length. |

Sources:
- `frontend/web/src/api/ApiClient.ts`
- `frontend/web/src/components/Header.tsx`
- `frontend/web/src/crypto/Pbkdf2.ts`
- `frontend/web/src/crypto/CryptoHelper.ts`
