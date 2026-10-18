#!/usr/bin/env bash
# Snapshot the Content Planner PostgreSQL database and/or JSON store.
#
#   ops/scripts/backup.sh [--env-file FILE] [--out DIR] [--keep N]
#
# The backup holds password hashes, sessions and unpublished content — keep the
# backups and their directory private (this script uses mode 700/600).
set -euo pipefail

ENV_FILE="/etc/content-planner.env"
OUT=""
KEEP=14

while (($#)); do
  case "$1" in
    --env-file) ENV_FILE="${2:?}"; shift 2 ;;
    --out) OUT="${2:?}"; shift 2 ;;
    --keep) KEEP="${2:?}"; shift 2 ;;
    -h | --help) sed -n '2,8p' "$0" | sed 's/^# \{0,1\}//'; exit 0 ;;
    *) echo "unknown option: $1" >&2; exit 2 ;;
  esac
done

die() { printf 'error: %s\n' "$*" >&2; exit 1; }

[[ -r "$ENV_FILE" ]] || die "cannot read $ENV_FILE (run with sudo or pass --env-file)"
DATA_FILE="$(grep -E '^DATA_FILE=' "$ENV_FILE" | tail -1 | cut -d= -f2- | tr -d '[:space:]' || true)"
DATABASE_URL="$(grep -E '^DATABASE_URL=' "$ENV_FILE" | tail -1 | cut -d= -f2- | tr -d '[:space:]' || true)"

if [[ -z "$DATA_FILE" && -z "$DATABASE_URL" ]]; then
  die "Neither DATA_FILE nor DATABASE_URL is set in $ENV_FILE"
fi

if [[ -n "$DATA_FILE" ]]; then
  OUT="${OUT:-$(dirname "$DATA_FILE")/backups}"
else
  OUT="${OUT:-/var/backups/content-planner}"
fi
mkdir -p "$OUT"
chmod 700 "$OUT" 2>/dev/null || true

STAMP="$(date +%Y%m%d-%H%M%S)"

if [[ -n "$DATABASE_URL" ]] && command -v pg_dump >/dev/null 2>&1; then
  pg_dest="$OUT/content-planner-pg-${STAMP}.dump"
  pg_dump --format=custom --file="$pg_dest" "$DATABASE_URL"
  chmod 600 "$pg_dest"
  if ((KEEP > 0)); then
    ls -1t "$OUT"/content-planner-pg-*.dump 2>/dev/null | tail -n +"$((KEEP + 1))" | xargs -r rm -f
  fi
  printf 'PostgreSQL Backup: %s (%s bytes)\n' "$pg_dest" "$(wc -c < "$pg_dest" | tr -d ' ')"
fi

if [[ -n "$DATA_FILE" && -f "$DATA_FILE" ]]; then
  dest="$OUT/content-planner-${STAMP}.json"
  cp "$DATA_FILE" "$dest"
  chmod 600 "$dest"
  if command -v python3 >/dev/null 2>&1; then
    python3 -c 'import json,sys; json.load(open(sys.argv[1]))' "$dest" \
      || die "backup is not valid JSON — is $DATA_FILE a Content Planner store?"
  fi
  if ((KEEP > 0)); then
    ls -1t "$OUT"/content-planner-*.json 2>/dev/null | tail -n +"$((KEEP + 1))" | xargs -r rm -f
  fi
  printf 'Backup: %s (%s bytes)\n' "$dest" "$(wc -c < "$dest" | tr -d ' ')"
fi

printf 'Store this off-host — it contains password hashes and unpublished content.\n'
