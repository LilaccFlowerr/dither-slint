#!/bin/sh
set -e
cd "$(dirname "$0")"
cargo build --release --target wasm32-unknown-unknown
wasm=target/wasm32-unknown-unknown/release/dither_slint_web.wasm
wasm-bindgen --target web --no-typescript --out-dir ../docs/public/live "$wasm"
sed "s/__BUILD__/$(shasum "$wasm" | cut -c1-12)/g" index.html > ../docs/public/live/index.html
