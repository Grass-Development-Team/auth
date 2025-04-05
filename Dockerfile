FROM rust:latest AS builder
WORKDIR /madoka_auth

COPY ./Cargo.toml ./Cargo.toml
ADD ./src ./src

RUN cargo build --release

FROM debian:bookworm-slim

RUN apt-get update -y && \
    apt-get upgrade -y

RUN apt-get install -y libssl-dev

WORKDIR /app
COPY --from=builder /madoka_auth/target/release/madoka_auth .
RUN mkdir data

WORKDIR /app/data

ENTRYPOINT ["/app/madoka_auth"]
