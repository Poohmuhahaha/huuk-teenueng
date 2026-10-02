#!/usr/bin/env bash
# Content Planner deployment helper (Docker Compose).
#
#   ops/scripts/deploy.sh init              create ops/deploy/.env.production with generated secrets
#   ops/scripts/deploy.sh up [--tls]        build + start (adds the Caddy TLS profile with --tls)
#   ops/scripts/deploy.sh down              stop containers, keep data
#   ops/scripts/deploy.sh destroy           stop and DELETE volumes (double confirmation)
#   ops/scripts/deploy.sh ps                show container status
#   ops/scripts/deploy.sh logs [service]    follow logs (api | web | proxy)
#   ops/scripts/deploy.sh restart           restart the stack
#   ops/scripts/deploy.sh update            rebuild (pulling base images) and roll forward
#   ops/scripts/deploy.sh backup [file]     copy the API snapshot to a local file
#   ops/scripts/deploy.sh restore <file>    replace the snapshot and restart the API
#   ops/scripts/deploy.sh smoke [url]       run ops/scripts/smoke.sh against the deployment
#   ops/scripts/deploy.sh help
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"

ENV_FILE="${ENV_FILE:-ops/deploy/.env.production}"
ENV_EXAMPLE="ops/deploy/.env.production.example"
BACKUP_DIR="${BACKUP_DIR:-backups}"
COMPOSE=(docker compose -f ops/docker-compose.yml --env-file "$ENV_FILE")

log() { printf '%s\n' "$*"; }
die() { printf 'error: %s\n' "$*" >&2; exit 1; }

require_docker() {
  if ! command -v docker >/dev/null 2>&1; then
    die "docker is not installed.
This host can run the Docker-free stack instead:
  sudo ops/scripts/install.sh          # one binary serves API + frontend + systemd
  ops/scripts/install.sh --help        # options (prefix, port, admin email, …)
Other container hosts are covered in docs/DEPLOYMENT.md."
  fi
  docker compose version >/dev/null 2>&1 || die "docker compose v2 is required"
}

require_env() {
  [[ -f "$ENV_FILE" ]] || die "missing $ENV_FILE — run: ops/scripts/deploy.sh init"
}

# Cryptographically random hex string (openssl preferred).
gen_secret() {
  local bytes="$1"
  if command -v openssl >/dev/null 2>&1; then
    openssl rand -hex "$bytes"
  else
    head -c "$((bytes * 2))" /dev/urandom | od -An -tx1 | tr -d ' \n'
  fi
}

# Read a plain KEY=value from the env file without executing it.
env_value() {
  [[ -f "$ENV_FILE" ]] || return 0
  grep -E "^$1=" "$ENV_FILE" 2>/dev/null | tail -1 | cut -d= -f2- | tr -d '[:space:]'
}

cmd_init() {
  [[ -f "$ENV_FILE" ]] && die "$ENV_FILE already exists — edit it or move it aside first"
  [[ -f "$ENV_EXAMPLE" ]] || die "missing $ENV_EXAMPLE"

  local password token
  password="$(gen_secret 12)"
  token="$(gen_secret 24)"
  umask 077
  sed -e "s/__ADMIN_PASSWORD__/${password}/" -e "s/__ADMIN_TOKEN__/${token}/" \
    "$ENV_EXAMPLE" > "$ENV_FILE"
  chmod 600 "$ENV_FILE"

  log "Created $ENV_FILE (mode 600)."
  log ""
  log "Bootstrap Owner     : $(env_value ADMIN_EMAIL)"
  log "Generated password  : ${password}"
  log "Recovery token      : ${token}"
  log ""
  log "Keep both safe and change the password after the first login."
  log "Before going live, edit $ENV_FILE:"
  log "  FRONTEND_URL / PUBLIC_URL / CORS_ORIGINS  -> your public URL"
  log "  SITE_ADDRESS                              -> your domain (for ops/scripts/deploy.sh up --tls)"
  log ""
  local port
  port="$(env_value WEB_PORT)"; port="${port:-8080}"
  log "Then:"
  log "  ops/scripts/deploy.sh up          # http://localhost:${port}"
  log "  ops/scripts/deploy.sh up --tls    # Caddy on 80/443 with automatic HTTPS"
}

cmd_up() {
  require_docker
  require_env
  local with_tls=0
  for arg in "$@"; do
    case "$arg" in
      --tls) with_tls=1 ;;
      *) die "unknown option: $arg (usage: ops/scripts/deploy.sh up [--tls])" ;;
    esac
  done

  log "Building images…"
  "${COMPOSE[@]}" build
  if ((with_tls)); then
    log "Starting stack with TLS profile…"
    "${COMPOSE[@]}" --profile tls up -d
  else
    log "Starting stack…"
    "${COMPOSE[@]}" up -d
  fi
  "${COMPOSE[@]}" ps
  local port
  port="$(env_value WEB_PORT)"; port="${port:-8080}"
  log ""
  log "Web UI:  http://localhost:${port}"
  log "API:     http://localhost:${port}/api/health"
}

cmd_down() {
  require_docker
  require_env
  "${COMPOSE[@]}" --profile tls down
}

cmd_destroy() {
  require_docker
  require_env
  log "This stops the stack and DELETES the data volume (all content/accounts)."
  read -r -p "Type 'delete everything' to continue: " answer
  [[ "$answer" == "delete everything" ]] || die "aborted"
  "${COMPOSE[@]}" --profile tls down --volumes --remove-orphans
}

cmd_backup() {
  require_docker
  require_env
  local out="${1:-$BACKUP_DIR/content-planner-$(date +%Y%m%d-%H%M%S).json}"
  mkdir -p "$(dirname "$out")"
  chmod 700 "$BACKUP_DIR" 2>/dev/null || true
  umask 077
  if ! "${COMPOSE[@]}" ps --status running --services 2>/dev/null | grep -qx api; then
    die "the api container is not running (start it with ops/scripts/deploy.sh up)"
  fi
  "${COMPOSE[@]}" exec -T api cat /app/data/content-planner.json > "$out"
  [[ -s "$out" ]] || die "backup file is empty"
  chmod 600 "$out"
  # The snapshot holds password hashes and unpublished content: verify we got a
  # complete JSON document before calling the backup good.
  if command -v python3 >/dev/null 2>&1; then
    python3 -c 'import json,sys; json.load(open(sys.argv[1]))' "$out" \
      || die "$out is not valid JSON"
  fi
  # Retention: keep the newest ${BACKUP_KEEP:-14} default-named snapshots.
  local keep="${BACKUP_KEEP:-14}"
  if ((keep > 0)); then
    ls -1t "$BACKUP_DIR"/content-planner-*.json 2>/dev/null \
      | tail -n +"$((keep + 1))" | xargs -r rm -f
  fi
  log "Backup written: $out ($(wc -c < "$out" | tr -d ' ') bytes, mode 600)"
  log "Store it off-host; it contains password hashes and unpublished content."
}

cmd_restore() {
  require_docker
  require_env
  local file="${1:-}"
  [[ -n "$file" && -f "$file" ]] || die "usage: ops/scripts/deploy.sh restore <backup.json>"
  if command -v python3 >/dev/null 2>&1; then
    python3 -c 'import json,sys; json.load(open(sys.argv[1]))' "$file" \
      || die "$file is not valid JSON"
  fi
  log "Stopping api…"
  "${COMPOSE[@]}" stop api >/dev/null 2>&1 || true
  # Write through a one-off container so the snapshot is owned by the app user
  # (docker compose cp can leave it unwritable for the API).
  log "Writing snapshot into the data volume…"
  "${COMPOSE[@]}" run --rm --no-deps -T --entrypoint sh api \
    -c 'cat > /app/data/content-planner.json' < "$file"
  "${COMPOSE[@]}" up -d api
  log "Restored $file and started the api container."
}

cmd_update() {
  require_docker
  require_env
  "${COMPOSE[@]}" build --pull
  "${COMPOSE[@]}" up -d
  "${COMPOSE[@]}" ps
}

cmd_smoke() {
  # Usage: ops/scripts/deploy.sh smoke [https://app.example.com] [email password] [--write]
  local url=""
  if [[ "${1:-}" == http* ]]; then
    url="$1"
    shift
  fi
  if [[ -z "$url" ]]; then
    [[ -f "$ENV_FILE" ]] || die "missing $ENV_FILE (or pass a URL: ops/scripts/deploy.sh smoke https://app.example.com)"
    local port site
    port="$(env_value WEB_PORT)"; port="${port:-8080}"
    site="$(env_value SITE_ADDRESS)"
    if [[ -n "$site" && "$site" != :* ]]; then
      url="https://$site"
    elif [[ "$site" == :* ]]; then
      url="http://localhost$site"
    else
      url="http://localhost:${port}"
    fi
  fi
  bash ops/scripts/smoke.sh "$url" "$@"
}

cmd_help() {
  sed -n '2,15p' "$0" | sed 's/^# \{0,1\}//'
}

command="${1:-help}"
shift || true
case "$command" in
  init) cmd_init "$@" ;;
  up) cmd_up "$@" ;;
  down) cmd_down "$@" ;;
  destroy) cmd_destroy "$@" ;;
  ps) require_docker; require_env; "${COMPOSE[@]}" ps "$@" ;;
  logs) require_docker; require_env; "${COMPOSE[@]}" logs -f --tail=200 "$@" ;;
  restart) require_docker; require_env; "${COMPOSE[@]}" restart "$@" ;;
  update) cmd_update "$@" ;;
  backup) cmd_backup "$@" ;;
  restore) cmd_restore "$@" ;;
  smoke) cmd_smoke "$@" ;;
  help | --help | -h) cmd_help ;;
  *) die "unknown command: $command (run ops/scripts/deploy.sh help)" ;;
esac
