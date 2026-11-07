#!/usr/bin/env bash
# Create deploy/.env.vm with secure random admin credentials.
# Run from the repo root on the VM:
#   bash deploy/init-env.sh
set -euo pipefail

cd "$(dirname "$0")/.."

if [[ -f deploy/.env.vm ]]; then
  echo "deploy/.env.vm already exists — move it aside first." >&2
  exit 1
fi

read -r -p "Public URL (e.g. https://beta.example.com): " PUBLIC_URL
read -r -p "Admin email: " ADMIN_EMAIL

if [[ -z "$PUBLIC_URL" || -z "$ADMIN_EMAIL" ]]; then
  echo "Public URL and admin email are required." >&2
  exit 1
fi

PUBLIC_URL="${PUBLIC_URL%/}"
ADMIN_PASSWORD="$(openssl rand -hex 16)"
ADMIN_TOKEN="$(openssl rand -hex 24)"

umask 077
cat > deploy/.env.vm <<ENV
# Generated $(date -u +%Y-%m-%dT%H:%M:%SZ)
ADMIN_EMAIL=$ADMIN_EMAIL
ADMIN_PASSWORD=$ADMIN_PASSWORD
ADMIN_TOKEN=$ADMIN_TOKEN

AUTH_REQUIRED=true
ALLOW_REGISTRATION=false
DEMO_MODE=0

PUBLIC_URL=$PUBLIC_URL
FRONTEND_URL=$PUBLIC_URL
CORS_ORIGINS=$PUBLIC_URL
COOKIE_SECURE=1

HUUK_IMAGE_TAG=local
RUST_LOG=info
ENV
chmod 600 deploy/.env.vm

echo
echo "Created deploy/.env.vm"
echo "Admin email : $ADMIN_EMAIL"
echo "Password    : $ADMIN_PASSWORD"
echo "Token       : $ADMIN_TOKEN"
echo
echo "Save the password and token now. Change the password after first login."
echo "Next: docker compose --env-file deploy/.env.vm -f deploy/docker-compose.vm.yml up -d --build"
