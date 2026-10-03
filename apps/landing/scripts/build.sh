#!/bin/sh
set -eu
cd "$(dirname "$0")/.."
worker-build --release --no-panic-recovery --out-dir build -- --locked
cargo build --locked -p relay-landing-client --release --target wasm32-unknown-unknown
wasm-bindgen target/wasm32-unknown-unknown/release/relay_landing_client.wasm --out-dir public/client --target web --no-typescript
cargo run --locked --quiet --bin assemble
