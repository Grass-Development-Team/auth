# Default recipe to display available commands
default:
    @just --list

# Development recipes
dev: build
    cargo run

dev-watch:
    cargo watch -x run

# Testing recipes
test:
    cargo test

test-watch:
    cargo watch -x test

# Building recipes
build:
    cargo build --all

build-release:
    cargo build --release

# Code quality recipes
check:
    cargo check

clippy:
    cargo clippy -- -D warnings

fmt:
    cargo fmt

fmt-check:
    cargo fmt -- --check

# Security and dependency management
@audit:
    cargo audit

@outdated:
    cargo outdated -R

@msrv:
    cargo msrv find

# Combined quality check
@quality: audit outdated msrv fmt-check clippy test

# Docker recipes
docker-build:
    docker build -t madoka_auth:latest .

docker-run:
    docker run --rm -p 8080:7817 madoka_auth:latest

docker-dev: docker-build docker-run

# Cleanup recipes
clean:
    cargo clean
    docker rmi madoka_auth:latest || true

clean-all: clean
    docker system prune -f

# Release preparation
prepare-release: quality build-release
    @echo "Release preparation complete!"

# Development setup
setup:
    rustup component add clippy rustfmt
    cargo install cargo-watch cargo-audit cargo-outdated cargo-msrv
    @echo "Development environment setup complete!"
