# hdrop
![Backed by Zitane Labs][badge_zitane]
![Powered by Rust][badge_rust]
![MSRV][badge_msrv]
![License: AGPL 3.0][badge_license]
![Github CI][github_ci]

[badge_zitane]: https://badgers.space/badge/Backed%20by/Zitane%20Labs/pink
[badge_rust]: https://badgers.space/badge/Powered%20by/Rust/orange
[badge_msrv]: https://badgers.space/badge/MSRV/1.88.0/green
[badge_license]: https://badgers.space/github/license/ZitaneLabs/hdrop
[github_ci]: https://badgers.space/github/checks/zitanelabs/hdrop

<img align="right" width="64" src="./frontend/public/logo512.png" />

**Simple, self-hosted encrypted file transfer.**

## Features
<img align="right" width="200" src="https://github.com/ZitaneLabs/hdrop/assets/107320179/b88ee1a8-219e-4659-bbd5-03782ca61f17" />

- Easy to self-host
- Modern UI/UX
- End-to-end encrypted (E2EE)
- Automatic file deletion
- Supports S3 and simple disk storage
- No user accounts
- Metrics included

## Documentation

We are migrating our docs to [the wiki](https://github.com/ZitaneLabs/hdrop/wiki).  
You will find extensive and up-to-date documentation there.

| Name                  | Link                                  |
| --------------------- | ------------------------------------- |
| License               | [LICENSE](./LICENSE)                  |
| Security              | [security.md](./docs/security.md)     |
| API Spec v1 (OpenAPI) | [api_v1.yml](./docs/api_v1.yml)       |
| Changelog             | [changelog.md](./docs/changelog.md)   |

## Production environment

### Install on a single VPS

Log in as your usual Linux user and run this from the repository directory.

```sh
./scripts/vps-install.sh --start
```

Point a public hostname's AAAA record at your IPv6 VPS and allow inbound TCP
80 and 443. On a host with native IPv6 and no IPv4 gateway, you can transfer a
checkout or Git archive over SSH, preserving executable permissions. Cloning
directly from GitHub requires a working route to GitHub, such as provider NAT64
with DNS64. The installer works from a folder without Git metadata.

The installer sets up Docker Engine, Buildx and Compose on supported Debian
and Ubuntu hosts, using sudo when needed. If you already run Docker, Engine
27 and newer allocate IPv6 subnets automatically. Older engines need a
configured IPv6 address pool. Choose IPv6 networking (the default), enter the
public hostname, and accept local storage, disk caching and bundled PostgreSQL,
or choose advanced settings. Docker builds the wizard and application images on
the VPS. The first build needs outbound access to Docker Hub and package
repositories and can take a while on a small server.

With an existing rootless Docker Engine, publishing host TCP ports 80 and 443
requires either a host `net.ipv4.ip_unprivileged_port_start` of 80 or lower, or
`CAP_NET_BIND_SERVICE` on RootlessKit with the user Docker service restarted.
The installer checks this before building on `--start` and `--update`.
See [Docker's rootless port instructions](https://docs.docker.com/engine/security/rootless/tips/#exposing-privileged-ports).

The installer saves a private `.env`, waits for the services, and checks the
public homepage and API over trusted HTTPS. IPv6 mode checks both over IPv6,
uses the host network for image builds, and enables IPv6 on the application
bridge. Rerun the same command after a failed install. It reuses `.env`, database
credentials, certificates and file volumes.
To configure without starting, omit `--start`. For a full encrypted
upload/download check from another machine, see the [browser checks](./frontend/README.md#checks).

When reusing a manually created `.env`, the installer defaults to IPv6
networking. Set `HDROP_IPV6=false` and `HDROP_BUILD_NETWORK=default` in
that file for an IPv4-only deployment.

### Update an installed VPS

Run the updater from the installed checkout.

```sh
./scripts/vps-install.sh --update
```

The updater pulls the enabled service images and rebuilds the API and frontend
with fresh base images before restarting affected services. It uses the existing
`.env` and volumes, waits for healthy services, and checks the public homepage
and API over HTTPS. To deploy a new hdrop release, update the checkout first.

Bundled PostgreSQL tracks patch releases within major version 19. Plan a database
migration before changing the major version of a populated volume. Prometheus
publishes exact patch tags for its LTS line, so new LTS patches require updating
the tag in both Compose files before running the updater. When moving to a new
Alpine release, update the Rust builder and runtime together in both backend
Dockerfiles so they use the same system libraries.

### Manual production setup

Caddy handles HTTPS and routes `/v1/*` and `/status` to the Rust API. The frontend
defaults to static files served by unprivileged, read-only Nginx, with an optional
Next.js runtime. See [frontend setup](./frontend/README.md) for configuration and tests.

You'll need Docker Engine, the Compose plugin, and Git. Point your domain at
the host and allow TCP ports 80 and 443.

Generate the Compose `.env` file with Rust installed. You can do this on your
workstation using the same checkout:

```sh
(cd backend && cargo run -p hdrop-env -- --output ../.env)
```

For scripted deployments, pass values with flags and add `--non-interactive`.
Review the generated `TODO_...` values and keep `.env` private in the host's
repository root, then run:

```sh
docker compose config --quiet
docker compose up -d --build
```

To use the Next.js production server, set `HDROP_WEB_RUNTIME=nextjs`
in `.env`, then run `docker compose up -d --build web`.

Bundled Postgres stores its initialized users in the `postgres_data` Docker
volume. If you regenerate `.env` or change `POSTGRES_PASSWORD` after the first
startup, Postgres will keep the old password in that existing volume. Either
update the database role password inside Postgres or, for a disposable installation,
recreate the bundled database volume before starting with the new `.env`.
Recreating the volume deletes its stored data.

## Development database

Start PostgreSQL 19 on its own for local development with this command.

```sh
docker compose -f docker-compose.dev.yml up -d postgres
```

The development database uses its own `dev_postgres_data` volume. If you have
data to keep from the old PostgreSQL 15 development container, dump it before
replacing the container and restore it into 19. For the full development stack,
export `LOCALSTACK_AUTH_TOKEN` for the LocalStack S3 emulator, then run
`docker compose -f docker-compose.dev.yml up --build`.

## License

At Zitane Labs, we are committed to promoting a free and open Internet. We believe in the principles of open source software and the power of community collaboration.

We have chosen to license hdrop under the Affero General Public License version 3 (AGPLv3) for a few key reasons:

1. **Community Benefit:** The AGPLv3 license ensures that anyone who modifies hdrop and then uses their modified version to provide a service over a network (such as a SaaS product), must make their modifications available to the community. This promotes collaboration and ensures the wider community can benefit from these enhancements.

2. **User Freedom:** The AGPLv3 license gives users the freedom to use, study, share, and modify the software. We believe in these freedoms and want to extend them to all users of hdrop.

3. **Hardened Security:** By requiring users of modified versions to share their modifications, the AGPLv3 license helps to ensure that the software remains secure. Security-relevant modifications are more likely to be shared with the community, and can be upstreamed into the main project to benefit all users.

**hdrop is free for everyone**, and we are committed to keeping it that way.
