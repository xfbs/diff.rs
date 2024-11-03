# This file contains some helpful aliases you can use for development. These are
# run using `just`, a Rust task runner. To use them, you may need to install Just,
# which you can do by running `cargo install just`.

# List available targets
default:
    just --list

# Fetch data used in tests
test-data: test-data-info test-data-crates

# Fetch crate responses
test-data-info:
    curl -sSL https://crates.io/api/v1/crates/log -o data/log.json
    curl -sSL https://crates.io/api/v1/crates/serde -o data/serde.json
    curl -sSL https://crates.io/api/v1/crates/axum -o data/axum.json
    curl -sSL https://crates.io/api/v1/crates/reqwest -o data/reqwest.json

# Fetch crate sources
test-data-crates:
    curl -sSL https://crates.io/api/v1/crates/log/0.4.15/download -o data/log-0.4.15.crate
    curl -sSL https://crates.io/api/v1/crates/log/0.4.16/download -o data/log-0.4.16.crate
    curl -sSL https://crates.io/api/v1/crates/log/0.4.17/download -o data/log-0.4.17.crate

# Install dependencies needed to run
setup:
    rustup target add wasm32-unknown-unknown
    rustup toolchain add nightly
    cargo install trunk cargo-deny

# Format code
format:
    cargo +nightly fmt

# Launch local debug server
serve:
    trunk serve

# Run checks (same as in CI)
check:
    cargo +nightly fmt --check
    cargo deny check
    cargo clippy -- -D warnings
    cargo test
    trunk build

# generate a build, like we do in CI (including compression)
build:
    trunk clean
    trunk build --release
    find dist -not -name '*.gz' -not -name '*.br' -type f -exec gzip -vk {} \;
    find dist -not -name '*.gz' -not -name '*.br' -type f -exec brotli -vk {} \;
