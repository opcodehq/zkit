#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/../.."
expected_version=$(awk '/^name = "wasm-bindgen"$/{getline; gsub(/"/, "", $3); print $3; exit}' Cargo.lock)
if ! command -v wasm-bindgen >/dev/null; then
  printf 'Install the matching CLI first: cargo install wasm-bindgen-cli --version %s --locked\n' "$expected_version" >&2
  exit 1
fi
if [[ "$(wasm-bindgen --version)" != "wasm-bindgen $expected_version" ]]; then
  printf 'The CLI must match Cargo.lock. Run: cargo install wasm-bindgen-cli --version %s --locked\n' "$expected_version" >&2
  exit 1
fi
cargo build --locked -p zkit-dither --target wasm32-unknown-unknown --features web --release
wasm-bindgen target/wasm32-unknown-unknown/release/zkit_dither.wasm --target web --out-dir website/src/engine --out-name dither
