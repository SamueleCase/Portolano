#!/bin/sh
# Builds Portolano.app and its disk image with the CLI as sidecar
# (SPEC-UI §4.10). `externalBin` lives only in tauri.bundle.json, so that the
# builds of the workspace never need the copied binary. Ends with the checks
# of check-bundle.sh, which also runs alone.
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

# A bundle that misses a piece of SPEC-UI §4.10 fails here, not when it is
# opened: `npm run bundle` cannot end well with an incomplete bundle.
sh "$frontend/scripts/check-bundle.sh"
