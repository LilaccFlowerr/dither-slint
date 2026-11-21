#!/bin/sh
set -e
cd "$(dirname "$0")/.."

if ! command -v cargo >/dev/null 2>&1; then
  curl -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal
  . "$HOME/.cargo/env"
fi
rustup target add wasm32-unknown-unknown

if ! command -v wasm-bindgen >/dev/null 2>&1; then
  version=0.2.130
  name=wasm-bindgen-$version-x86_64-unknown-linux-musl
  curl -sSfL "https://github.com/rustwasm/wasm-bindgen/releases/download/$version/$name.tar.gz" | tar -xz -C /tmp
  export PATH="/tmp/$name:$PATH"
fi

engine-web/build.sh
web/build.sh
cd docs
npm run build
