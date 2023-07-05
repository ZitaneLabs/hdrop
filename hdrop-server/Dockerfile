#
# Build env
#

FROM rust:latest

WORKDIR /app

# Install node
RUN curl -sL https://deb.nodesource.com/setup_20.x | bash -
RUN apt -y install nodejs

# Install dependencies
RUN cargo install --locked diesel_cli --no-default-features --features postgres

# Copy workspace contents
COPY .cargo ./.cargo
COPY Cargo.toml ./
COPY Cargo.lock ./
COPY hdrop-db ./hdrop-db
COPY hdrop-shared ./hdrop-shared
COPY hdrop-server ./hdrop-server

# Build server
RUN cargo build --release

# Copy entrypoint script
COPY hdrop-server/infra/entrypoint.sh ./
RUN chmod +x entrypoint.sh

# Install pm2
RUN npm i -g pm2@latest

# API
EXPOSE 80

# Prometheus exporter
EXPOSE 3001

ENTRYPOINT ["sh", "entrypoint.sh"]
