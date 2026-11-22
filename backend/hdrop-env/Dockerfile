FROM rust:alpine3.24 AS builder
WORKDIR /src
COPY . .
RUN --mount=type=cache,id=hdrop-cargo-registry,target=/usr/local/cargo/registry,sharing=locked \
    cargo build --release --locked -p hdrop-env

FROM alpine:3.24
COPY --from=builder /src/target/release/hdrop-env /usr/local/bin/hdrop-env
ENTRYPOINT ["hdrop-env"]
