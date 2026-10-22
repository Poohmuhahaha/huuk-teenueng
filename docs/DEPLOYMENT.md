# Deployment guide

Everything needed to run Huuk by teenueng (planner + CMS) in production.
Production domain: `huuk.teenueng.com` (main site: `teenueng.com`).

## Architecture

```
            ┌──────────────────────────────────────────────────────────┐
            │ docker compose (name: content-planner)                   │
            │                                                          │
  :443/:80  │  proxy (Caddy, profile "tls")                            │
 ─────────► │     /api/*  ─────────────► api  (Rust binary, :8787)     │
            │     /*      ─────────────► web  (nginx + built SPA, :8080) │
  :8080     │                              │ /api/* proxied to api      │
 ─────────► │                              ▼                           │
            │                        cp-data volume                    │
            │                        /app/data/content-planner.json   │
            └──────────────────────────────────────────────────────────┘
```

- **api** — the release Rust server. State is a JSON snapshot on the `cp-data` volume
  (atomic writes, flushed every 5s and on shutdown).
- **web** — nginx serving `app/dist` and proxying `/api/` to `api`, so the browser uses a single
  origin (no CORS, no `VITE_API_URL` needed). The SPA uses hash routing, so every path serves
  `index.html`.
- **proxy** — optional Caddy container that terminates TLS; it routes `/api/*` to `api` and
  everything else to `web`. Enable with `ops/scripts/deploy.sh up --tls`.

## Choose your path

| Path | Needs | Best for |
|---|---|---|
| **A. No Docker (systemd)** | Linux + systemd + Rust + Bun/npm | Hosts without Docker; the binary serves API **and** frontend |
| **B. Docker Compose** | Docker Engine + Compose v2 | Hosts with Docker; adds nginx + optional Caddy TLS |
| **C. Podman** | Podman (no compose provider needed) | Docker-compatible hosts (see Option C for direct `podman run`) |

---

## Option A — no Docker (recommended when Docker is unavailable)

The Rust binary can serve the built frontend itself (`STATIC_DIR`), so the whole product is **one
process, one data file, no nginx**. The installer builds everything, creates the service user,
writes a config with generated secrets, and installs a systemd unit.

```sh
git clone <repo> && cd content-planner

sudo ops/scripts/install.sh
#    → builds server + frontend
#    → installs to /opt/content-planner, data in /var/lib/content-planner
#    → writes /etc/content-planner.env (generated Owner password + ADMIN_TOKEN)
#    → enables + starts the content-planner systemd service
#    → prints the bootstrap credentials once

curl -s localhost:8787/api/ready     # {"status":"ready",...}
```

Useful install options:

```sh
sudo ops/scripts/install.sh --host https://huuk.teenueng.com --admin-email you@example.com
sudo ops/scripts/install.sh --port 9000 --prefix /srv/content-planner --data-dir /srv/cp-data
sudo ops/scripts/install.sh --update          # rebuild + restart, keeps config/data (auto pre-update backup)
sudo ops/scripts/install.sh --no-build        # install already-built artifacts
ops/scripts/install.sh --help                 # user-space test install (--skip-service)
```

**TLS.** Install Caddy (single binary/package) and use the provided one-liner config:

```sh
sudo cp ops/deploy/Caddyfile.standalone /etc/caddy/Caddyfile
sudo sed -i 's/huuk.teenueng.com/YOUR.DOMAIN/' /etc/caddy/Caddyfile
sudo systemctl reload caddy
```

Any other reverse proxy works too — it just forwards to `127.0.0.1:8787`.

**Bare-metal day-2:**

```sh
systemctl status content-planner         # health / restarts
journalctl -u content-planner -f         # logs
ops/scripts/backup.sh                        # snapshot + verify + retention (14 by default)
sudo ops/scripts/restore.sh backups/content-planner-20260920-120000.json
sudo ops/scripts/install.sh --update         # deploy a new build
```

Files: binary + frontend in `/opt/content-planner`, data in
`/var/lib/content-planner/store.json`, config in `/etc/content-planner.env` (mode 640,
`root:contentplanner`). The single writer rule still applies — run one instance; move to the
database path for multi-replica (see `server/README.md`).

---

## Option B — Docker Compose

Prerequisites: Docker Engine + Compose v2 (`docker compose version`). For TLS: a DNS record
pointing at the host and ports 80/443 reachable (`SITE_ADDRESS=huuk.teenueng.com`; Caddy issues
Let's Encrypt certificates). Without the TLS profile, forward `${WEB_PORT}` (default 8080) from
your existing load balancer.

## Quick start

```sh
git clone <repo> && cd content-planner

ops/scripts/deploy.sh init          # creates ops/deploy/.env.production + generated secrets (mode 600)
$EDITOR ops/deploy/.env.production
#   ADMIN_EMAIL        owner@yourdomain.com
#   FRONTEND_URL / PUBLIC_URL / CORS_ORIGINS   https://app.yourdomain.com
#   SITE_ADDRESS       app.yourdomain.com        (only for --tls)

ops/scripts/deploy.sh up            # http://localhost:8080
# or:
ops/scripts/deploy.sh up --tls      # Caddy on 80/443 with automatic HTTPS

ops/scripts/deploy.sh smoke         # run the smoke test against the deployment
```

`init` prints the generated Owner password and `ADMIN_TOKEN` once — store them safely and change
the password after the first login (avatar menu → Change password).

The Docker-specific operations are under [Day-2 operations](#day-2-operations); the environment
reference below is shared by all options.

## Option C — Podman (no compose provider)

`podman compose` needs a provider plugin that is not always installed. Direct commands work
without one:

```sh
podman network create cp-net

podman build -t content-planner-api:1 -f server/Dockerfile server
podman build -t content-planner-web:1 -f app/Dockerfile .

podman run -d --name cp-api --network cp-net --network-alias api \
  -v cp-data:/app/data --env-file ops/deploy/.env.production \
  -e BIND_ADDR=0.0.0.0 -e DATA_FILE=/app/data/content-planner.json \
  content-planner-api:1

podman run -d --name cp-web --network cp-net -p 8080:8080 content-planner-web:1
```

The web container reaches the API by the name `api` (`--network-alias`), matching
`ops/nginx/nginx.conf`. Open `http://localhost:8080`. Day-2:

```sh
podman logs -f cp-api
podman exec -T cp-api cat /app/data/content-planner.json > backup.json   # backup
podman stop cp-api && podman start cp-api                                # apply restored file
podman build -t content-planner-api:1 -f server/Dockerfile server && \
  podman rm -f cp-api && podman run -d ...                               # update
```

For TLS, run any proxy in front of `cp-web` (or a Caddy container on `cp-net`); the bundled
compose Caddy profile only applies to Docker Compose.

## Environment variables (`ops/deploy/.env.production`)

| Variable | Purpose |
|---|---|
| `ADMIN_EMAIL`, `ADMIN_PASSWORD` | Bootstrap the first **Owner** account on first boot |
| `ADMIN_TOKEN` | `X-Admin-Token` break-glass recovery (≥16 chars); keep secret |
| `AUTH_REQUIRED` | `true` in production (default when `DEMO_MODE` is unset) |
| `ALLOW_REGISTRATION` | `true` opens self-serve signup on `/register` (SaaS); `false` is invite-only (onboard from Settings) |
| `DEMO_MODE` | Leave unset. `1` seeds demo accounts and opens writes (local only) |
| `FRONTEND_URL`, `PUBLIC_URL`, `CORS_ORIGINS` | Public origin(s); `PUBLIC_URL` is used for OAuth callbacks |
| `WEB_BIND` | Host interface for the web container: `127.0.0.1` (default, this machine only) or `0.0.0.0` for LAN access |
| `SWAGGER_UI` | Leave unset in production (API docs are off in release builds). Set `1` to serve `/api/docs` + `/api/openapi.yaml` |
| `WEB_PORT` | Host port for the web container without TLS (default 8080) |
| `SITE_ADDRESS` | Domain for the Caddy profile; empty = HTTP only |
| `RUST_LOG` | Log level, e.g. `info` |
| `META_APP_ID/SECRET`, `GOOGLE_CLIENT_ID/SECRET`, `TIKTOK_CLIENT_KEY/SECRET` | Optional real OAuth providers |

Provider callbacks must point at `PUBLIC_URL`:
`https://huuk.teenueng.com/api/oauth/{facebook|instagram|youtube|tiktok}/callback`.

## Onboarding a client

1. Sign in as Owner → avatar → **Workspace settings** → **Create sign-in account (for clients)**.
2. Enter name/email, pick **Client**, leave the password empty.
3. Copy the generated temporary password (shown once) and send it to the client.
4. The client signs in and lands in the **Content Studio** (`/#/studio`): they can write, preview,
   publish or schedule, and share `/#/read/{slug}` links. They cannot see the planner or settings.

## Day-2 operations

```sh
ops/scripts/deploy.sh ps                  # container status + health
ops/scripts/deploy.sh logs api            # follow API logs (also: web, proxy)
ops/scripts/deploy.sh restart
ops/scripts/deploy.sh update              # rebuild with fresh base images, roll forward
ops/scripts/deploy.sh backup              # backups/content-planner-<timestamp>.json
ops/scripts/deploy.sh restore backups/... # stop api, swap snapshot, start api
ops/scripts/deploy.sh down                # stop, keep data
ops/scripts/deploy.sh destroy             # stop and delete volumes (asks for confirmation)
```

**Backups.** The snapshot contains password hashes, sessions and unpublished content — store
backups encrypted/off-host. The `backup` command is safe to run live because writes are atomic
(you get a complete old or new file). For extra safety run it after `ops/scripts/deploy.sh down`.

For a daily automatic snapshot, install the provided systemd units (substitute the repo path
and the user that can run docker):

```sh
sed "s|__REPO__|$PWD|; s|__RUN_USER__|$USER|" ops/deploy/systemd/content-planner-backup.service \
  | sudo tee /etc/systemd/system/content-planner-backup.service >/dev/null
sudo cp ops/deploy/systemd/content-planner-backup.timer /etc/systemd/system/
sudo systemctl daemon-reload && sudo systemctl enable --now content-planner-backup.timer
```

**Updates / rollback.** `ops/scripts/deploy.sh update` rebuilds both images and recreates containers; a
final snapshot is flushed on shutdown. To roll back, `git checkout <previous tag>`, then
`ops/scripts/deploy.sh update`; if the snapshot shape ever changes incompatibly, restore the matching
backup with `ops/scripts/deploy.sh restore`.

## Testing production (staging rehearsal)

Run the exact compose stack on a staging host with its own `web`, `ops/deploy/.env.production` (no
`DEMO_MODE`) and volume, then:

```sh
ops/scripts/deploy.sh smoke https://staging.example.com owner@staging.example.com 'password' --write
```

What the smoke test checks:

1. `/api/health` liveness and `/api/ready` readiness (warns if `demoMode` is on or auth is off).
2. Security headers (`x-content-type-options`, CSP, `cache-control`, HSTS) on API responses.
3. **Production safety**: registration returns 403 and the seeded `owner@studio.local /
   demo1234` login is refused.
4. Public content delivery answers with a JSON array.
5. The frontend serves the app shell.
6. With credentials: login + `/api/auth/me` (prints the role).
7. With `--write`: full CMS round-trip — create draft → edit → publish → public read → delete.

Manual spot checks before cutover:

- Sign in as the Owner, confirm nav shows Plan/Calendar/…/Content.
- Sign in as a Client (created in staging), confirm only **Content** is shown and `/studio` works.
- Publish an item and open its `/#/read/{slug}` link in a private window (no session).
- Restart the stack (`ops/scripts/deploy.sh restart`) and confirm content is still there.
- Try 11 wrong passwords — the 11th request should return `429` (login lockout).
- Optionally configure OAuth credentials and complete one provider login end-to-end; confirm the
  callback lands on `/#/profile?oauth=…&status=ok`.

Only after staging is clean should you point production DNS at the host and rotate the bootstrap
password.

## Manual systemd install (without the installer)

`ops/scripts/install.sh` is the supported path, but the pieces are all plain files if you prefer to
wire them yourself: build with `cargo build --release` and `bun run build`, install the binary and
`app/dist`, then use `ops/deploy/systemd/content-planner.service` (substitute `__RUN_USER__`,
`__APP_DIR__`, `__ENV_FILE__`, `__DATA_DIR__`) with an environment file following the table above,
including `STATIC_DIR=<dist path>`. The API serves the SPA itself; no nginx is required.

If you do want nginx instead, `ops/nginx/nginx.conf` is a ready-made server block (replace
`proxy_pass http://api:8787` with `http://127.0.0.1:8787` and leave `STATIC_DIR` unset).

## Troubleshooting

| Symptom | Fix |
|---|---|
| `api` is unhealthy | `ops/scripts/deploy.sh logs api` — usually a bad env file or a corrupt snapshot; the log names the file and exits instead of losing data |
| Port 8080 already in use | Set another `WEB_PORT` in the env file and `ops/scripts/deploy.sh up` |
| TLS profile can't get a certificate | DNS must point at the host and ports 80/443 must be free; check `ops/scripts/deploy.sh logs proxy` |
| "no accounts exist and demo mode is off" warning | Set `ADMIN_EMAIL`/`ADMIN_PASSWORD` and restart, or set `ALLOW_REGISTRATION=true` temporarily |
| Clients see the planner | They must hold the **Client** role (Settings → create account with role Client); roles come from the directory |
| OAuth callback errors | `PUBLIC_URL` must match the provider's registered redirect base; the provider app must allow `PUBLIC_URL/api/oauth/<platform>/callback` |
| Public link 404s | The item must be **published** (not draft/scheduled/future-dated) |
| Bare-metal service won't start | `journalctl -u content-planner -n 50` — a corrupt snapshot or bad `/etc/content-planner.env` is named in the log |
| Page loads but shows no UI (blank) | `STATIC_DIR` must point at the built `dist`; check the startup line `Web : serving …` |
| `deploy.sh` says Docker is missing | Use Option A: `sudo ops/scripts/install.sh` (or follow Option C for Podman) |
| Assets 404 after an update | Re-run the installer so `dist` is copied; the binary serves hashed assets from `STATIC_DIR/assets` |

## Security and scale notes

- Run **one API instance** with this JSON store (Compose, Podman and systemd are all set up that
  way). Multi-replica deployments need the database repository path described in
  `server/README.md`.
- Secrets live only in `ops/deploy/.env.production` (600) or `/etc/content-planner.env` (640) and
  container env; never in `VITE_*`.
- The API enforces Argon2id, login lockout, CSPRNG OAuth state, body limits, request timeouts and
  security headers. Served SPA responses get their own CSP; API responses stay `default-src 'none'`.
  Static serving rejects path traversal and keeps the JSON 404 envelope for `/api/*`.
- Off-host backups are mandatory: the store file/volume is the only copy of content, accounts and
  sessions.
