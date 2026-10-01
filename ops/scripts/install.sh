#!/usr/bin/env bash
# Docker-free installer: builds Content Planner and installs it as a systemd
# service. The binary serves the API *and* the built frontend (STATIC_DIR), so
# no nginx/Docker is required. Put any TLS reverse proxy of your choice in
# front (see ops/deploy/Caddyfile.standalone for a one-line Caddy example).
#
#   sudo ops/scripts/install.sh [options]
#
#   --prefix DIR      install root                 (default /opt/content-planner)
#   --data-dir DIR    state directory              (default /var/lib/content-planner)
#   --env-file FILE   environment file             (default /etc/content-planner.env)
#   --unit-file FILE  systemd unit path            (default /etc/systemd/system/content-planner.service)
#   --user NAME       service account              (default contentplanner)
#   --port N          listen port                  (default 8787)
#   --host URL        public origin (links/CORS)   (default http://localhost:PORT)
#   --admin-email E   bootstrap Owner email        (default admin@example.com)
#   --update          keep config + data, rebuild and restart
#   --no-build        install the existing release binary and app/dist
#   --skip-service    install files without touching systemd
#   -h, --help
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

PREFIX="/opt/content-planner"
DATA_DIR="/var/lib/content-planner"
ENV_FILE="/etc/content-planner.env"
UNIT_FILE="/etc/systemd/system/content-planner.service"
RUN_USER="contentplanner"
PORT="8787"
PUBLIC_HOST=""
ADMIN_EMAIL="admin@example.com"
UPDATE=0
BUILD=1
SKIP_SERVICE=0

while (($#)); do
  case "$1" in
    --prefix) PREFIX="${2:?--prefix needs a directory}"; shift 2 ;;
    --data-dir) DATA_DIR="${2:?--data-dir needs a directory}"; shift 2 ;;
    --env-file) ENV_FILE="${2:?--env-file needs a path}"; shift 2 ;;
    --unit-file) UNIT_FILE="${2:?--unit-file needs a path}"; shift 2 ;;
    --user) RUN_USER="${2:?--user needs a name}"; shift 2 ;;
    --port) PORT="${2:?--port needs a number}"; shift 2 ;;
    --host) PUBLIC_HOST="${2:?--host needs a URL}"; shift 2 ;;
    --admin-email) ADMIN_EMAIL="${2:?--admin-email needs an address}"; shift 2 ;;
    --update) UPDATE=1; shift ;;
    --no-build) BUILD=0; shift ;;
    --skip-service) SKIP_SERVICE=1; shift ;;
    -h | --help) sed -n '2,22p' "$0" | sed 's/^# \{0,1\}//'; exit 0 ;;
    *) echo "unknown option: $1 (try --help)" >&2; exit 2 ;;
  esac
done

log() { printf '%s\n' "$*"; }
die() { printf 'error: %s\n' "$*" >&2; exit 1; }

gen_secret() {
  local bytes="$1"
  if command -v openssl >/dev/null 2>&1; then
    openssl rand -hex "$bytes"
  else
    head -c "$((bytes * 2))" /dev/urandom | od -An -tx1 | tr -d ' \n'
  fi
}

# ---- build ----
if ((BUILD)); then
  command -v cargo >/dev/null 2>&1 || die "cargo is required (or install with --no-build)"
  log "Building backend (release)…"
  cargo build --release --manifest-path "$ROOT/server/Cargo.toml"
  log "Building frontend…"
  command -v bun >/dev/null 2>&1 || die "bun is required to build the frontend (https://bun.sh) (or install with --no-build)"
  (cd "$ROOT/app" && bun install --frozen-lockfile && bun run build)
fi

BIN="$ROOT/server/target/release/content-planner-server"
DIST="$ROOT/app/dist"
[[ -x "$BIN" ]] || die "missing $BIN — run without --no-build"
[[ -f "$DIST/index.html" ]] || die "missing $DIST/index.html — run without --no-build"

# ---- privileges ----
if [[ "$(id -u)" -eq 0 ]]; then
  if ! id "$RUN_USER" >/dev/null 2>&1; then
    useradd --system --create-home --home-dir "$PREFIX" --shell /usr/sbin/nologin "$RUN_USER"
    log "Created system user $RUN_USER"
  fi
else
  ((SKIP_SERVICE)) || die "run with sudo, or use --skip-service with --prefix/--data-dir/--env-file overrides"
  RUN_USER="$(id -un)"
  log "warning: not root — installing into $PREFIX for user $RUN_USER (no systemd changes)"
fi

SYSTEMD_AVAILABLE=0
if [[ -d /run/systemd/system ]] && command -v systemctl >/dev/null 2>&1; then
  SYSTEMD_AVAILABLE=1
fi
if ((!SKIP_SERVICE)) && ((SYSTEMD_AVAILABLE == 0)); then
  log "warning: systemd is not running — installing files only"
  SKIP_SERVICE=1
fi

# ---- files ----
install -d -m 0755 "$PREFIX/bin" "$PREFIX/dist"
# The store holds password hashes and unpublished content — owner-only.
install -d -m 0700 "$DATA_DIR"

if ((UPDATE)) && [[ -f "$DATA_DIR/store.json" ]]; then
  install -d -m 0700 "$DATA_DIR/backups"
  backup="$DATA_DIR/backups/pre-update-$(date +%Y%m%d-%H%M%S).json"
  cp "$DATA_DIR/store.json" "$backup"
  chmod 600 "$backup"
  log "Pre-update backup: $backup"
fi

install -m 0755 "$BIN" "$PREFIX/bin/content-planner-server"
rm -rf "${PREFIX:?}/dist"
cp -R "$DIST" "$PREFIX/dist"
log "Installed binary + frontend into $PREFIX"

# ---- configuration ----
GENERATED=0
if [[ ! -f "$ENV_FILE" ]]; then
  umask 077
  ADMIN_PASSWORD="$(gen_secret 12)"
  ADMIN_TOKEN="$(gen_secret 24)"
  ORIGIN="${PUBLIC_HOST:-http://localhost:$PORT}"
  ORIGIN="${ORIGIN%/}"
  cat > "$ENV_FILE" <<ENV
# Content Planner environment (generated $(date -u +%Y-%m-%dT%H:%M:%SZ))
PORT=$PORT
BIND_ADDR=127.0.0.1
DATA_FILE=$DATA_DIR/store.json
STATIC_DIR=$PREFIX/dist
AUTH_REQUIRED=true
ALLOW_REGISTRATION=false
ADMIN_EMAIL=$ADMIN_EMAIL
ADMIN_PASSWORD=$ADMIN_PASSWORD
ADMIN_TOKEN=$ADMIN_TOKEN
FRONTEND_URL=$ORIGIN
PUBLIC_URL=$ORIGIN
CORS_ORIGINS=$ORIGIN
RUST_LOG=info
ENV
  chmod 600 "$ENV_FILE"
  GENERATED=1
  log "Wrote $ENV_FILE"
else
  log "Keeping existing $ENV_FILE"
fi

if [[ "$(id -u)" -eq 0 ]]; then
  chown -R "$RUN_USER":"$RUN_USER" "$DATA_DIR" "$PREFIX"
  chown root:"$RUN_USER" "$ENV_FILE" 2>/dev/null || true
  chmod 640 "$ENV_FILE" 2>/dev/null || true
fi

# ---- systemd ----
if ((!SKIP_SERVICE)); then
  sed -e "s|__RUN_USER__|$RUN_USER|g" \
    -e "s|__APP_DIR__|$PREFIX|g" \
    -e "s|__ENV_FILE__|$ENV_FILE|g" \
    -e "s|__DATA_DIR__|$DATA_DIR|g" \
    "$ROOT/ops/deploy/systemd/content-planner.service" > "$UNIT_FILE"
  systemctl daemon-reload
  if ((UPDATE)); then
    systemctl restart content-planner
    log "Service restarted"
  else
    systemctl enable --now content-planner
    log "Service enabled and started"
  fi
  sleep 1
  systemctl --no-pager --full status content-planner 2>/dev/null | head -12 || true
fi

# ---- summary ----
log ""
log "Install dir : $PREFIX"
log "Data file   : $DATA_DIR/store.json"
log "Config      : $ENV_FILE"
log "URL         : http://localhost:$PORT"
log "TLS         : put Caddy/nginx in front (see ops/deploy/Caddyfile.standalone)"
if ((GENERATED)); then
  log ""
  log "Bootstrap Owner    : $ADMIN_EMAIL"
  log "Generated password : $ADMIN_PASSWORD"
  log "Recovery token     : $ADMIN_TOKEN"
  log "Change the password after the first login and store the token safely."
fi
