# Ops — deploy, install, back up, verify

Everything needed to run Content Planner in production. Entry-point scripts live
in `scripts/` (`dev.sh` for development, `deploy.sh` for Docker Compose); this
folder holds what they act on.

| Path | What it is |
|---|---|
| `docker-compose.yml` | Production stack: `api` (Rust) + `web` (nginx serving the built frontend) + optional Caddy TLS profile. Driven by `ops/scripts/deploy.sh` (it passes `-f ops/docker-compose.yml` for you) |
| `nginx/` | `nginx.conf` + `nginx-api-proxy.inc` for the `web` image (copied in at image build from the repo-root context) |
| `deploy/.env.production` | Live secrets + public URLs (git-ignored, mode 600 — create with `ops/scripts/deploy.sh init`) |
| `deploy/.env.production.example` | Template with generated `__ADMIN_PASSWORD__` / `__ADMIN_TOKEN__` placeholders |
| `deploy/Caddyfile` / `Caddyfile.standalone` | Caddy front for the compose TLS profile / one-line example for bare-metal installs |
| `deploy/systemd/` | `content-planner.service` (binary + `STATIC_DIR`), backup service + timer (edit the `__REPO__`/`__RUN_USER__` placeholders per `docs/DEPLOYMENT.md`) |
| `deploy/postgres/` | Reserved for the compose Postgres volume (git-ignored) |
| `scripts/install.sh` | Docker-free install: build (`server` + `app`), install, systemd unit — `sudo ops/scripts/install.sh` |
| `scripts/dev.sh` | Dev entry point: Rust backend + Vite frontend (`ops/scripts/dev.sh [--mock|--server]`) |
| `scripts/deploy.sh` | Docker Compose entry point (`ops/scripts/deploy.sh init|up|down|…`) |
| `scripts/backup.sh` / `restore.sh` | Snapshot / replace the store for systemd installs (`DATA_FILE` from `/etc/content-planner.env`) |
| `scripts/smoke.sh` | Post-deploy verification (`ops/scripts/deploy.sh smoke [url] …`) |

Full guide: [`../docs/DEPLOYMENT.md`](../docs/DEPLOYMENT.md). Day-to-day commands:

```sh
ops/scripts/deploy.sh init          # create ops/deploy/.env.production
ops/scripts/deploy.sh up            # build + start
ops/scripts/deploy.sh backup        # snapshot -> backups/
sudo ops/scripts/install.sh --help  # Docker-free path
```
