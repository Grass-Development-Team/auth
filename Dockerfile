FROM rust:latest AS builder
WORKDIR /madoka_auth

COPY ./Cargo.toml ./Cargo.toml
ADD ./src ./src

RUN cargo build --release

FROM debian:bookworm-slim

RUN apt-get update -y && \
    apt-get upgrade -y && \
    apt-get install -y libssl-dev ca-certificates && \
    apt-get clean && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=builder /madoka_auth/target/release/madoka_auth .
RUN mkdir -p data

WORKDIR /app/data

ENTRYPOINT ["/app/madoka_auth"]
