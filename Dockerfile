ARG RUST_VERSION=1.94

############################
#      Debian runtime      #
############################
FROM rust:${RUST_VERSION}-bookworm AS builder-debian
WORKDIR /workspace

RUN apt-get update -y \
    && apt-get install -y --no-install-recommends pkg-config libssl-dev ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY Cargo.toml Cargo.lock ./
COPY auth ./auth
COPY crates ./crates

RUN cargo build --release -p auth

FROM debian:bookworm-slim AS runtime-debian-slim

RUN apt-get update -y \
    && apt-get install -y --no-install-recommends ca-certificates libssl3 \
    && rm -rf /var/lib/apt/lists/* \
    && useradd --system --create-home --home-dir /app app

WORKDIR /app
COPY --from=builder-debian /workspace/target/release/auth /usr/local/bin/auth

USER app
EXPOSE 7817
ENTRYPOINT ["/usr/local/bin/auth"]

############################
#      Alpine runtime      #
############################
FROM rust:${RUST_VERSION}-alpine AS builder-alpine
WORKDIR /workspace

RUN apk add --no-cache build-base musl-dev pkgconfig openssl-dev ca-certificates

COPY Cargo.toml Cargo.lock ./
COPY auth ./auth
COPY crates ./crates

RUN cargo build --release -p auth

FROM alpine:3.20 AS runtime-alpine

RUN apk add --no-cache ca-certificates libssl3 \
    && adduser -D -h /app app

WORKDIR /app
COPY --from=builder-alpine /workspace/target/release/auth /usr/local/bin/auth

USER app
EXPOSE 7817
ENTRYPOINT ["/usr/local/bin/auth"]

# Entrypoint for the default runtime
FROM runtime-debian-slim AS runtime
