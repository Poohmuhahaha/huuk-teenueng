#!/usr/bin/env bash
# Build the Rust backend in release mode and stage it as the Tauri sidecar
# binary (triple-suffixed, as `externalBin` requires). Re-run after any
# server change; the result is gitignored build output.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
TRIPLE="$(rustc --print host-tuple)"
BIN_DIR="$ROOT/../desktop/binaries"

cargo build --release --manifest-path "$ROOT/../server/Cargo.toml"
mkdir -p "$BIN_DIR"
cp "$ROOT/../server/target/release/content-planner-server" "$BIN_DIR/content-planner-server-${TRIPLE}"
chmod +x "$BIN_DIR/content-planner-server-${TRIPLE}"
echo "sidecar ready: $BIN_DIR/content-planner-server-${TRIPLE}"
