# Deploy Huuk on AWS Free Plan (EC2 t4g.small) with Docker

This runbook runs the app on an always-on AWS EC2 instance and exposes it with
a free Cloudflare Tunnel. Your own computer can stay off.

- AWS Free Plan: most new customers can sign up without a payment method.
- New accounts get $100 in credits (up to $200 after activities), for **6
  months** or until the credits run out.
- `t4g.small` (2 vCPU / 2 GB RAM, ARM64) has a free trial of 750 hours/month
  through **31 Dec 2026**; after that it can draw from Free Plan credits.
- When the Free Plan ends, export the data volume and move to Oracle/Hetzner.

> If AWS asks for a payment method during signup, you can stop there — do not
> enter a virtual/prepaid card just to bypass it. Ask for the fallback plan.
>
> The Docker setup itself is VM-agnostic: steps 2–9 also work on Hetzner,
> GratisVPS, or any always-on Ubuntu host. Only the signup/instance section is
> AWS-specific.

## Architecture

```
Cloudflare Tunnel ──► app container :8787
                         │  Rust API + Vue SPA
                         └─ /data volume (store.json · images/ · fonts/)
```

## 1. Launch the EC2 instance

AWS Console → EC2 → **Launch instance**:

| Setting | Value |
|---|---|
| Region | Singapore (`ap-southeast-1`) or the nearest region |
| Name | `huuk-beta` |
| AMI | Ubuntu Server 24.04 LTS (64-bit Arm) |
| Instance type | `t4g.small` (2 vCPU, 2 GB) |
| Key pair | Create/download an SSH key |
| Storage | 30 GB gp3 |
| Security group | SSH 22 from **My IP** only (no 80/443) |

The instance needs a public IPv4 for outbound access. AWS bills public IPv4
separately (small hourly charge); it is covered by Free Plan credits while they
last. Set a zero-spend budget alert in Billing.

## 2. Connect and prepare the host

```bash
ssh -i ~/huuk.pem ubuntu@<EC2_PUBLIC_IP>
```

Add swap before the Rust build (2 GB RAM is tight):

```bash
sudo fallocate -l 4G /swapfile
sudo chmod 600 /swapfile
sudo mkswap /swapfile
sudo swapon /swapfile
echo '/swapfile none swap sw 0 0' | sudo tee -a /etc/fstab
free -h
```

Install Docker and basics:

```bash
sudo apt update
sudo apt install -y ca-certificates curl git
curl -fsSL https://get.docker.com | sudo sh
sudo usermod -aG docker "$USER"
newgrp docker
docker version
docker compose version
```

## 3. Clone and configure

Use either the Git repository (if it contains the `deploy/` files) or the
prepared tarball:

```bash
# Option A — from Git
git clone <REPO_URL> ~/huuk

# Option B — from a tarball already copied to the VM
mkdir -p ~/huuk
tar xzf ~/huuk-deploy.tar.gz -C ~/huuk

cd ~/huuk
bash deploy/init-env.sh
```

The script generates `deploy/.env.vm` with random `ADMIN_PASSWORD` and
`ADMIN_TOKEN`, and prints them once. Save both.

If you prefer to edit manually:

```bash
cp deploy/.env.vm.example deploy/.env.vm
chmod 600 deploy/.env.vm
nano deploy/.env.vm
```

Fill at least:

| Variable | Notes |
|---|---|
| `ADMIN_EMAIL` | Owner account created on first boot |
| `ADMIN_PASSWORD` | ≥ 12 characters |
| `ADMIN_TOKEN` | ≥ 32 random characters; also encrypts OAuth tokens |
| `PUBLIC_URL`, `FRONTEND_URL`, `CORS_ORIGINS` | `https://<your-domain>` |
| `COOKIE_SECURE` | `1` behind the HTTPS Tunnel |
| `DEMO_MODE` | `0` |
| `AUTH_REQUIRED` | `true` |
| `ALLOW_REGISTRATION` | `true` for self-serve signup, `false` for invite-only |

Generate secrets if needed:

```bash
openssl rand -hex 16   # ADMIN_PASSWORD (32 chars)
openssl rand -hex 24   # ADMIN_TOKEN    (48 chars)
```

## 4. Build and start

```bash
docker compose --env-file deploy/.env.vm \
  -f deploy/docker-compose.vm.yml up -d --build
```

The first ARM64 Rust build takes roughly 15–30 minutes with swap.

Check the architecture and health:

```bash
docker image inspect huuk:local --format '{{.Architecture}}'   # must be arm64
curl -fsS http://127.0.0.1:8787/api/health
curl -fsS http://127.0.0.1:8787/api/ready
```

Logs:

```bash
docker compose --env-file deploy/.env.vm \
  -f deploy/docker-compose.vm.yml logs -f app
```

## 5. Quick test without a domain (optional)

You can verify the app before setting up DNS with a temporary `trycloudflare.com`
URL:

```bash
docker run -d --name huuk-quicktunnel --network host \
  cloudflare/cloudflared:latest \
  tunnel --no-autoupdate --url http://127.0.0.1:8787
docker logs huuk-quicktunnel 2>&1 | grep -o 'https://[^ ]*trycloudflare.com'
```

Open that URL, log in, and check the app. The URL changes if the container is
recreated — use it for testing only.

Remove it before starting the real HTTPS setup:

```bash
docker rm -f huuk-quicktunnel
```

## 6. HTTPS directly on the VM (Caddy, recommended)

This uses a normal DNS `A` record and Let's Encrypt; no Cloudflare account or
nameserver change is needed. Open ports **80** and **443** in the EC2 security
group first.

1. Add the DNS record at your registrar (Namecheap → Advanced DNS):
   - Type: `A Record`
   - Host: `beta.huuk` (for `beta.huuk.example.com`)
   - Value: the EC2 public IPv4
   - TTL: Automatic
2. Set these in `deploy/.env.vm`:
   ```dotenv
   APP_DOMAIN=beta.huuk.example.com
   ACME_EMAIL=you@example.com
   PUBLIC_URL=https://beta.huuk.example.com
   FRONTEND_URL=https://beta.huuk.example.com
   CORS_ORIGINS=https://beta.huuk.example.com
   COOKIE_SECURE=1
   ```
3. Start the app and Caddy:
   ```bash
   sudo docker compose --env-file deploy/.env.vm \
     -f deploy/docker-compose.vm.yml --profile caddy up -d
   ```
4. Watch the certificate issuance:
   ```bash
   sudo docker compose --env-file deploy/.env.vm \
     -f deploy/docker-compose.vm.yml logs -f caddy
   ```
   Wait for `certificate obtained successfully`, then open
   `https://beta.huuk.example.com`.

Caddy renews certificates automatically and stores them in the
`huuk-vm_caddy-data` volume. Keep port 80 reachable so renewals can complete.
Changing `APP_DOMAIN` requires recreating the Caddy container:

```bash
sudo docker compose --env-file deploy/.env.vm \
  -f deploy/docker-compose.vm.yml --profile caddy up -d --force-recreate caddy
```

## 6b. Cloudflare Tunnel (alternative)

1. Add your domain to Cloudflare (free plan) and change its nameservers at
   Namecheap to the two Cloudflare nameservers. Wait for **Active**.
2. Zero Trust → **Networks → Tunnels → Create a tunnel** → Cloudflared.
3. Copy the connector token into `deploy/.env.vm`:
   ```dotenv
   CLOUDFLARED_TOKEN=eyJhIjoi...
   ```
4. Add a public hostname:
   - Subdomain/domain: e.g. `beta` + `your-domain.com`
   - Service type: **HTTP**
   - URL: `app:8787` (the compose service name)
5. Start the tunnel container:
   ```bash
   docker compose --env-file deploy/.env.vm \
     -f deploy/docker-compose.vm.yml --profile tunnel up -d
   ```

Open `https://beta.<your-domain>`.

> If you enable Cloudflare Access in front of the app, add a bypass policy for
> `/api/oauth/*` so OAuth callbacks are not blocked.

## 7. First login and feedback access

1. Sign in with `ADMIN_EMAIL` / `ADMIN_PASSWORD`.
2. Change the password (avatar menu → Change password).
3. Create the workspace / brand.
4. Invite testers: Owner → Workspace settings → create accounts with temporary
   passwords. New accounts are read-only until a role is assigned.
5. Share a Tally/Google Form for feedback, or temporarily enable registration.

Smoke test from your computer:

```bash
ops/scripts/smoke.sh https://beta.<your-domain> <admin-email> '<password>' --write
```

## 8. Self-serve registration and Google sign-in

### Registration
Set `ALLOW_REGISTRATION=true` in `deploy/.env.vm` and restart the app:

```bash
sudo docker compose --env-file deploy/.env.vm   -f deploy/docker-compose.vm.yml up -d --force-recreate app
```

`/register` and the "Create an account" link become available. New password
accounts get the read-only `Viewer` role; the owner can add them to a workspace
from **Branding → Members**.

### Google sign-in
The app supports Google OpenID Connect login. It uses the same Google OAuth
client as the YouTube data connection:

1. Google Cloud Console → APIs & Services → Credentials → create an
   **OAuth client ID** of type **Web application**.
2. Add the authorized redirect URI:
   `https://<your-domain>/api/auth/oauth/google/callback`
3. Put the credentials in `deploy/.env.vm`:
   ```dotenv
   GOOGLE_CLIENT_ID=...
   GOOGLE_CLIENT_SECRET=...
   ```
4. Restart:
   ```bash
   sudo docker compose --env-file deploy/.env.vm \
     -f deploy/docker-compose.vm.yml up -d --force-recreate app
   ```

The "Continue with Google" button only appears when the credentials are
configured. A stable domain is required — a temporary Quick Tunnel URL changes
and would break the registered redirect URI.

New Google accounts also get `Viewer`; add their email from **Branding →
Members** to let them into the workspace.

## 8b. PostgreSQL container and DB access

This is the low-cost option: PostgreSQL runs as a container on the same EC2
instance. It uses the `huuk-pgdata` Docker volume and is not exposed publicly.

1. Add a password to `deploy/.env.vm`:
   ```dotenv
   POSTGRES_USER=huuk
   POSTGRES_DB=huuk
   POSTGRES_PASSWORD=<long-random-password>
   AUTO_MIGRATE=0
   ```
2. Start PostgreSQL and run the migrations once:
   ```bash
   sudo docker compose --env-file deploy/.env.vm \
     -f deploy/docker-compose.vm.yml up -d postgres

   sudo docker compose --env-file deploy/.env.vm \
     -f deploy/docker-compose.vm.yml run --rm app --run-migrations-only
   ```
3. Start the app. On first boot it loads the existing `/data/store.json` and
   writes it into PostgreSQL:
   ```bash
   sudo docker compose --env-file deploy/.env.vm \
     -f deploy/docker-compose.vm.yml up -d app
   sudo docker compose --env-file deploy/.env.vm \
     -f deploy/docker-compose.vm.yml logs -f app
   ```
4. Verify that the snapshot row exists:
   ```bash
   sudo docker exec huuk-postgres psql -U huuk -d huuk \
     -c "SELECT id, version, updated_at, pg_size_pretty(length(payload::text)::bigint) FROM app_state_snapshots;"
   ```

### Important: how the data is stored

PostgreSQL mode is a **JSON snapshot**, not a normalized relational schema.
The app stores the whole state in:

```text
app_state_snapshots.payload  (JSONB, id = 1)
```

The `users`, `workspaces`, etc. tables in the migrations are not the source of
truth for this server. The runtime snapshot stores **accounts as an array** of
objects inside `payload->'accounts'` (not a map keyed by email). Use the app
UI/API for accounts, members and content.

### Connect with DBeaver / pgAdmin / psql

Port 5432 is bound to `127.0.0.1` only. Create an SSH tunnel from your computer:

```bash
ssh -i /path/to/huuk-key.pem -N \
  -L 5432:127.0.0.1:5432 ubuntu@54.206.23.69
```

Then connect to:

```text
Host: 127.0.0.1
Port: 5432
Database: huuk
User: huuk
Password: value of POSTGRES_PASSWORD in deploy/.env.vm
```

Inspect the snapshot:

```sql
SELECT jsonb_pretty(payload) FROM app_state_snapshots WHERE id = 1;
```

### Editing the database safely

The app persists its in-memory state every 5 seconds. If you edit the database
while the app is running, your change will be overwritten. Always stop the app
first:

```bash
cd ~/huuk

sudo docker exec huuk-postgres pg_dump -U huuk -d huuk -Fc \
  > "$HOME/huuk-db-$(date +%F-%H%M).dump"

sudo docker compose --env-file deploy/.env.vm \
  -f deploy/docker-compose.vm.yml stop app

# edit with psql/DBeaver here, then start it again
sudo docker compose --env-file deploy/.env.vm \
  -f deploy/docker-compose.vm.yml start app
```

Example: change one account's plan. Accounts are an array, so update the
matching element rather than using an email-keyed path:

```sql
UPDATE app_state_snapshots
SET payload = jsonb_set(
  payload,
  '{accounts}',
  (
    SELECT jsonb_agg(
      CASE
        WHEN account->>'email' = 'user@example.com'
          THEN jsonb_set(account, '{plan}', '"starter"')
        ELSE account
      END
    )
    FROM jsonb_array_elements(payload->'accounts') AS account
  )
)
WHERE id = 1;
```

Check the result before restarting:

```sql
SELECT jsonb_pretty(payload->'accounts') FROM app_state_snapshots WHERE id = 1;
```

Do not hand-edit password hashes. Create/reset accounts through the app; use SQL
only for data fixes that the UI does not cover.

### Backup and restore

Back up PostgreSQL:

```bash
sudo docker exec huuk-postgres pg_dump -U huuk -d huuk -Fc \
  > "$HOME/huuk-db-$(date +%F-%H%M).dump"
```

Restore (app stopped):

```bash
sudo docker compose --env-file deploy/.env.vm \
  -f deploy/docker-compose.vm.yml stop app
cat "$HOME/huuk-db-YYYY-MM-DD-HHMM.dump" | \
  sudo docker exec -i huuk-postgres pg_restore -U huuk -d huuk --clean --if-exists
sudo docker compose --env-file deploy/.env.vm \
  -f deploy/docker-compose.vm.yml start app
```

Keep dumps off the instance and private; they contain password hashes and
OAuth tokens.

## 9. Backups

The app data is in the Docker volume `huuk-vm_huuk-data`:

```bash
mkdir -p backups
docker run --rm \
  -v huuk-vm_huuk-data:/data:ro \
  -v "$PWD/backups:/backup" \
  alpine sh -c 'tar czf /backup/huuk-$(date +%F-%H%M).tar.gz -C /data .'
```

Keep an EBS snapshot too (EC2 → Volumes → Create snapshot), and back up
`deploy/.env.vm` (it contains `ADMIN_TOKEN`). The store holds password hashes
and unpublished content — keep backups private. Copy them off the instance
(S3, R2, or your own computer).

Restore:

```bash
docker compose --env-file deploy/.env.vm -f deploy/docker-compose.vm.yml down
docker run --rm -v huuk-vm_huuk-data:/data \
  -v "$PWD/backups:/backup" alpine \
  sh -c 'cd /data && tar xzf /backup/huuk-YYYY-MM-DD-HHMM.tar.gz'
docker compose --env-file deploy/.env.vm -f deploy/docker-compose.vm.yml up -d
```

## 10. Updates and rollback

Use a tag per deploy so rollback is a tag change:

```bash
git pull

TAG=$(date +%F)
HUUK_IMAGE_TAG="$TAG" docker compose --env-file deploy/.env.vm \
  -f deploy/docker-compose.vm.yml build app
HUUK_IMAGE_TAG="$TAG" docker compose --env-file deploy/.env.vm \
  -f deploy/docker-compose.vm.yml up -d

# rollback
HUUK_IMAGE_TAG=2026-11-01 docker compose --env-file deploy/.env.vm \
  -f deploy/docker-compose.vm.yml up -d
```

Check health after every update.

## 11. When the AWS Free Plan ends

The Free Plan closes after 6 months or when credits run out, and the account's
resources are lost. Before that:

1. Stop writes (or accept a final snapshot).
2. Export the `huuk-vm_huuk-data` volume and `deploy/.env.vm`.
3. Create the same stack on Oracle/Hetzner (or another VM).
4. Restore the volume, point the Cloudflare Tunnel at the new host, and verify.

## Troubleshooting

| Symptom | Fix |
|---|---|
| Build killed / OOM | Add more swap (`t4g.small` has 2 GB). Temporarily resize to `t4g.medium` for the build. |
| `exec format error` | Image architecture does not match the instance. Rebuild on the instance (t4g = arm64). |
| `Permission denied` writing `/data` | Use the named volume. For a bind mount, `sudo chown -R 10001:10001 <dir>`. |
| Cloudflare 502 | Tunnel service URL must be `app:8787`. Check `curl /api/health`. |
| No account / login fails | `ADMIN_*` were missing on first boot. Set them and recreate the volume only for a fresh instance. |
| AWS signup asks for a card | Stop and use a fallback: a debit card for Oracle, or a small paid VM. Do not use virtual/prepaid cards. |
