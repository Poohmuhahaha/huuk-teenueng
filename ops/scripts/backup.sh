#!/usr/bin/env bash
# Snapshot the Content Planner store (systemd/bare-metal installs).
#
#   ops/scripts/backup.sh [--env-file FILE] [--out DIR] [--keep N]
#
# The store holds password hashes, sessions and unpublished content — keep the
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
DATA_FILE="$(grep -E '^DATA_FILE=' "$ENV_FILE" | tail -1 | cut -d= -f2- | tr -d '[:space:]')"
[[ -n "$DATA_FILE" ]] || die "DATA_FILE is not set in $ENV_FILE"
[[ -f "$DATA_FILE" ]] || die "store not found: $DATA_FILE"

OUT="${OUT:-$(dirname "$DATA_FILE")/backups}"
mkdir -p "$OUT"
chmod 700 "$OUT" 2>/dev/null || true

dest="$OUT/content-planner-$(date +%Y%m%d-%H%M%S).json"
cp "$DATA_FILE" "$dest"
chmod 600 "$dest"
# Verify we copied a complete JSON document (the store writes atomically, so
# this should always pass; it catches operator mistakes).
if command -v python3 >/dev/null 2>&1; then
  python3 -c 'import json,sys; json.load(open(sys.argv[1]))' "$dest" \
    || die "backup is not valid JSON — is $DATA_FILE a Content Planner store?"
fi

# Retention: keep the newest $KEEP files.
if ((KEEP > 0)); then
  ls -1t "$OUT"/content-planner-*.json 2>/dev/null | tail -n +"$((KEEP + 1))" | xargs -r rm -f
fi

printf 'Backup: %s (%s bytes)\n' "$dest" "$(wc -c < "$dest" | tr -d ' ')"
printf 'Store this off-host — it contains password hashes and unpublished content.\n'
