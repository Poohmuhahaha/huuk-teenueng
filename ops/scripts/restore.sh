#!/usr/bin/env bash
# Replace the Content Planner store from a backup and restart the service.
#
#   sudo ops/scripts/restore.sh <backup.json> [--env-file FILE] [--no-service]
set -euo pipefail

FILE="${1:-}"
[[ -n "$FILE" ]] || { echo "usage: $0 <backup.json> [--env-file FILE] [--no-service]" >&2; exit 2; }
shift

ENV_FILE="/etc/content-planner.env"
NO_SERVICE=0
while (($#)); do
  case "$1" in
    --env-file) ENV_FILE="${2:?}"; shift 2 ;;
    --no-service) NO_SERVICE=1; shift ;;
    *) echo "unknown option: $1" >&2; exit 2 ;;
  esac
done

die() { printf 'error: %s\n' "$*" >&2; exit 1; }

[[ -f "$FILE" ]] || die "backup not found: $FILE"
[[ -r "$ENV_FILE" ]] || die "cannot read $ENV_FILE (run with sudo or pass --env-file)"
DATA_FILE="$(grep -E '^DATA_FILE=' "$ENV_FILE" | tail -1 | cut -d= -f2- | tr -d '[:space:]')"
[[ -n "$DATA_FILE" ]] || die "DATA_FILE is not set in $ENV_FILE"

if command -v python3 >/dev/null 2>&1; then
  python3 -c 'import json,sys; json.load(open(sys.argv[1]))' "$FILE" \
    || die "$FILE is not valid JSON"
fi

SERVICE_RUNNING=0
if ((!NO_SERVICE)) && [[ -d /run/systemd/system ]] && command -v systemctl >/dev/null 2>&1; then
  systemctl is-active --quiet content-planner && SERVICE_RUNNING=1
  systemctl stop content-planner 2>/dev/null || true
else
  NO_SERVICE=1
fi

install -d -m 0755 "$(dirname "$DATA_FILE")"
cp "$FILE" "$DATA_FILE"
chmod 600 "$DATA_FILE"
if [[ "$(id -u)" -eq 0 ]] && id contentplanner >/dev/null 2>&1; then
  chown contentplanner: "$DATA_FILE"
fi

if ((NO_SERVICE)); then
  printf 'Restored %s -> %s\nStart the service manually to load it.\n' "$FILE" "$DATA_FILE"
else
  systemctl start content-planner
  printf 'Restored %s and restarted content-planner (was running: %s).\n' "$FILE" "$SERVICE_RUNNING"
fi
