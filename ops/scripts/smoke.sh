#!/usr/bin/env bash
# Production / staging smoke test for a Content Planner deployment.
#
#   ops/ops/scripts/smoke.sh <base-url> [admin-email admin-password] [--write]
#
# Without credentials only public/prod-safety checks run. With credentials the
# login + profile path is verified; --write additionally exercises the full CMS
# round-trip (create draft → edit → publish → public read → delete).
#
# Exit code 0 = all checks passed (warnings allowed); 1 = at least one failure.
set -uo pipefail

BASE="${1:-}"
if [[ -z "$BASE" ]]; then
  echo "usage: $0 <base-url> [admin-email admin-password] [--write]" >&2
  exit 2
fi
shift

ADMIN_EMAIL=""
ADMIN_PASSWORD=""
WRITE_TEST=0
while (($#)); do
  case "$1" in
    --write) WRITE_TEST=1 ;;
    *)
      if [[ -z "$ADMIN_EMAIL" ]]; then ADMIN_EMAIL="$1"
      elif [[ -z "$ADMIN_PASSWORD" ]]; then ADMIN_PASSWORD="$1"
      else echo "unexpected argument: $1" >&2; exit 2
      fi
      ;;
  esac
  shift
done
BASE="${BASE%/}"

command -v curl >/dev/null 2>&1 || { echo "curl is required" >&2; exit 2; }
command -v python3 >/dev/null 2>&1 || { echo "python3 is required" >&2; exit 2; }

PASS=0
FAIL=0
WARN=0
ok()   { PASS=$((PASS + 1)); printf '  ok    %s\n' "$1"; }
bad()  { FAIL=$((FAIL + 1)); printf '  FAIL  %s\n' "$1"; }
warn() { WARN=$((WARN + 1)); printf '  warn  %s\n' "$1"; }

code() { curl -sS -o /dev/null -w '%{http_code}' "$@" 2>/dev/null || echo "000"; }
body() { curl -sS "$@" 2>/dev/null; }
# json '<python-expression over d>'  (reads stdin)
json() { python3 -c "import json,sys
try:
    d = json.load(sys.stdin)
except Exception:
    print('')
    raise SystemExit
print($1)" 2>/dev/null; }

echo "Smoke test: $BASE"
echo

echo "[1] Liveness and readiness"
if [[ "$(code "$BASE/api/health")" == "200" && "$(body "$BASE/api/health" | json "d.get('status','')")" == "ok" ]]; then
  ok "GET /api/health is 200 and status=ok"
else
  bad "GET /api/health did not return {status: ok}"
fi
READY="$(body "$BASE/api/ready")"
if [[ "$(echo "$READY" | json "d.get('status','')")" == "ready" ]]; then
  ok "GET /api/ready is ready (version $(echo "$READY" | json "d.get('version','?')"))"
  if [[ "$(echo "$READY" | json "str(d.get('demoMode'))")" == "True" ]]; then
    warn "demoMode=true — production should run with DEMO_MODE unset"
  fi
  if [[ "$(echo "$READY" | json "str(d.get('authRequired'))")" != "True" ]]; then
    warn "authRequired=false — anonymous writes are open (production should require auth)"
  fi
else
  bad "GET /api/ready did not report status=ready"
fi

echo
echo "[2] Security headers on API responses"
HEADERS="$(curl -sSI "$BASE/api/setup" 2>/dev/null || true)"
check_header() {
  if grep -qi "^$1:" <<<"$HEADERS"; then ok "header $1 present"; else warn "header $1 missing"; fi
}
check_header "x-content-type-options"
check_header "content-security-policy"
check_header "cache-control"
check_header "strict-transport-security"

echo
echo "[3] Production safety"
SMOKE_EMAIL="smoke-$(date +%s)@example.com"
REG_BODY="$(body -X POST "$BASE/api/auth/register" -H 'content-type: application/json' \
  -d "{\"name\":\"Smoke Test\",\"email\":\"$SMOKE_EMAIL\",\"password\":\"password12345\"}" -w '\n%{http_code}')"
REG_CODE="$(tail -n1 <<<"$REG_BODY")"
REG_JSON="$(sed '$d' <<<"$REG_BODY")"
case "$REG_CODE" in
  403) ok "public registration is disabled (403)" ;;
  200)
    ok "public registration is open — SaaS signup enabled (200)"
    REG_TOKEN="$(echo "$REG_JSON" | json "d.get('token','')")"
    if [[ -n "$REG_TOKEN" ]]; then
      PLAN="$(body -X POST "$BASE/api/auth/plan" -H 'content-type: application/json' \
        -H "Authorization: Bearer $REG_TOKEN" -d '{"plan":"free"}')"
      if [[ "$(echo "$PLAN" | json "d.get('user',{}).get('plan','')")" == "free" ]]; then
        ok "new account continues with the free package"
      else
        warn "plan selection did not return plan=free"
      fi
      # Clean up the throwaway signup so open registration does not litter prod.
      if [[ -n "$ADMIN_EMAIL" && -n "$ADMIN_PASSWORD" ]]; then
        SMOKE_ADMIN_TOKEN="$(body -X POST "$BASE/api/auth/login" -H 'content-type: application/json' \
          -d "{\"email\":\"$ADMIN_EMAIL\",\"password\":\"$ADMIN_PASSWORD\"}" | json "d.get('token','')")"
        if [[ -n "$SMOKE_ADMIN_TOKEN" ]]; then
          CLEAN_CODE="$(code -X DELETE "$BASE/api/auth/accounts/$SMOKE_EMAIL" -H "Authorization: Bearer $SMOKE_ADMIN_TOKEN")"
          if [[ "$CLEAN_CODE" == "200" ]]; then ok "smoke signup removed from the account list"; else warn "could not remove smoke signup ($CLEAN_CODE)"; fi
        fi
      fi
    else
      warn "registration response had no token"
    fi
    ;;
  *)   warn "registration check returned an unexpected $REG_CODE" ;;
esac

DEMO_CODE="$(code -X POST "$BASE/api/auth/login" -H 'content-type: application/json' \
  -d '{"email":"owner@studio.local","password":"demo1234"}')"
case "$DEMO_CODE" in
  401) ok "seeded demo credentials are refused (401)" ;;
  200) bad "DEMO ACCOUNT LOGIN SUCCEEDED — run with DEMO_MODE unset" ;;
  *)   warn "demo-login check returned an unexpected $DEMO_CODE" ;;
esac

echo
echo "[4] Public content delivery"
PUB_CODE="$(code "$BASE/api/public/content")"
if [[ "$PUB_CODE" == "200" ]]; then
  COUNT="$(body "$BASE/api/public/content" | json "len(d) if isinstance(d, list) else -1")"
  if [[ "$COUNT" =~ ^[0-9]+$ ]]; then
    ok "GET /api/public/content is 200 ($COUNT published item(s))"
  else
    bad "GET /api/public/content did not return a JSON array"
  fi
else
  bad "GET /api/public/content returned $PUB_CODE"
fi

echo
echo "[5] Frontend"
INDEX="$(body "$BASE/")"
if [[ "$(code "$BASE/")" == "200" && "$INDEX" == *'<div id="app">'* ]]; then
  ok "app shell is served"
else
  bad "the frontend did not serve the app shell"
fi

if [[ -n "$ADMIN_EMAIL" && -n "$ADMIN_PASSWORD" ]]; then
  echo
  echo "[6] Admin login"
  LOGIN="$(body -X POST "$BASE/api/auth/login" -H 'content-type: application/json' \
    -d "{\"email\":\"$ADMIN_EMAIL\",\"password\":\"$ADMIN_PASSWORD\"}")"
  TOKEN="$(echo "$LOGIN" | json "d.get('token','')")"
  if [[ -n "$TOKEN" ]]; then
    ok "login succeeded"
    ME_CODE="$(code "$BASE/api/auth/me" -H "authorization: Bearer $TOKEN")"
    if [[ "$ME_CODE" == "200" ]]; then
      ROLE="$(body "$BASE/api/auth/me" -H "authorization: Bearer $TOKEN" | json "d.get('user',{}).get('role','?')")"
      ok "GET /api/auth/me is 200 (role: $ROLE)"
    else
      bad "GET /api/auth/me returned $ME_CODE with a fresh token"
    fi

    if ((WRITE_TEST)); then
      echo
      echo "[7] CMS round-trip (--write)"
      TITLE="Smoke test $(date +%s)"
      CREATED="$(body -X POST "$BASE/api/content" -H "authorization: Bearer $TOKEN" \
        -H 'content-type: application/json' -d "{\"title\":\"$TITLE\",\"kind\":\"note\"}")"
      ID="$(echo "$CREATED" | json "d.get('id','')")"
      SLUG="$(echo "$CREATED" | json "d.get('slug','')")"
      VERSION="$(echo "$CREATED" | json "d.get('version','')")"
      if [[ -n "$ID" ]]; then
        ok "draft created ($ID)"
        UPDATED="$(body -X PATCH "$BASE/api/content/$ID" -H "authorization: Bearer $TOKEN" \
          -H 'content-type: application/json' \
          -d "{\"version\":$VERSION,\"body\":\"# Smoke\\n\\nChecked by ops/scripts/smoke.sh.\"}")"
        VERSION="$(echo "$UPDATED" | json "d.get('version','')")"
        [[ -n "$VERSION" ]] && ok "draft edited (version $VERSION)" || bad "edit failed"
        PUBLISHED="$(body -X POST "$BASE/api/content/$ID/publish" -H "authorization: Bearer $TOKEN" \
          -H 'content-type: application/json' -d "{\"version\":$VERSION}")"
        if [[ "$(echo "$PUBLISHED" | json "d.get('status','')")" == "published" ]]; then
          ok "draft published"
          PUBLIC_TITLE="$(body "$BASE/api/public/content/$SLUG" | json "d.get('title','')")"
          [[ "$PUBLIC_TITLE" == "$TITLE" ]] && ok "public delivery serves it" || bad "public delivery did not serve the item"
        else
          bad "publish failed"
        fi
        DEL_CODE="$(code -X DELETE "$BASE/api/content/$ID" -H "authorization: Bearer $TOKEN")"
        [[ "$DEL_CODE" == "200" ]] && ok "test item deleted" || bad "cleanup delete returned $DEL_CODE"
      else
        bad "could not create a draft (is content.write granted?)"
      fi
    fi
  else
    bad "login failed — check the credentials and ADMIN_EMAIL/ADMIN_PASSWORD bootstrap"
  fi
fi

echo
printf 'Result: %d passed, %d failed, %d warning(s)\n' "$PASS" "$FAIL" "$WARN"
((FAIL == 0))
