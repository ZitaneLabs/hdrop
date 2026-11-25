# Installation

| Deployment | Guide |
| --- | --- |
| One Linux VPS with local files and PostgreSQL | [Single VPS](#single-vps) (quickest) |
| Droplet with Managed PostgreSQL and Spaces | [DigitalOcean managed services](#digitalocean-managed-services) |
| EC2 or another Docker host | [Other providers](#other-providers) |
| DigitalOcean Marketplace | [One-click install (planned)](#digitalocean-marketplace-planned) |

## Single VPS

Point your hostname (e.g. `drop.example.com`) at the VPS using AAAA for reachable IPv6, A for reachable IPv4, or both. Open inbound **TCP 80 and 443** on both provider and host firewalls. Leave both ports free for Caddy to obtain a certificate and serve HTTPS.

SSH in as your usual user. On Debian or Ubuntu the installer can set up Docker Engine, Buildx and Compose with sudo. Install Git to clone, or run the last command from your existing checkout.

```sh
git clone https://github.com/ZitaneLabs/hdrop.git
cd hdrop
./scripts/vps-install.sh --start
```

1. Accept IPv6 if your AAAA record works. Choose `n` for IPv4 only.
2. Enter the hostname without `https://`. Press **Enter** at the customization prompt for local storage, disk cache and bundled PostgreSQL.
3. Wait for `hdrop is ready`. The installer builds the `hdrop-env` wizard, saves a private `.env`, starts Compose and checks the homepage and `/status` over trusted HTTPS. Caddy routes `/v1/*` and `/status` to the API and all pages to the static Nginx frontend.

The first build needs outbound access to Docker Hub and package registries (about 15 minutes on our test VPS). If an IPv6-only host cannot reach GitHub, copy a `git archive` tarball over SSH and extract it in a user-owned folder. It preserves executable permissions and works without `.git`.

<details>
<summary>Existing Docker or manual Compose setup</summary>

Use local Linux Docker Engine, Buildx and Compose 2.24+. Engine 27+ allocates IPv6 subnets automatically. Older versions need an IPv6 address pool.

For rootless Docker to publish 80/443, set the host's `net.ipv4.ip_unprivileged_port_start` to 80 or lower. With RootlessKit's built-in port driver, you can instead grant `CAP_NET_BIND_SERVICE` to `rootlesskit` and restart the user Docker service. The installer checks port access before building. [Docker rootless ports](https://docs.docker.com/engine/security/rootless/tips/#exposing-privileged-ports)

Run `./scripts/vps-install.sh` without flags to generate and review `.env`, or use Rust and Cargo from the same checkout. For scripts, pass `--non-interactive` with your settings. Transfer `.env` privately.

```sh
(cd backend && cargo run --locked -p hdrop-env -- --vps --output ../.env)
```

| Network | Settings for a manual `.env` |
| --- | --- |
| IPv6 | `HDROP_IPV6=true`, `HDROP_BUILD_NETWORK=host` |
| IPv4 only | `HDROP_IPV6=false`, `HDROP_BUILD_NETWORK=default` |

An existing `.env` without these settings defaults to IPv6 under the installer. For the optional Next.js server, set `HDROP_WEB_RUNTIME=nextjs` before building. Validate and start from the checkout.

```sh
docker compose config --quiet
docker compose up -d --build --wait
```

</details>

### Verify an upload and download

Upload a small, recognizable text file over HTTPS. Copy its link and password, then open the link in a private window. Decrypt and download it, check the name and contents, and delete it from the original tab. To automate this, run the Chromium test from a workstation that can reach the site. [Browser test instructions](../frontend/README.md#checks)

## DigitalOcean managed services

The Droplet runs Caddy, the frontend, API and metrics. It can serve clients over IPv6 but needs IPv4 egress for IPv4-only provider endpoints.

### Prepare the database and Space

1. **Managed PostgreSQL.** Use the Droplet's region and VPC. Add the Droplet as a trusted source. Create a database for hdrop's metadata and a user that owns its schema for startup migrations. Copy the private hostname, port (often `25060`), database name and credentials from **Connection Details**. For Standard Edition, download the CA to `postgres-ca.crt` in the checkout. [Connections](https://docs.digitalocean.com/products/databases/postgresql/how-to/connect/) and [trusted sources and TLS](https://docs.digitalocean.com/products/databases/postgresql/how-to/secure/)
2. **Spaces.** Create a dedicated Standard Space for encrypted uploads. Restrict listing and require the API's Spaces key for writes and deletes. Anyone with an unsigned object URL can fetch the ciphertext. Decryption requires the share password. A private bucket needs a browser-accessible read gateway. [Encryption details](./crypto.md)
3. **Direct downloads.** Allow anonymous `s3:GetObject` only on this Space's objects. Per-bucket keys cannot coexist with a bucket policy. Keep the required full-access key in the server's `.env`. It covers every Space in the team. [Access keys](https://docs.digitalocean.com/products/spaces/how-to/manage-access/) and [bucket policies](https://docs.digitalocean.com/products/spaces/how-to/configure-bucket-policies/)
4. **CORS.** Under **Settings → CORS Configurations**, allow `GET` from the exact site origin, e.g. `https://drop.example.com`. Uploads go through the API. Browsers only need read access, and object permissions still apply. [Spaces CORS](https://docs.digitalocean.com/products/spaces/how-to/configure-cors/)

On your workstation, save the Spaces credentials in the AWS CLI with `aws configure --profile hdrop-spaces`. For the new Space, replace the region and both occurrences of the bucket name.

```sh
aws --profile hdrop-spaces --endpoint-url https://fra1.digitaloceanspaces.com \
  s3api put-bucket-policy --bucket example-hdrop-files --policy '{
    "Version":"2012-10-17",
    "Statement":[{"Effect":"Allow","Principal":"*","Action":"s3:GetObject",
                  "Resource":"arn:aws:s3:::example-hdrop-files/*"}]
  }'
```

### Configure and start the Droplet

Follow [Single VPS](#single-vps) for DNS, firewall and checkout. Run `./scripts/vps-install.sh` without flags. Enter your hostname and customize. Choose **S3-compatible object store**, disk cache (default) and **External Postgres server**, then enter the database details. For `hdrop-env`, use `--vps --storage-provider s3 --database-mode external`.

| Wizard setting | Example (replace region and bucket throughout) |
| --- | --- |
| `STORAGE_PROVIDER` | `s3` |
| `S3_ACCESS_KEY_ID`, `S3_SECRET_ACCESS_KEY` | Spaces key pair |
| `S3_REGION`, `S3_BUCKET_NAME` | `fra1`, `example-hdrop-files` |
| `S3_ADDRESSING_STYLE` | `virtual` (bucket in hostname) |
| `S3_ENDPOINT`, `S3_PUBLIC_URL` | `https://example-hdrop-files.fra1.digitaloceanspaces.com` |

Recipients must reach `S3_PUBLIC_URL` over HTTPS. For `path` addressing, use the service endpoint and append `/bucket-name` to its URL for public reads.

Leave `COMPOSE_PROFILES` and `POSTGRES_PASSWORD` unset. In `.env`, keep the hostname and credentials in the generated `DATABASE_URL`. Append the cluster's TLS query **inside the closing quote**.

**Advanced Edition.** Append `?sslmode=verify-full&sslrootcert=system` to use the container's trust store. Run `./scripts/vps-install.sh --start` and use the regular VPS updater.

**Standard Edition.** Append this query to verify the certificate and hostname against the downloaded CA.

```text
?sslmode=verify-full&sslrootcert=/run/hdrop-postgres-ca.crt
```

Run `chmod 644 postgres-ca.crt` so the API and its migrations can read the public CA. Save this mount as `docker-compose.override.yml` beside `docker-compose.yml`.

```yaml
services:
  api:
    volumes:
      - ./postgres-ca.crt:/run/hdrop-postgres-ca.crt:ro
```

Run Compose directly to load the override. The installer selects only the base file. Use `sudo docker compose` if system Docker is installed and your user lacks socket access.

```sh
docker compose config --quiet
docker compose config --services
docker compose up -d --build --wait
```

Expect `api`, `web`, `caddy` and `prometheus`. If `postgres` appears, the bundled profile is enabled.

**Verify Spaces.** During the [browser transfer check](#verify-an-upload-and-download), confirm a `GET` to the Spaces hostname returns 200 in the recipient's Network tab. If the API cache served it, reopen the link after the background upload finishes. Check the decrypted contents before deletion.

### DigitalOcean Marketplace (planned)

Use the Droplet setup today. The planned flow is to create a Marketplace Droplet and point your domain at it. First boot would prompt for hostname and storage, run the installer and start Caddy for HTTPS. Data would persist in volumes. Verify with the same browser transfer.

Release requires a maintained image, first-boot setup and startup, clean-install and persistence tests on DigitalOcean, and a published Marketplace listing.

## Other providers

- **AWS EC2 + RDS + S3.** Allow PostgreSQL traffic from the EC2 security group into the RDS security group. Mount the RDS CA bundle as above and use `verify-full`. Use the S3 bucket's region and virtual-hosted endpoint, e.g. `https://example-hdrop-files.s3.dualstack.eu-central-1.amazonaws.com`. Grant the server key `s3:GetObject`, `s3:PutObject` and `s3:DeleteObject` on objects and `s3:ListBucket` on the bucket. [RDS TLS](https://docs.aws.amazon.com/AmazonRDS/latest/UserGuide/PostgreSQL.Concepts.General.SSL.html) and [S3 endpoints](https://docs.aws.amazon.com/AmazonS3/latest/developerguide/dual-stack-endpoints.html)
  - For direct downloads, S3 Block Public Access settings must permit the object-read policy. For a private bucket, set `S3_PUBLIC_URL` to a read-only CloudFront distribution with origin access control. Disable caching so deletes take effect. Either endpoint must allow your site's origin. [CloudFront origin access](https://docs.aws.amazon.com/AmazonCloudFront/latest/DeveloperGuide/private-content-restricting-access-to-s3.html)
- **Other Docker hosts and VMs.** The single-VPS steps need a local Docker socket, persistent volumes and ports 80/443. For managed services, substitute the database hostname, port and CA, object endpoint and public URL. Check reachability from the API and recipients.

## Updates and backups

Back up before updating. Run `git pull --ff-only` or unpack the new Git archive into the existing checkout. Keep the project directory, `.env` and any Compose override to reuse your volumes.

- **VPS installer.** Run `./scripts/vps-install.sh --update` to pull enabled service images, rebuild the API and frontend with fresh bases, wait for healthy services and check public HTTPS. Retry a failed start with `./scripts/vps-install.sh --start`.
- **Compose override.** Run these commands from the checkout, then repeat the browser check.

```sh
docker compose pull --ignore-buildable --policy always
docker compose build --pull
docker compose up -d --no-build --wait
```

Bundled PostgreSQL tracks 19.x patches. Plan a migration for major upgrades. Each Prometheus LTS patch needs a new tag in both Compose files. Keep the Rust builder and runtime on matching Alpine releases in the backend Dockerfiles.

**VPS backup.** Run `docker compose stop`, snapshot the Docker disk and attached volumes, then run `docker compose up -d --wait`. Keep a private copy of `.env`, the checkout version, CA and override if used. Restoring uploads needs both `postgres_data` (metadata) and `local_storage` (encrypted files). `caddy_data` and `caddy_config` hold HTTPS state.

**Managed backup.** Let queued uploads finish, then stop hdrop while backing up PostgreSQL and copying encrypted objects. Test on a separate host, database and bucket with separate credentials. Existing shares retain their URLs. In the restored database, replace the prefix in `files."dataUrl"` with the copied bucket's public URL. Allow the test site's origin in bucket CORS, decrypt an unexpired share, and confirm the browser fetched it from the copy.

## Troubleshooting

| Symptom | What to check |
| --- | --- |
| HTTPS fails or startup times out | Check DNS, inbound TCP 80/443 and port conflicts. Run `docker compose ps -a` and `docker compose logs --tail=80 caddy api`. For IPv6, run `curl -6 https://drop.example.com/status` from another network. Expect `OK`. |
| API cannot connect to PostgreSQL | Check hostname, port, trusted sources, CA and migration permissions. Bundled PostgreSQL keeps its initial role password. If `.env` changed, restore the original credentials or rotate the role password and update both `DATABASE_URL` and `POSTGRES_PASSWORD`. |
| Upload succeeds but download fails | Inspect the browser request to `S3_PUBLIC_URL`. For a 403, check object permissions. For a 200 with a CORS error, check the allowed origin. A direct bucket URL should name the bucket once. For a gateway, check which bucket it serves. |
| Share links use the wrong domain | Check `HDROP_SITE_ADDRESS` (hostname) and `HDROP_PUBLIC_URL` (HTTPS origin). The frontend embeds URLs at build time. Rebuild it and recreate services after changes. [Frontend configuration](../frontend/README.md#configuration-and-hosting) |
