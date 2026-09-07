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

FROM alpine:3.23

WORKDIR /app

RUN apk add --no-cache ca-certificates libgcc openssl postgresql-libs
RUN addgroup -S hdrop && adduser -S -D -H -G hdrop hdrop

COPY --from=builder /app/target/release/hdrop-server /usr/local/bin/hdrop-server
COPY --from=builder /usr/local/cargo/bin/diesel /usr/local/bin/diesel
COPY --from=builder /app/hdrop-db/migrations ./hdrop-db/migrations

# Copy entrypoint script
COPY --chmod=0555 hdrop-server/infra/entrypoint.sh ./

# Transfer ownership of necessary directories to the hdrop user
RUN chown -R root:root /app \
    && install -d -o hdrop -g hdrop -m 0750 /data/hdrop-files /cache/hdrop
USER hdrop:hdrop

# API
EXPOSE 3000

# Prometheus exporter
EXPOSE 3001

ENTRYPOINT ["sh", "entrypoint.sh"]
