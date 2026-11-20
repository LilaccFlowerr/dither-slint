#!/bin/sh
set -e
cd "$(dirname "$0")"
cargo build --release --target wasm32-unknown-unknown
wasm-bindgen --target web --no-typescript --out-dir ../docs/src/engine target/wasm32-unknown-unknown/release/dither_slint_engine_web.wasm
