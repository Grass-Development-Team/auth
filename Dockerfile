FROM rust:latest as builder
WORKDIR /madoka_auth

RUN cargo new --bin madoka_auth
RUN rm ./src/*

COPY ./Cargo.toml ./Cargo.toml
ADD ./src ./src

RUN cargo build --release

FROM debian:bookworm-slim

WORKDIR /app
COPY --from=builder /madoka_auth/target/release/madoka_auth .
RUN mkdir data

WORKDIR /app/data

ENTRYPOINT ["../madoka_auth"]
