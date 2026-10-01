# Content Planner — Rust backend

REST API for the Content Planner wireframe app (`../../archive/wireframe`). Built with **Axum 0.8 + Tokio**,
in-memory state with atomic JSON snapshots, JSON with camelCase fields — the exact contract the
frontend's mock API exposes, so the app can switch between mock and backend with one env var.

Production-ready defaults: demo mode is explicit (`DEMO_MODE=1`), auth is required otherwise,
passwords are Argon2id with login rate-limiting, every input is validated, request size/timeouts
and security headers are enforced, CORS is configurable, and state survives restarts via
`DATA_FILE`.

```
server/
├─ Cargo.toml
├─ src/
│  ├─ main.rs      # binary: config from env, persistence loop, graceful shutdown
│  ├─ lib.rs       # Router (all routes), layered middleware, AppState, new_state()
│  ├─ model.rs     # domain types + partial-update payloads (camelCase serde)
│  ├─ store.rs     # Store + seed data + snapshots + id/session/password helpers
│  ├─ handlers.rs  # one handler per endpoint (validation + permission gates)
│  ├─ oauth.rs     # credential-optional OAuth 2.0 authorization-code flow
│  └─ error.rs     # ApiError + JSON-aware extractors → uniform {"error": …}
└─ tests/api.rs    # 59 integration tests (tower::ServiceExt oneshot)
```

## Quick start

From the repository root, one command runs the backend **and** the frontend wired to it:

```sh
bun run dev               # or: npm run dev / bash dev.sh   (Ctrl+C stops both)
bun run dev:server        # backend only
bun run test              # Rust + frontend tests
```

Or work on the server directly — `DEMO_MODE=1` keeps the local wireframe behavior
(seeded demo accounts, auth optional, open registration), which is what `dev.sh` sets:

```sh
cd server
DEMO_MODE=1 cargo run     # http://127.0.0.1:8787  (override with PORT/BIND_ADDR)
cargo test                # 59 integration tests
cargo build --release     # production binary in target/release/
```

Without `DEMO_MODE`, the server boots in production mode: `authRequired` is on, the well-known
demo accounts are removed, registration is closed, and you bootstrap the first Owner with
`ADMIN_EMAIL` + `ADMIN_PASSWORD` (see Going to production).

Smoke test:

```sh
curl http://localhost:8787/api/health
# {"service":"content-planner-server","status":"ok"}
curl "http://localhost:8787/api/posts?month=2"
```

## API documentation (Swagger UI)

The server ships its own [Swagger UI](https://swagger.io/tools/swagger-ui/) (assets vendored in
`server/swagger/`, so it works offline):

- **`GET /api/docs`** — interactive docs, "Try it out" enabled, Bearer auth persisted.
- **`GET /api/openapi.yaml`** — the OpenAPI 3.0 document. This is the **source of truth** for
  request/response shapes, permission requirements (`x-permission`) and error codes, including
  the CMS content API (`/api/content/*`, `/api/public/content/*`) and all 13 permission strings.

Update `openapi.yaml` in the same change that adds or edits an endpoint; `tests/api.rs` verifies
that `/api/docs` and the spec are served.

## Connect the frontend

The frontend picks its data source in `src/api/index.ts`:

- **no env var** → in-memory mock (`../production/src/mock/api.ts`), no server needed
- **`VITE_API_URL` set** → HTTP client (`../production/src/api/http.ts`) → this server

```sh
# app/.env.local
VITE_API_URL=http://localhost:8787
```

Then `npm run dev` (restart Vite after adding the env var) and every page reads/writes through
the Rust API. Both modules implement the same functions, so no page or component changes are
needed — this is the "swap point" described in `../production/docs/GUIDES.md`.

> CORS follows `CORS_ORIGINS` (comma-separated allow-list). In demo mode it stays permissive;
> in production it defaults to `FRONTEND_URL` + the local Vite origins, so set
> `CORS_ORIGINS=https://app.example.com` when the frontend is hosted elsewhere.

## API reference

All bodies are JSON; field names are camelCase. Errors return
`{ "error": "message" }` with `404` (missing id), `400` (invalid payload),
`401` (missing/invalid/expired token), `403` (role lacks the permission),
`409` (locked by another user, email already registered, or display name already taken).

| Method | Path | Body / query | Returns |
|---|---|---|---|
| GET | `/api/health` | — | `{ status, service }` |
| GET | `/api/setup` | — | `SetupConfig` |
| PATCH | `/api/setup` | partial `SetupConfig` (`language`, `year`, `owner`, `showEditableColors`, `authRequired`, `roles`) | `SetupConfig` (400 unknown permission / empty role name; a `roles` replacement must be non-empty, duplicate-free, and keep every role a directory user holds) |
| POST | `/api/setup/options` | `{ list, item }` — list ∈ pillars/formats/goals/statuses/platforms | `SetupConfig` |
| POST | `/api/setup/users` | `{ name, role? }` — role defaults to Editor | `SetupConfig` (400 on empty/duplicate name) |
| DELETE | `/api/setup/users/{name}` | — | `SetupConfig` (400 on the last user) |
| POST | `/api/setup/roles` | `{ name, permissions? }` — permissions defaults to `[]` | `SetupConfig` (400 empty/duplicate name or unknown permission) |
| DELETE | `/api/setup/roles/{name}` | — | `SetupConfig` (404 unknown; 400 last role or role in use) |
| POST | `/api/auth/register` | `{ name, email, password }` — password ≥ 8 chars, display names unique | `{ token, user }` (400 empty name / email without `@` / short password; 409 email taken, or `name already taken`) |
| POST | `/api/auth/login` | `{ email, password }` | `{ token, user }` (401 `invalid email or password` for unknown email and wrong password alike) |
| GET | `/api/auth/me` | `Authorization: Bearer <token>` | `{ user }` (401 missing/expired/unknown token) |
| POST | `/api/auth/logout` | `Authorization: Bearer <token>` | `{ ok: true }` (401 invalid token) |
| POST | `/api/auth/logout-all` | `Authorization: Bearer <token>` | `{ ok: true }` — revokes every session of the account, including the caller's |
| POST | `/api/auth/change-password` | `{ oldPassword, newPassword }` + Bearer | `{ ok: true }` (400 new password < 8 chars; 401 wrong old password) |
| PATCH | `/api/auth/profile` | `{ name }` + Bearer | `{ user }` — own display name (401 without a session even in demo mode; 400 empty; 409 `name already taken`; renames the directory entry so the role survives) |
| GET | `/api/auth/accounts` | — | `[{ name, email, sessions }]` (never exposes hashes; needs `users.manage` when auth is on) |
| DELETE | `/api/auth/accounts/{email}` | — | `{ ok: true }` — deletes the account and revokes its sessions (404 unknown; needs `users.manage`) |
| GET | `/api/posts?month=2` | `month` (1–12) | `Post[]` |
| POST | `/api/posts` | `Post` without `id` | `Post` (server assigns `id`; any client `lockedBy` is ignored) |
| PATCH | `/api/posts/{id}` | partial `Post` (+ `user` in demo mode) | `Post` (409 if locked by another user; with auth on the lock check uses the session's account, not `user`) |
| POST | `/api/posts/{id}/lock` | `{ user }` (demo mode only) | `Post` (409 if locked by another user; with auth on the holder is the signed-in account) |
| POST | `/api/posts/{id}/unlock` | `{ user }` (demo mode only) | `Post` (409 if locked by another user; with auth on the holder is the signed-in account) |
| GET | `/api/ideas` | — | `Idea[]` |
| POST | `/api/ideas` | `Idea` without `id` | `Idea` |
| POST | `/api/ideas/{id}/toggle` | — | `Idea` |
| POST | `/api/ideas/{id}/promote?month=4` | `month` | `Post` (idea marked done) |
| GET | `/api/tags` | — | `HashtagGroup[]` |
| POST | `/api/tags/{id}` | `{ tag }` | `HashtagGroup` |
| GET | `/api/metrics` | — | `Metric[]` |
| POST | `/api/metrics/import` | `{ platform, month }` | `{ imported, metrics }` (deterministic pseudo-metrics) |
| GET | `/api/txns` | — | `Txn[]` |
| POST | `/api/txns` | `Txn` without `id` (amount > 0) | `Txn` |
| GET | `/api/brand` | — | `Brand` |
| PATCH | `/api/brand` | partial `Brand` (includes `logos` ≤3 and `moodboard` ≤12 image URLs) | `Brand` |
| POST | `/api/brand/images` | `{ name, data, kind }` — base64 PNG/JPG/WebP/GIF, ≤4 MB; `kind` = logo or moodboard (both 32×32–500×500 px) | `{ url, width, height }` — store the url in `brand.logos` / `brand.moodboard` |
| GET | `/api/brand/images/{name}` | — | the image bytes (public, cacheable) |
| DELETE | `/api/brand/images/{name}` | — | `{ ok: true }` |
| GET | `/api/platforms` | — | `PlatformConnection[]` |
| POST | `/api/platforms/{id}/connect` | — | `PlatformConnection` (status connected) |
| POST | `/api/platforms/{id}/disconnect` | — | `PlatformConnection` (status disconnected) |
| POST | `/api/platforms/{id}/sync` | — | `PlatformConnection` (Meta: real Graph refresh of the live mirror; others: lastSync bumped) |
| GET | `/api/live` | — | `LiveData` — the active workspace's mirror of the connected platforms' real content |
| GET | `/api/ads` | — | `AdsView` — Meta Ads mirror (accounts/campaigns/ad sets/ads/insights/audit) + `canManage` |
| POST | `/api/ads/sync` | — | `AdsView` — real Marketing API refresh (400 reconnect/not connected) |
| POST | `/api/ads/manage` | `{ enabled }` | `{ canManage }` — the workspace's management opt-in |
| POST | `/api/ads/campaigns/{id}/status` | `{ status }` ∈ ACTIVE/PAUSED/ARCHIVED | `{ ok }` (400 without the opt-in or for unknown campaigns) |
| POST | `/api/ads/campaigns/{id}/budget` | `{ dailyBudget?, lifetimeBudget? }` (minor units) | `{ ok }` |
| POST | `/api/ads/campaigns/{id}/duplicate` | — | `{ campaignId }` — deep copy, paused |
| POST | `/api/ads/boost` | `{ name, objective, dailyBudget, days, countries[], storyId }` | `{ campaignId }` — paused campaign promoting a Page post |
| GET | `/api/oauth/{platform}` | — | `{platform, mode: "mock"\|"redirect", configured, authorizeHost}` |
| GET | `/api/oauth/{platform}/start` | — | `{mode}` or `{mode:"redirect", url}` — creates the CSRF `state`; needs `platforms.manage` when auth is on |
| GET | `/api/oauth/{platform}/callback` | provider redirect (`code`, `state`) | `303` back to the frontend (`#/profile?oauth=…&status=ok\|error`) |

Examples:

```sh
# add a pillar (duplicates are ignored)
curl -X POST localhost:8787/api/setup/options \
  -H 'content-type: application/json' -d '{"list":"pillars","item":"Pillar IV"}'

# advance a post
curl -X PATCH localhost:8787/api/posts/p-desk \
  -H 'content-type: application/json' -d '{"status":"Done","done":true,"user":"Studio Owner"}'

# add a team member, then remove them
curl -X POST localhost:8787/api/setup/users \
  -H 'content-type: application/json' -d '{"name":"Editor Earn","role":"Editor"}'
curl -X DELETE "localhost:8787/api/setup/users/Editor%20Earn"

# register (role comes from the directory, or Viewer) and auto-login
curl -X POST localhost:8787/api/auth/register \
  -H 'content-type: application/json' \
  -d '{"name":"Editor Earn","email":"earn@studio.local","password":"longenough1"}'
# → {"token":"<64 hex chars>","user":{"name":"Editor Earn","role":"Editor"}}

# log in with the seeded demo credentials, check the session, log out
curl -X POST localhost:8787/api/auth/login \
  -H 'content-type: application/json' \
  -d '{"email":"owner@studio.local","password":"demo1234"}'
# → {"token":"<64 hex chars>","user":{"name":"Studio Owner","role":"Owner"}}
curl localhost:8787/api/auth/me -H 'authorization: Bearer <token>'
curl -X POST localhost:8787/api/auth/logout -H 'authorization: Bearer <token>'

# change the password (revokes every other session of that account)
curl -X POST localhost:8787/api/auth/change-password -H 'authorization: Bearer <token>' \
  -H 'content-type: application/json' \
  -d '{"oldPassword":"demo1234","newPassword":"newpassword1"}'

# sign out everywhere, list accounts (users.manage when auth is on), delete an account
curl -X POST localhost:8787/api/auth/logout-all -H 'authorization: Bearer <token>'
curl localhost:8787/api/auth/accounts -H 'authorization: Bearer <token>'
curl -X DELETE localhost:8787/api/auth/accounts/editor@studio.local -H 'authorization: Bearer <token>'

# lock a post while editing it, then release it
curl -X POST localhost:8787/api/posts/p-desk/lock \
  -H 'content-type: application/json' -d '{"user":"Studio Owner"}'
curl -X POST localhost:8787/api/posts/p-desk/unlock \
  -H 'content-type: application/json' -d '{"user":"Studio Owner"}'

# pull deterministic pseudo-metrics for February Instagram posts
curl -X POST localhost:8787/api/metrics/import \
  -H 'content-type: application/json' -d '{"platform":"Instagram","month":2}'

# promote an idea into April
curl -X POST "localhost:8787/api/ideas/i-1/promote?month=4"

# simulate the Meta OAuth result (covers the Facebook Page + linked Instagram)
curl -X POST localhost:8787/api/platforms/meta/connect

# add a restricted role (then reference it from a user)
curl -X POST localhost:8787/api/setup/roles \
  -H 'content-type: application/json' -d '{"name":"Intern","permissions":["posts.write"]}'
curl -X DELETE localhost:8787/api/setup/roles/Intern

# require auth, then call an endpoint with a token
curl -X PATCH localhost:8787/api/setup -H 'content-type: application/json' -d '{"authRequired":true}'
curl -X PATCH localhost:8787/api/setup -H 'authorization: Bearer <token>' \
  -H 'content-type: application/json' -d '{"owner":"Studio Owner"}'
```

### Content studio (CMS)

Client-facing editorial workflow for articles/pages/notes: draft → review → scheduled →
published → archived, with markdown bodies, tags, SEO fields, revision history and optimistic
concurrency (`version`). Public delivery is read-only and only ever returns published items.

```sh
# list (needs a token when authRequired is on) — filter by status and search text
curl 'localhost:8787/api/content?status=draft&q=morning' -H 'authorization: Bearer <token>'

# create a draft (content.write) — slug is generated when omitted
curl -X POST localhost:8787/api/content -H 'authorization: Bearer <token>' \
  -H 'content-type: application/json' \
  -d '{"title":"Launch notes","kind":"article","body":"# Hello"}'

# edit (content.write) — stale `version` → 409, a revision is stored first
curl -X PATCH localhost:8787/api/content/c-xxxx -H 'authorization: Bearer <token>' \
  -H 'content-type: application/json' -d '{"version":1,"body":"# Hello v2","note":"Tighten intro"}'

# publish / schedule / unpublish (content.publish)
curl -X POST localhost:8787/api/content/c-xxxx/publish \
  -H 'authorization: Bearer <token>' -H 'content-type: application/json' -d '{"version":2}'
curl -X POST localhost:8787/api/content/c-xxxx/schedule \
  -H 'authorization: Bearer <token>' -H 'content-type: application/json' \
  -d '{"version":2,"scheduledFor":"2026-12-01T09:00:00Z"}'

# history + restore (content.write)
curl localhost:8787/api/content/c-xxxx/revisions
curl -X POST localhost:8787/api/content/c-xxxx/revisions/1/restore \
  -H 'authorization: Bearer <token>' -H 'content-type: application/json' -d '{"version":3}'

# admin onboards a client (users.manage); without a password one is generated and returned once
curl -X POST localhost:8787/api/auth/accounts -H 'authorization: Bearer <token>' \
  -H 'content-type: application/json' \
  -d '{"name":"Acme Client","email":"acme@example.com","role":"Client"}'
# → {"ok":true,"account":{…},"temporaryPassword":"cp-…"}   (null when a password was supplied)

# public delivery (no auth) — published items only
curl localhost:8787/api/public/content
curl localhost:8787/api/public/content/why-slow-mornings-changed-our-year
```

Scheduled items flip to published lazily whenever a content read happens after their
`scheduledFor` time (no cron required). Deletion needs `content.delete` (Owner only by default).

### Auth

- **Accounts** are email + password, hashed with **Argon2id** (`argon2` crate, default params).
  Passwords are at least 8 characters. Accounts are internal (`Store::accounts`, keyed by
  lowercased email) and are **never serialized** — API responses only ever carry
  `User { name, role }`, the directory entry from `setup.users`.
- **Identity is the account, not the display name.** Sessions store the account email, and
  permission checks resolve email → account name → directory role. Display names are unique
  across accounts (`409 name already taken` on register), so no two accounts can share an
  identity.
- **Sessions** are random 64-char lowercase hex tokens (32 bytes from `rand`) with a **30-day
  TTL**; expired sessions are rejected as `401` and dropped on use. Sessions are in memory, so
  they reset on restart.
- **Demo credentials** (seeded, directory role): `owner@studio.local` / `demo1234` (Owner) and
  `editor@studio.local` / `demo1234` (Editor).
- **Registering** adds a directory entry with role `Viewer` when the name is new; if the name
  already exists in `setup.users` (and is not already an account), that role is kept.
  Registration auto-logs-in.
- **Changing the password** keeps the caller's token valid and revokes every other session of
  that account. `POST /api/auth/logout-all` revokes them all.
- **Every account can rename itself** (`PATCH /api/auth/profile`) — no permission needed beyond a
  live session; the matching directory entry follows the rename so the role is preserved.
- **Account administration**: `GET /api/auth/accounts` lists accounts with live session counts
  (no hashes) and `DELETE /api/auth/accounts/{email}` deletes an account and revokes its
  sessions. `setup.users` stays a directory (team list + roles) — removing a directory entry
  unassigns a role, while removing an account revokes access.
- **Locks derive from the session** when `authRequired` is on: lock/unlock store the signed-in
  account's name and `PATCH /api/posts/{id}` compares against it, so a client cannot claim
  someone else's name. In demo mode the wireframe keeps the client-supplied `user`. Creating a
  post never accepts a `lockedBy` from the client.

### Permissions

Mutating endpoints are gated by `authRequired` and the caller's role:

- `authRequired: false` (the seed default) → demo mode: every mutating request is allowed and
  tokens are optional.
- `authRequired: true` → mutating requests need `Authorization: Bearer <token>`:
  - missing/unknown/expired token → `401 missing bearer token` / `401 invalid token`
  - the user's role (via `SetupConfig.roles`) must list the required permission, otherwise
    `403 missing permission: <perm>`
- `POST /api/auth/register` and `POST /api/auth/login` are public; `GET /api/auth/me`,
  `POST /api/auth/logout` and `POST /api/auth/change-password` parse the bearer token themselves.
  Reads (`GET`) also require a live bearer token in production, except for liveness
  (`/api/health`, `/api/ready`), the redacted `GET /api/setup`, the public delivery API
  (`/api/public/*`), the brand CI (`/api/brand`, `/api/brand/fonts`, font and image files), and the
  OAuth/docs routes. An anonymous `GET /api/setup` returns the workspace config without the
  staff directory or role matrix. New self-serve registrations get the read-only `Viewer`
  role; registering with the name of an existing directory entry (an invite) keeps that role.

Known permissions and where they apply:

| Permission | Endpoints |
|---|---|
| `setup.write` | `PATCH /api/setup`, `POST /api/setup/options` |
| `users.manage` | `POST/DELETE /api/setup/users`, `POST/DELETE /api/setup/roles`, `GET /api/auth/accounts`, `DELETE /api/auth/accounts/{email}` |
| `posts.write` | `POST /api/posts`, `PATCH /api/posts/{id}` |
| `posts.lock` | `POST /api/posts/{id}/lock`, `POST /api/posts/{id}/unlock` |
| `ideas.write` | `POST /api/ideas`, `POST /api/ideas/{id}/toggle`, `POST /api/ideas/{id}/promote` |
| `hashtags.write` | `POST /api/tags/{id}` |
| `metrics.import` | `POST /api/metrics/import` |
| `finance.write` | `POST /api/txns` |
| `brand.write` | `PATCH /api/brand`, brand image upload/delete |
| `platforms.manage` | `POST /api/platforms/{id}/connect`, `/disconnect`, `/sync`, `/api/ads/sync`, `/api/ads/manage`, and the ads campaign write endpoints |
| `content.write` | `POST /api/content`, `PATCH /api/content/{id}`, `/duplicate`, `/revisions/{n}/restore` |
| `content.publish` | `POST /api/content/{id}/publish`, `/unpublish`, `/schedule` |
| `content.delete` | `DELETE /api/content/{id}` |
| `users.manage` (CMS) | `POST /api/auth/accounts` (admin creates a sign-in account) |

Seed roles: **Owner** (all 13), **Editor** (`posts.*`, `metrics.import`, `ideas.write`,
`hashtags.write`, `content.write`, `content.publish`), **Viewer** (none), **Client**
(`content.write`, `content.publish` — the Content Studio path). In demo mode a ready-made
`client@studio.local / demo1234` account is seeded so the client portal can be tried locally.

## Real OAuth (credential-optional)

The social login pages run against a real OAuth 2.0 authorization-code flow. It activates
automatically when credentials are present; otherwise everything stays in **mock mode** (the
wireframe simulation keeps working, and the callback marks the platform connected locally).

| Mode | When | What happens |
|---|---|---|
| `mock` | no credentials for that provider | `/start` returns `{mode:"mock"}` → the frontend runs its simulated login; `/callback` (if reached) connects locally |
| `redirect` | credentials set | `/start` creates a single-use `state` and returns the provider authorize URL; the browser logs in **on the provider's domain**; `/callback` exchanges the code, resolves the account, stores handle/externalId/scopes/token/expiry |

Environment:

```sh
# Meta — one login covers the Facebook Page + its linked Instagram account
export META_APP_ID=...          # Meta developer app → Settings → Basic
export META_APP_SECRET=...
# Google / YouTube
export GOOGLE_CLIENT_ID=...
export GOOGLE_CLIENT_SECRET=...
# TikTok
export TIKTOK_CLIENT_KEY=...
export TIKTOK_CLIENT_SECRET=...
# URLs
export PUBLIC_URL=http://localhost:8787      # must match the redirect URI in the provider console
export FRONTEND_URL=http://localhost:5173    # where the callback sends the browser back
export META_GRAPH_BASE=                      # optional: override graph.facebook.com (staging/tests)
```

The redirect URI to register in each provider console is `{PUBLIC_URL}/api/oauth/{platform}/callback`
(e.g. `http://localhost:8787/api/oauth/meta/callback`). On startup the server prints
`OAuth : real mode for …` or `OAuth : mock mode …`.

Security notes:

- Credentials are entered **only on the provider's own page** — in `redirect` mode the frontend
  hides its demo login form and shows a "Continue to …" button instead.
- `state` is random (`st-N`), single-use, and expires after 10 minutes (CSRF protection). The
  callback cannot carry a bearer header (it is a browser redirect), so `state` is its guard;
  bind it to the session/workspace in a multi-tenant deployment.
- Client secrets never leave the server (a test asserts the authorize URL contains no secret).
- Tokens are stored in memory today; persist them encrypted in production.

Example (Meta):

```sh
curl localhost:8787/api/oauth/meta
# → {"platform":"meta","mode":"redirect","configured":true,"authorizeHost":"www.facebook.com"}
curl localhost:8787/api/oauth/meta/start
# → {"mode":"redirect","url":"https://www.facebook.com/v26.0/dialog/oauth?client_id=…&scope=…&state=st-4"}
```

The connection handle is the Page's public profile URL (e.g.
`https://www.facebook.com/profile.php?id=61592348575800`), taken from the Graph `link` field, so
the app can link straight to the account.

Meta app checklist: create a **Business**-type app, add the *Facebook Login* product, register the
redirect URI, and request the scopes you need (`pages_show_list`, `pages_read_engagement`,
`pages_manage_posts`, `read_insights`, `business_management`, `instagram_basic`,
`instagram_manage_insights`) — the Instagram scopes require App Review. Until review passes, the
app works in development mode for app admins/testers.

## Live content mirror

OAuth proves who the user is; the live mirror (`src/live.rs`) pulls what the account actually
publishes:

- `GET /api/live` returns the active workspace's mirror: `fetchedAt`, `accounts[]` (Page + linked
  Instagram stats), and `posts[]` (caption, media, permalink, likes/comments/shares/views).
- `POST /api/platforms/meta/sync` performs a real Graph refresh: `/{page-id}/posts` with engagement
  summaries, the linked Instagram account's `/{ig-id}/media`, and best-effort view counts for the
  newest videos. Other providers only get their sync stamp bumped for now.
- The first mirror is pulled automatically right after a successful connect (in the background, so
  the OAuth redirect is not delayed), and a background loop refreshes every workspace with a
  connected Meta login every `LIVE_REFRESH_MIN` minutes (default 30; `0` disables).
- A failed refresh never wipes the mirror: the posts stay and `error` explains the staleness.
- Provider tokens are sealed at rest with ChaCha20-Poly1305 under a key derived from `ADMIN_TOKEN`
  (Argon2id + random per-store salt, `src/secrets.rs`), so they survive restarts without lying in
  the snapshot in the clear. Without `ADMIN_TOKEN` (demo mode) tokens stay in memory only, exactly
  as before.

## Meta Ads mirror + management

The same Meta login also mirrors the account's advertising (`src/ads.rs`):

- `GET /api/ads` returns `accounts[]`, `campaigns[]`, `adsets[]`, `ads[]`, last-30-days
  `insights[]`, the workspace `audit[]`, and `canManage` (the opt-in state).
- `POST /api/ads/sync` performs a real Marketing API refresh: `/me/adaccounts` (with the
  `/me/businesses` fallback for Business-portfolio accounts), then per account `campaigns`,
  `adsets`, `ads`, and one `insights` call at campaign level. ~5 read points per account — the
  development tier allows 60 points per ad account per 300 s window. A background loop refreshes
  every `ADS_REFRESH_MIN` minutes (default 60; `0` disables).
- Reading needs `ads_read`. The Marketing API's default (Limited) tier covers the owner's own ad
  accounts — no App Review. Permission failures map to "reconnect Meta to grant ads access".
- **Management is off by default** and gated by the workspace opt-in (`POST /api/ads/manage`), the
  `platforms.manage` permission, and the campaign existing in this workspace's mirror:
  `POST /api/ads/campaigns/{id}/status` (ACTIVE/PAUSED/ARCHIVED), `/budget` (minor currency units),
  `/duplicate` (deep copy, paused), and `/api/ads/boost` (paused campaign + ad set + ad promoting a
  Page post from the live mirror). Every write is appended to `audit[]` (newest first, cap 50) and
  uses `ads_management`. Nothing Huuk creates starts spending on its own.

## Data model

Defined in `src/model.rs` (mirrors `../production/src/mock/db.ts`):

- `Post` — the write-master: `id, month, topic, pillar, format, goal, date?, time, status,
  hook, caption, cta, hashtagGroup, hashtags[], imageUrl, note, done, platforms[], lockedBy`
  (`lockedBy` is null when unlocked)
- `SetupConfig` — language, year, owner, pillars/formats/goals/statuses/platforms lists, `users[]`,
  `authRequired`, `roles[]`
- `User` — `name, role` (directory entry for the team UI; passwords live in internal
  `Account`s keyed by lowercased email, never serialized)
- `Role` — `name, permissions[]`; known permissions are the ten listed above (validated on write)
- `Idea`, `HashtagGroup`, `Metric`, `Txn`, `Brand`
- `PlatformConnection` — `status, handle, externalId, scopes[], tokenType, expiresAt, lastSync,
  mediaCount, note`

Validation rules (stricter than the mock, on purpose):

| Rule | Response |
|---|---|
| `POST /api/posts` or `/api/ideas` with empty `topic` | `400 topic is required` |
| `POST /api/setup/options` with unknown `list` | `400 unknown option list '…'` |
| Option/tag duplicated | ignored (list unchanged) |
| `POST /api/setup/users` with empty or duplicate `name` | `400 name is required` / `400 user already exists` |
| `DELETE /api/setup/users/{name}` removing the last user | `400 cannot remove the last user` |
| `PATCH /api/setup` with an unknown permission or empty role name | `400 unknown permission '…'` / `400 role name is required` |
| `POST /api/setup/roles` with empty/duplicate `name` or unknown permission | `400 role name is required` / `400 role already exists` / `400 unknown permission '…'` |
| `DELETE /api/setup/roles/{name}` on the last role / a role assigned to users | `400 cannot remove the last role` / `400 role is assigned to users` |
| Mutating request while `authRequired` without/with an invalid or expired token | `401 missing bearer token` / `401 invalid token` |
| Mutating request while `authRequired` with a role lacking the permission | `403 missing permission: …` |
| `POST /api/auth/register` with empty `name` / no `@` in `email` / password < 8 chars | `400 name is required` / `400 a valid email is required` / `400 password must be at least 8 characters` |
| `POST /api/auth/register` with an email that already exists (case-insensitive) | `409 email already registered` |
| `POST /api/auth/login` with an unknown email or a wrong password | `401 invalid email or password` |
| `GET /api/auth/me` / `POST /api/auth/logout` / `POST /api/auth/change-password` without a valid token | `401 missing bearer token` / `401 invalid token` (expired sessions are deleted) |
| `POST /api/auth/change-password` with a new password < 8 chars / a wrong old password | `400 password must be at least 8 characters` / `401 old password is incorrect` |
| Lock/unlock/patch a post held by a different user | `409 post is locked by …` |
| `POST /api/metrics/import` with empty `platform` | `400 platform is required` |
| `POST /api/txns` with `amount <= 0` | `400 amount must be positive` |
| Any `{id}` route with an unknown id | `404 … not found` |

## Architecture notes

- **State**: one `Arc<RwLock<Store>>` (`AppState`). Handlers take a read lock for queries and a
  write lock for mutations. Fine for a single instance; see below for scaling.
- **Seed data**: `Store::seed()` reproduces the frontend mock exactly (Feb/Mar posts, ideas,
  tags, metrics, transactions, brand, platform connections) so the app looks identical when
  pointed at the server.
- **IDs/timestamps**: `store::uid(prefix)` (atomic counter) and `store::stamp()` /
  `store::plus_days(n)` (chrono). Deterministic enough for tests.
- **Auth**: accounts live in `Store::accounts` (lowercased email → `Account { name, email,
  password_hash }`, Argon2id PHC strings via `hash_password`/`verify_password`). Demon accounts
  are seeded for `owner@studio.local` and `editor@studio.local` (password `demo1234`).
- **Sessions**: `Store::open_session(name)` inserts a random 64-char hex token
  (`store::session_token()`, 32 bytes via `rand`) → `Session { user, expires }` with a 30-day TTL
  (`SESSION_TTL_DAYS`). `/api/auth/me`, `/api/auth/logout` and the permission gate resolve the
  token from the `Authorization: Bearer …` header; expired sessions are removed on use. Everything
  is in memory, so accounts and sessions reset on restart.
- **Authorization**: every mutating handler calls `require_perm` (handlers.rs) before touching the
  store. While `authRequired` is off it is a no-op; otherwise the session's user → role →
  `permissions` chain decides between `401` and `403`.
- **Post locks**: `Post::locked_by` is the only lock state; `PATCH` requires `user` to match the
  holder (the field is consumed for the check and never stored).
- **Metrics import**: `POST /api/metrics/import` derives views/likes from a byte-sum hash of
  `post.id + platform` — same input, same numbers every time — and upserts one row per
  post/platform into `Store::metrics`.
- **Errors**: `ApiError` (`NotFound`/`BadRequest`/`Unauthorized`/`Forbidden`/`Conflict`)
  implements `IntoResponse` → JSON + status.
- **Logging**: `tracing` + `tower_http::TraceLayer`; set `RUST_LOG=debug` for request logging.
- **Layer order** in `lib.rs`: `CorsLayer` → `TraceLayer` → `with_state`. Add auth middleware
  here when needed.

## Tests

```sh
cargo test
```

`tests/api.rs` boots the real router with a fresh state and uses `tower::ServiceExt::oneshot`
to exercise it (no network): health, month filtering, patch + 404, option dedupe, idea promotion,
platform connect/sync/disconnect, transaction validation, the login → me → logout flow (plus 401s),
registration validation/auto-login/duplicate email + duplicate display name and directory-role
resolution, the same-message 401 for unknown email and wrong password, random 64-char tokens with
expired-session cleanup, the change-password flow (wrong old, short new, old password retired,
other sessions revoked), profile rename (session required in demo mode, empty/duplicate guards,
role-preserving directory rename), account administration (list without hashes, per-role 403,
logout-all revocation, account deletion + session revocation + 404), user add/duplicate/remove/last-user
guard, the post lock lifecycle (409s, holder patch, unlock, session-derived holders that ignore
client names, `lockedBy` ignored on create), deterministic metric imports, the auth/permission
system (open demo writes, `authRequired` blocking anonymous writes, per-role 401/403 enforcement,
role add/edit/guards, last-role guard, full role-set replacement guards: empty, duplicate,
in-use), and the OAuth flow (mock mode without credentials, single-use state, mock callback
connecting + redirecting, redirect-mode authorize URL shape with no secret leakage, and
`platforms.manage` enforcement on `/start`).

Production-hardening regressions are covered too: collision-free ids for new rows, atomic
`PATCH /api/setup` (a rejected roles payload cannot half-apply), unknown-role/status/date rejection,
clearing `date` with `null`, unknown-platform and out-of-range month metric imports, malformed-JSON
and unknown-route JSON envelopes, `/api/ready`, month-less `GET /api/posts`, disabled registration,
login lockout (429), sync-requires-connection, OAuth-configured connect blocked, locks following a
profile rename, last-administrator guards, the `X-Admin-Token` recovery path, and the JSON
snapshot round-trip.

## Going to production

Already built in:

- **Serves the frontend itself** — set `STATIC_DIR=app/dist` and the binary hosts the built SPA
  (hash-route fallback, MIME types, immutable caching for hashed assets, traversal protection,
  JSON 404s kept for `/api/*`, app-specific CSP). That is the no-Docker deployment; leave it
  unset for API-only hosting behind nginx.
- **Config from env** — `PORT`, `BIND_ADDR` (default `127.0.0.1`), `DATA_FILE`, `STATIC_DIR`, `DEMO_MODE`,
  `AUTH_REQUIRED`, `ALLOW_REGISTRATION`, `ADMIN_EMAIL`/`ADMIN_PASSWORD`, `ADMIN_TOKEN`,
  `CORS_ORIGINS`, `PUBLIC_URL`, `FRONTEND_URL`, provider credentials.
- **Durability** — the store is snapshotted to `DATA_FILE` atomically every 5s when it changes and
  on SIGTERM/SIGINT; a corrupt snapshot fails fast instead of silently starting empty.
- **Security** — Argon2id off the async runtime, per-account login lockout (10 failures / 15 min),
  timing-equalized unknown-email responses, CSPRNG single-use OAuth states, admin recovery token,
  per-field validation with length caps, 256 KB body limit, 30s request timeout, security headers,
  `no-store` responses, and a configurable CORS allow-list.
- **Operations** — `/api/health` (liveness), `/api/ready` (readiness + version/mode), structured
  tracing with login/denied/admin audit events, graceful shutdown.

Recommended next steps for a multi-instance deployment:

1. **Database** — replace `store::Store` with a repository backed by
   [sqlx](https://github.com/launchbadge/sqlx) + Postgres/SQLite. Keep `AppState` opaque to
   handlers: make the store an `Arc<Repo>` with async methods (`list_posts`, `update_post`, …) and
   move the current logic into it. Run `sqlx migrate` with a schema derived from `model.rs`. The
   JSON snapshot path is fine for a single instance; SQLite is a drop-in middle step.
2. **Secrets & tokens** — provider access tokens are kept in memory only; persist them in a
   secrets manager or encrypted at rest before enabling unattended sync.
3. **OAuth state binding** — states are random and single-use; for multi-instance deployments move
   `oauth_states` into the shared database (or Redis) so any replica can complete the callback.
4. **Background sync** — move `sync_platform` into a scheduled worker and add per-platform
   throttling; the API already returns `lastSync` for the UI.
5. **TLS** — terminate HTTPS at the proxy/load balancer (HSTS is already sent) and set
   `BIND_ADDR=0.0.0.0` explicitly when running in a container. A minimal container is in
   `server/Dockerfile`.

## Why the contract matches the mock

`../production/src/mock/api.ts` is the interface definition for this service (kept in sync by hand; both
use the same TypeScript types). If you add an endpoint, add it in both places and cover it with a
test on each side — the frontend's `src/api/http.ts` maps 1:1 to the routes above.
