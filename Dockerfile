FROM rust:latest as builder
WORKDIR /madoka_auth

RUN cargo new --bin madoka_auth

COPY ./Cargo.toml ./Cargo.toml
RUN cargo build --release && \
    rm src/*.rs target/release/deps/madoka_auth*

ADD ./src .

RUN cargo build --release

FROM debian:bookworm-slim
WORKDIR /app

COPY --from=builder /madoka_auth/target/release/madoka_auth .

ENTRYPOINT ["./madoka_auth"]
