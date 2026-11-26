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

- [Installation and deployment](./docs/installation.md)
- [Features](./docs/features.md)
- [Encryption and cryptography](./docs/crypto.md)
- [API reference](./docs/api_v1.yml)
- [Metrics](./docs/metrics.md)
- [License](./LICENSE)

## Production environment

Point your domain at the VPS and run `./scripts/vps-install.sh --start` from the checkout.
The wizard configures HTTPS, local file storage and PostgreSQL. The
[installation guide](./docs/installation.md) covers prerequisites, managed services, backups and updates.

## Development

Export `LOCALSTACK_AUTH_TOKEN` on your workstation, then start the development stack from the repository root.

```sh
docker compose -f docker-compose.dev.yml up --build
```

The stack uses local test credentials and publishes the ports below. Next.js reloads when you edit the frontend source.

| Service | Local address |
| --- | --- |
| Frontend | `http://localhost:3000` |
| API health check | `http://localhost:8080/status` |
| PostgreSQL | `localhost:5432` |
| LocalStack S3 | `http://localhost:4566` |
| Prometheus | `http://localhost:9090` |

To test edits faster, run the [Rust API](./backend/hdrop-server/README.md) or
[frontend dev server](./frontend/README.md#development) directly on your workstation.

### Development database

For native API development, use the same Compose file to start PostgreSQL 19 alone.

```sh
docker compose -f docker-compose.dev.yml up -d postgres
```

The development database uses its own `dev_postgres_data` volume. If you have
data to keep from the old PostgreSQL 15 development container, dump it before
replacing the container and restore it into 19.

## License

At Zitane Labs, we are committed to promoting a free and open Internet. We believe in the principles of open source software and the power of community collaboration.

We have chosen to license hdrop under the Affero General Public License version 3 (AGPLv3) for a few key reasons:

1. **Community Benefit:** The AGPLv3 license ensures that anyone who modifies hdrop and then uses their modified version to provide a service over a network (such as a SaaS product), must make their modifications available to the community. This promotes collaboration and ensures the wider community can benefit from these enhancements.

2. **User Freedom:** The AGPLv3 license gives users the freedom to use, study, share, and modify the software. We believe in these freedoms and want to extend them to all users of hdrop.

3. **Hardened Security:** By requiring users of modified versions to share their modifications, the AGPLv3 license helps to ensure that the software remains secure. Security-relevant modifications are more likely to be shared with the community, and can be upstreamed into the main project to benefit all users.

**hdrop is free for everyone**, and we are committed to keeping it that way.
