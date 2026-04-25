FROM rust:alpine AS builder

WORKDIR /app
ENV RUSTFLAGS="-C target-feature=-crt-static"

RUN apk add --no-cache build-base git openssl-dev pkgconfig postgresql-dev

# Install dependencies
RUN cargo install --locked diesel_cli --no-default-features --features postgres

# Copy workspace contents
COPY .cargo ./.cargo
COPY Cargo.toml ./
COPY Cargo.lock ./
COPY hdrop-db ./hdrop-db
# Cargo resolves every workspace member before building -p hdrop-server.
# hdrop-env is only needed for manifest resolution and is removed after build.
COPY hdrop-env ./hdrop-env
COPY hdrop-shared ./hdrop-shared
COPY hdrop-server ./hdrop-server

# Build server
RUN cargo build --release --locked -p hdrop-server

FROM node:25-alpine

WORKDIR /app

RUN apk add --no-cache ca-certificates libgcc openssl postgresql-libs \
    && npm i -g pm2@latest

COPY --from=builder /app/target/release/hdrop-server /usr/local/bin/hdrop-server
COPY --from=builder /usr/local/cargo/bin/diesel /usr/local/bin/diesel
COPY --from=builder /app/hdrop-db/migrations ./hdrop-db/migrations

# Copy entrypoint script
COPY hdrop-server/infra/entrypoint.sh ./
RUN chmod +x entrypoint.sh

# API
EXPOSE 80

# Prometheus exporter
EXPOSE 3001

ENTRYPOINT ["sh", "entrypoint.sh"]
