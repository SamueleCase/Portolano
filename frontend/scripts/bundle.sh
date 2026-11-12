#!/bin/sh
# Builds Portolano.app and its disk image with the CLI as sidecar
# (SPEC-UI §4.10). `externalBin` lives only in tauri.bundle.json, so that the
# builds of the workspace never need the copied binary.
set -eu

frontend=$(cd "$(dirname "$0")/.." && pwd)
app="$frontend/../crates/portolano-app"
triple=$(rustc --print host-tuple)

cd "$frontend/.."
cargo build --release -p portolano-cli
mkdir -p "$app/binaries"
cp target/release/portolano "$app/binaries/portolano-$triple"

cd "$frontend"
npm run tauri -- build --config "$app/tauri.bundle.json"
