#!/usr/bin/env bash
# One command to run the whole stack:
#
#   bash ops/scripts/dev.sh          → Rust backend (:8787) + Vue dev server, frontend wired to the API
#   bash ops/scripts/dev.sh --mock   → frontend only, in-memory mock data (no backend)
#   bash ops/scripts/dev.sh --server → backend only
#   (or from the repo root: make dev / make dev-mock / make dev-server)
#
# Ctrl+C stops everything (the backend is cleaned up automatically).
set -euo pipefail
set -m  # job control: each background job becomes its own process group

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
PORT="${PORT:-8787}"
HEALTH="http://localhost:${PORT}/api/health"
MODE="all"

for arg in "$@"; do
  case "$arg" in
    --mock) MODE="mock" ;;
    --server) MODE="server" ;;
    --app) MODE="mock" ;;
    *) echo "unknown option: $arg (use --mock | --server)"; exit 1 ;;
  esac
done

SRV_PID=""
APP_PID=""
# Kills a background job's whole process group (vite is a grandchild of bun, so a
# plain kill of $! would leave it running).
kill_tree() {
  local pid="$1"
  [[ -z "$pid" ]] && return 0
  kill -- -"$pid" 2>/dev/null || kill "$pid" 2>/dev/null || true
  for _ in $(seq 1 10); do
    kill -0 "$pid" 2>/dev/null || break
    sleep 0.2
  done
  kill -9 -- -"$pid" 2>/dev/null || kill -9 "$pid" 2>/dev/null || true
  wait "$pid" 2>/dev/null || true
}
cleanup() {
  kill_tree "$APP_PID"
  kill_tree "$SRV_PID"
}
trap cleanup EXIT INT TERM HUP

if [[ "$MODE" != "mock" ]]; then
  if curl -sf "$HEALTH" >/dev/null 2>&1; then
    echo "✔ backend  : already running on :${PORT} (reusing)"
  else
    echo "▶ backend  : building… (first run takes a minute)"
    cargo build --manifest-path "$ROOT/server/Cargo.toml" --quiet
    # dev.sh is a local development entry point: run the backend demo seed
    # (auth optional, known demo accounts, open registration).
    PORT="$PORT" DEMO_MODE=1 BIND_ADDR="${BIND_ADDR:-127.0.0.1}" \
      "$ROOT/server/target/debug/content-planner-server" &
    SRV_PID=$!

    for _ in $(seq 1 60); do
      if curl -sf "$HEALTH" >/dev/null 2>&1; then break; fi
      sleep 0.25
    done
    if ! curl -sf "$HEALTH" >/dev/null 2>&1; then
      echo "✖ backend  : failed to start on :${PORT}" >&2
      exit 1
    fi
    echo "✔ backend  : http://localhost:${PORT}"
  fi
fi

if [[ "$MODE" == "server" ]]; then
  echo "… backend only — Ctrl+C to stop"
  wait "$SRV_PID"
  exit 0
fi

if [[ "$MODE" == "mock" ]]; then
  unset VITE_API_URL || true
  echo "▶ frontend : mock data (no backend)"
else
  export VITE_API_URL="http://localhost:${PORT}"
  echo "▶ frontend : connected to backend (VITE_API_URL=${VITE_API_URL})"
fi

# Bun only (single lockfile: bun.lock).
command -v bun >/dev/null 2>&1 || { echo "bun is required (https://bun.sh)"; exit 1; }
echo "▶ frontend : launching with bun run dev"
( cd "$ROOT/app" && exec bun run dev ) &
APP_PID=$!
wait "$APP_PID"
