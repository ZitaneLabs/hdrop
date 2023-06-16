#
# Build env
#

FROM rust:latest

WORKDIR /app

# Install dependencies
RUN cargo install --locked diesel_cli --no-default-features --features postgres

# Copy workspace contents
COPY .cargo ./.cargo
COPY Cargo.toml ./
COPY Cargo.lock ./
COPY hdrop-db ./hdrop-db
COPY hdrop-server ./hdrop-server

# Build server
RUN cargo build --release

# Copy entrypoint script
COPY hdrop-server/infra/entrypoint.sh ./
RUN chmod +x entrypoint.sh

EXPOSE 80
ENTRYPOINT ["sh", "entrypoint.sh"]
