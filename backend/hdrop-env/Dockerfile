FROM rust:alpine AS builder
WORKDIR /src
COPY . .
RUN cargo build --release --locked -p hdrop-env

FROM alpine:3.23
COPY --from=builder /src/target/release/hdrop-env /usr/local/bin/hdrop-env
ENTRYPOINT ["hdrop-env"]
