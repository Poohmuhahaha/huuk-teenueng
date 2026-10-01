# Ops — deploy, install, back up, verify

Everything needed to run Content Planner in production. Entry points stay at
the repo root (`dev.sh` for development, `deploy.sh` for Docker Compose);
this folder holds what they act on.

| Path | What it is |
|---|---|
| `docker-compose.yml` | Production stack: `api` (Rust) + `web` (nginx serving the built frontend) + optional Caddy TLS profile. Run from the repo root via `./deploy.sh` (it passes `-f ops/docker-compose.yml` for you) |
| `deploy/.env.production` | Live secrets + public URLs (git-ignored, mode 600 — create with `./deploy.sh init`) |
| `deploy/.env.production.example` | Template with generated `__ADMIN_PASSWORD__` / `__ADMIN_TOKEN__` placeholders |
| `deploy/Caddyfile` / `Caddyfile.standalone` | Caddy front for the compose TLS profile / one-line example for bare-metal installs |
| `deploy/systemd/` | `content-planner.service` (binary + `STATIC_DIR`), backup service + timer (edit the `__REPO__`/`__RUN_USER__` placeholders per `DEPLOYMENT.md`) |
| `deploy/postgres/` | Reserved for the compose Postgres volume (git-ignored) |
| `scripts/install.sh` | Docker-free install: build (`server` + `app`), install, systemd unit — `sudo ops/scripts/install.sh` |
| `scripts/backup.sh` / `restore.sh` | Snapshot / replace the store for systemd installs (`DATA_FILE` from `/etc/content-planner.env`) |
| `scripts/smoke.sh` | Post-deploy verification (`./deploy.sh smoke [url] …`) |

Full guide: [`../DEPLOYMENT.md`](../DEPLOYMENT.md). Day-to-day commands:

```sh
./deploy.sh init          # create ops/deploy/.env.production
./deploy.sh up            # build + start
./deploy.sh backup        # snapshot -> backups/
sudo ops/scripts/install.sh --help   # Docker-free path
```
