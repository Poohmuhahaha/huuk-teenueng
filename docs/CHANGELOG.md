# Changelog — what was built and why

Wireframe frontend for the Content Planner, built 14 Sep 2026 from the five specification PDFs in
the parent folder. This file records the decisions that matter for maintenance.

## Base app

- **Vue 3 + Vite + TypeScript (strict)**, component-based, black-and-white wireframe theme
  (`src/style.css` tokens). No backend: `src/mock/db.ts` (types + seed) → `src/mock/api.ts`
  (async endpoints) → `src/queries.ts` (TanStack Query hooks) → pages/components.
- **All 11 sheets became screens**: Setup, Brand, Planner 01–12, Calendar, Monthly, Feed,
  Dashboard, Performance, Ideas, Hashtags, Finance — journey order J1–J10, one route each.
- **Guide** was a screen at first (`GuidePage.vue`); it was later **removed as a card** and turned
  into a navbar overlay (`GuideHero` inside `AppShell.vue`) so the deck starts at real work.
- **TanStack Table v9** for the two data tables. Migrated from the v8 pattern during setup:
  `useTable` + explicit `tableFeatures`, `TFeatures`-first generics, `FlexRender :cell/:header`
  shorthand, `sortFn: 'basic'` where numeric desc-first sorting is wrong. Row types are `type`
  aliases because v9's `RowData` requires an index signature.
- **Computed-field guard** (`ProtectedModal`) reproduces the workbook's protected-range dialog:
  Cancel default, "hide for 5 minutes", OK is the override path.

## Navigation: pages → slide deck

- The app originally swapped pages via `RouterView`. It became a **card deck** (`ScreensDeck.vue`):
  one horizontal snap track with all screens, so navigation feels like sliding cards.
- Order and metadata come from a single registry, `src/screens.ts`; the router derives from it too.
- Two-way sync: route changes slide the deck; the settled slide updates the URL (`router.replace`).
- Inputs: drag, touch, ← →, dots, navbar links, tapping a peeking card, `Escape` to exit full page.
- Guards added after real bugs: `syncing` (ignore programmatic scroll), `commit()` disabled in full
  mode, `nearestIndex()` skips collapsed slides, drag capture deferred past the threshold.

## Full-page mode (Go full page)

Evolved through several approaches; the current one is the result of that iteration:

1. Layout-only version: paged cards were 94% wide → felt cramped.
2. 1:3:3:1 viewport restored: `.slide = 37.5vw`, `scroll-padding-inline: 12.5vw` (peeks), phone 75vw.
3. Fixed card height so sliding never changes vertical rhythm; border added; snap-stop `always`.
4. First animation attempts: CSS transitions on `flex-basis`/height (warped), then View Transitions
   (scaled vertically), then `clip-path` (laggy — repaints), then transform curtains (smooth, but
   content felt static), then a `scaleX` squeeze (smooth but stretched the content).
5. **Current**: the card is pinned `position: fixed`, its **real `width` and `left` are animated**
   with Motion for Vue (`motion-v`, Framer Motion's engine) — content reflows naturally, no
   distortion. The other cards **cross-fade during** the animation (`.deck.fading` on enter;
   layout released at the start on exit). Height never changes.
6. Fixed after the first release: exit target math now accounts for clamped scroll positions, so the
   first card (Setup) keeps its left edge while shrinking and end cards land flush.

## Responsive layout

- Global shell is full-screen; paged viewport shows **[1 peek][3 card][3 card][1 peek]**.
- Grid classes are 2/3 columns for small cards; `grid4`/`grid6` are capped at 3 when narrow.
- **Container queries** on `.slide-card` (`container-name: screen`) make layouts respond to the
  card's own width: ≥ 720 px → 4 columns, ≥ 1100 px → 6 columns for `grid4`/`grid6`/`cards`.
  Because the card's width animates, the layout visibly reflows mid-zoom (3 → 4 → 6/8).
- Viewport breakpoints: ≤ 860 px grids → 2 cols, ≤ 560 px → 1 col; ≤ 640 px the deck switches to
  75vw cards with no left peek.

## Shell & navigation chrome

- Full-screen shell, no side rails (an earlier 1:3:3:1 layout with left/right rails was removed).
- Top bar enlarged for readability; links have hover/focus states; **Guide sits on the right**.
- **Avatar menu**: owner name + Setup / Brand / Ideas / Finance; closes on outside click or `Esc`.
- Removed: the route path bar (`/setup` readout) and all explanatory subtitles/notes with spec
  citations — the wireframe now speaks through its interface.

## Data & features

- `useDashboard()` computes KPIs, pillar/platform/status splits and Top-5 from posts + metrics.
- Planner writes through a **local draft** then a single save mutation; the CopyBar assembles
  caption + CTA + hashtags for one-click publishing (the sheet's Copy-Paste column).
- Ideas promote into dated Planner rows; Performance shows +7-day follow-ups (`plus7`); Finance
  sums the ledger and renders monthly IN/OUT charts.

## Testing & tooling

- Vitest + happy-dom: `tables.test.ts` (render, asc/desc sort, search filter, row select, checkbox
  selection, numeric desc-first) and `carousel.test.ts` (drag threshold, ends clamp, arrow keys).
- `npm run build` = `vue-tsc -b && vite build` (type errors fail the build — keep it green).

## Known gaps (future work)

- Persistence/realtime — swap `src/mock/api.ts`/`server/src/store.rs` for real storage; both stores
  are in-memory today and reset on restart.
- Auth is wireframe-grade (named sessions, no passwords, tokens in memory only) and platform OAuth
  is simulated; lock has no TTL/heartbeat.
- i18n covers navigation, Setup, Guide and month names; long page body copy stays English.

## Collaboration & completion (US-008/011/012/014), i18n, feed sizing

- **i18n EN/TH** (`src/i18n.ts`): two dictionaries with enforced key parity, `t()`, `monthName()`,
  `monthNameShort()`. Setup's language radio opens an inline confirm; navbar `EN|TH` flips directly;
  an `AppShell` watcher calls `syncLang()` so all labels/months re-render. Tests: parity, switching,
  fallback, Thai month names.
- **Users (US-011)**: `SetupConfig.users[{name, role}]`, editable on the Setup screen; API rejects
  empty/duplicate names and removing the last user.
- **Auth (US-012)**: `POST /api/auth/login` (name → `{token, user}`), `GET /api/auth/me` (Bearer),
  `POST /api/auth/logout`; sessions live in the store. Frontend keeps token+user in `src/auth.ts`
  (in-memory), shows the user in the avatar menu and a `LoginModal` that lists Setup users.
- **Post locking (US-014)**: `Post.lockedBy`; lock/unlock endpoints (409 for other users,
  idempotent for the holder); `PATCH` refuses writes by non-holders. Planner shows a "Locked by …"
  chip, disables Save, offers Lock/Unlock (login required). One row is pre-locked in both seeds.
- **Metrics import (US-008)**: `POST /api/metrics/import {platform, month}` upserts deterministic
  pseudo-metrics (`views = 1000 + h%9000`, `likes = views/12`); Performance screen gained the import
  panel. Mock mirrors both the hash and the response shape.
- **Feed sizing (US-005)**: `platforms.ts` gained per-platform canvas sizes + variants; FeedGrid
  takes `platform`/`variant` props, binds `aspect-ratio`, spans 2 columns for wide ratios, and shows
  the selected canvas; Feed screen adds a Size selector and legend.
- **Docs + PDF**: `specs/06-project-documentation.pdf` (Typst) documents the as-built system; server
  README covers the new endpoints; this file and the others below updated.


## Platform connections (wireframe)

- Added a horizontal **Social Media bar** below the navbar (Instagram / Facebook / YouTube /
  TikTok) — checkboxes pick the platforms to work with (min 1, registry order); the first checked
  is the primary `activePlatform` that Feed Review follows. Clicking a platform name opens its
  connection modal.
- Added a wireframe **OAuth 2.0 connect flow** (`PlatformConnectModal`): animated pipeline steps
  per platform (Meta: short-lived → long-lived token → Page → `instagram_business_account`;
  Google: offline access + refresh token + channel ID; TikTok: open_id + refresh token), scopes,
  constraints (rate limits, story insight lifetime, quotas), plus connected-state actions
  (Sync now / Disconnect).
- Mock layer: `PlatformConnection` type + seeds, four API functions, four query hooks.
  Feed Review now follows the active platform from the switcher.
- Tests: `platforms.test.ts` — switcher render/switch/status dot, connect/disconnect, sync count.

## Rust backend + data-source switch

- Added `server/` — Axum 0.8 + Tokio REST backend that mirrors the frontend mock contract:
  setup/options, posts (month filter, add, patch), ideas (add/toggle/promote), tags, metrics,
  transactions (validation), brand, platform connections (connect/disconnect/sync). In-memory
  `Arc<RwLock<Store>>` with the same seed data; JSON camelCase; `ApiError` → 400/404 JSON;
  permissive CORS + tracing. 7 integration tests (`cargo test`).
- Frontend now switches data source via `src/api/index.ts`: unset `VITE_API_URL` → mock,
  set → `src/api/http.ts` → the Rust server. `queries.ts` imports the switch only; pages unchanged.
  Added `.env.example`.
- Docs: `server/README.md` (run, API reference, data model, production path), plus updates here
  and in `ARCHITECTURE.md §1`, `GUIDES.md`, `README.md`.

## Auth & permissions config + social platform login pages

- **Configurable permissions**: `SetupConfig` gained `authRequired` (default `false` = demo mode) and
  `roles: Role[]` (`Role = {name, permissions[]}`). Ten permissions: `setup.write`, `users.manage`,
  `posts.write`, `posts.lock`, `metrics.import`, `finance.write`, `brand.write`, `ideas.write`,
  `hashtags.write`, `platforms.manage`. Seeded: Owner (all), Editor (5 content perms), Viewer (none).
- **Setup panel "Auth & permissions"**: require-login checkbox, roles × permissions matrix with a
  draft + save, add/remove role (server guards: in-use / last role), all gated on `users.manage`.
- **Enforcement**: when `authRequired` is on, mutating endpoints require `Authorization: Bearer`
  (401) and the role must hold the permission (403) — implemented in both the Rust server
  (`require_perm`) and the mock (`requirePerm`). `usePermission().can()` drives disabled states on
  Planner (save/lock), Performance (import), Finance, Brand, Ideas, Hashtags and Setup.
- **Social platform login pages** (`PlatformLogin.vue`) replace the old connect modal: a full-screen
  page with browser chrome → the platform's own login form (per-platform field labels + SSO) →
  2FA code → consent with scopes/constraints → animated OAuth pipeline → connected summary with
  Sync/Disconnect. Connect actions honour `platforms.manage`. No real network calls.
- **Token storage** moved to `src/token.ts` so `api/http.ts` can attach the Bearer header without an
  import cycle; `auth.ts` re-exports it.
- Tests: Rust 19/19, frontend 36/36 (`permissions.test.ts` covers demo mode, 401/403, roles CRUD,
  matrix edits). Docs updated; `specs/06-project-documentation.pdf` refreshed.

## Real OAuth (credential-optional) — Meta first

- **`server/src/oauth.rs`**: authorization-code flow for Facebook, Instagram, YouTube and TikTok.
  `GET /api/oauth/{platform}` reports `mode: mock | redirect`; `/start` creates a random single-use
  `state` (10-min TTL) and returns the authorize URL; `/callback` validates the state, exchanges the
  code (`reqwest`), resolves the account per provider, and stores handle/externalId/scopes/token/
  expiry on the connection, then redirects to `/#/setup?oauth=…&status=…`.
- **Credential-optional**: with no env credentials the flow reports `mock` and the callback connects
  locally, so the wireframe demo and all existing tests keep working. Env:
  `META_APP_ID`/`META_APP_SECRET` (Meta = Facebook + Instagram), `GOOGLE_CLIENT_ID`/`GOOGLE_CLIENT_SECRET`,
  `TIKTOK_CLIENT_KEY`/`TIKTOK_CLIENT_SECRET`, `PUBLIC_URL`, `FRONTEND_URL`. Startup prints the mode.
- **Security**: secrets stay server-side (test asserts the URL contains none); the frontend hides its
  demo credential form in redirect mode (passwords only on the provider's domain); `/start` honours
  `platforms.manage` when auth is on; the callback is guarded by the single-use state.
- **Frontend**: `PlatformLogin.vue` fetches the mode per platform, shows "Continue to …" instead of
  the fake form in redirect mode, and redirects. `AppShell` shows a connected/failed banner when the
  callback returns and refreshes the platform query. `http.ts` now attaches the bearer token to every
  request (needed for the gated `/start`).
- Tests: Rust 24/24 (mock mode, state validation + single-use, mock callback redirect, redirect-mode
  URL shape, permission gating), frontend 37/37. Docs updated (`server/README.md` § Real OAuth).

## Real SaaS authentication (email + password)

- **Server** (`src/store.rs`, `src/handlers.rs`): `Account {name, email, password_hash}` (internal —
  never serialized) and `Session {user, expires}`. Passwords are hashed with **Argon2** default
  params; session tokens are **random 64-char hex** with a 30-day TTL. New endpoints:
  `POST /api/auth/register` (400 empty name/bad email/<8-char password, 409 duplicate email; role from
  the Setup directory or `Viewer`; auto-login), `POST /api/auth/login {email, password}` (401
  `invalid email or password` for both failure modes), `POST /api/auth/change-password` (revokes the
  user's other sessions). The old `login {name}` pick-a-user call is gone. Seeded demo accounts:
  `owner@studio.local` / `editor@studio.local`, password `demo1234`. `SetupConfig.users` stays the
  role directory.
- **Frontend**: `LoginModal` is a login/register form; new `ChangePasswordModal` in the avatar menu;
  `auth.ts` gained `register`, `changePassword`, `restore()`; `token.ts` persists to `localStorage`
  (`cp.token`) and rehydrates on boot. Mock mirrors every endpoint, error message and TTL.
- Tests: Rust 28/28 (validators, duplicate email, role mapping, generic 401s, token randomness +
  expiry cleanup, change-password revocation), frontend 49/49 (`auth.test.ts` plus updated collab
  and permission suites).

## Hardening audit fixes (identity, revocation, role guards, locks)

- **Identity is the account, not the display name.** Sessions store the account email; permissions
  resolve email → account name → directory role. Registration rejects a display name already used by
  another account (`409 name already taken`), so two accounts can never share a role.
  `change-password` now edits the caller's own account by email instead of the first name match.
- **Account administration + revocation.** New endpoints: `GET /api/auth/accounts` (name, email,
  live session count — hashes never serialized; `users.manage`), `DELETE /api/auth/accounts/{email}`
  (deletes the account and revokes its sessions; the Setup directory entry stays), and
  `POST /api/auth/logout-all` (revokes every session of the calling account). Setup gains an
  Accounts panel with remove; the avatar menu gains "Sign out everywhere".
- **Full role-set replacement is guarded.** `PATCH /api/setup {roles}` now rejects an empty array,
  blank/duplicate names, unknown permissions, and any role still assigned to a directory user
  (`role is in use: X`) — same rules as the individual role endpoints.
- **Locks derive from the session.** With `authRequired` on, lock/unlock store the signed-in
  account's name and `PATCH /api/posts/{id}` checks it against the holder; client-supplied `user`
  names are ignored. Creating a post always stores `lockedBy: null`. Demo mode keeps the client
  name for the wireframe.
- Tests: Rust 32/32 (new: duplicate-name registration, account list/revoke/logout-all, role-set
  guards, session-derived locks), frontend 56/56 (new: account admin, logout-all, lock derivation,
  duplicate-name register, role guards; mock mirrors every rule).

## Profile & setup UX restructure

- **Setup moved out of the top nav into the avatar menu** ("Profile & setup" → `/setup`).
- **Sign up is a first-class navbar action**: logged-out users see `Sign up` + `Log in` buttons;
  `LoginModal` takes an `initialMode` and emits `success(mode)`. Registering routes to `/setup`
  with a welcome banner so the first sign-up lands on profile setup.
- **Profile panel on Setup** (always visible): any signed-in account can rename itself via the new
  `PATCH /api/auth/profile` (401 without a session even in demo mode, 400 empty, 409 duplicate);
  the Setup directory entry follows the rename so the role survives. Role is read-only, and a hint
  explains when the account lacks setup rights.
- **Language is a personal preference now**: the navbar `EN|TH` toggle applies immediately, persists
  to `localStorage` (`cp.lang`), and only writes `setup.language` when the user has `setup.write`.
  The Setup language radio/inline-confirm panel was removed.
- **Guide moved from the navbar into the avatar menu** (overlay unchanged).
- **PlatformLogin**: the fake browser chrome (dots + URL bar + Close) is gone; the flow's own
  back/cancel/done buttons remain.
- Tests: Rust 33/33 (new profile rename suite), frontend 59/59 (3 new profile tests).

## Profile route + spacing

- The first deck card is now **Profile** (`/profile`, sheet "Profile & setup"); `/setup` stays as an
  alias redirect that preserves query params (the OAuth callback still lands there), `/` redirects
  to `/profile`, and the OAuth `303` now targets `#/profile?oauth=…`.
- Setup page: added breathing room between the Profile panel and the Year/Owner section, and the
  page title reads "Profile & setup" in both languages.

## Profile & setup are popups (no deck card)

- The Profile card was removed from the slide deck: `SetupPage.vue` is deleted and `screens.ts`
  is back to 10 cards. `/` redirects to the first card and `/profile` + `/setup` are aliases that
  redirect to `/` preserving query params (the OAuth callback still lands and shows its notice).
- Clicking the avatar now opens a **compact popup** (`ProfilePanel.vue`): name (editable),
  role, Save, plus Guide, Change password, Sign out everywhere, Log out (or Log in for guests).
- "Workspace settings" in that popup opens the **settings popup** (`SettingsPanel.vue`) with the
  former setup content: year/owner, option lists, users, accounts, permissions (read-only hint
  when signed in without rights).
- Registering now reopens the profile popup with the welcome notice instead of navigating.

## Production-hardening pass (frontend)

- **HTTP client** (`src/api/http.ts`): typed `ApiError` that surfaces the server's `{"error": …}`
  message, 15s timeout, 204/empty-body handling, URL normalization (`/api` suffix/trailing slash),
  no `Content-Type` on GETs (fewer preflights), and centralized 401 handling that clears the
  session. A production build always uses HTTP unless `VITE_USE_MOCK=true` — the mock can no
  longer ship by accident.
- **Session lifecycle** (`src/session.ts` + `src/auth.ts`): one shared session module, query-cache
  clearing on login/logout/expiry, and `restore()` only clears the token on 401/403 (a network
  blip no longer signs the user out). `Api` contract (`src/api/contract.ts`) is compile-checked
  against both the mock and the HTTP client.
- **Mock/server parity**: mock now mirrors server validation (month/status/platform/amount),
  trims and de-duplicates options/tags, pushes Viewer directory entries on register, resolves
  removed directory users to Viewer, and uses the same deterministic metric hash.
- **Bug fixes**: arrow keys no longer hijack typing inside the deck; `/planner/:month` params
  actually reach the planner and month tabs update the URL; Dashboard views/likes are
  month-scoped; post dates are `null`-clearable; `ProtectedModal`'s 5-minute suppression now
  works; `PlatformLogin` always has a close affordance and waits for the connect call before
  showing success; last platform checkbox snaps back; brand/ideas/finance/hashtags/settings
  mutations surface errors, disable while pending and only clear drafts on success; destructive
  settings actions confirm first; selection prunes when rows change; hardcoded 2026 dates now
  follow `setup.year`; unknown routes render a 404 card; an error boundary protects the deck.
- Tests: Rust 50/50, frontend 71/71 (new http, guard-suppression, deck-keyboard, PlatformLogin
  close, DOM-sync tests), clippy/fmt clean, CI workflow added.

## Client-facing CMS (Content Studio)

- **Why**: the app managed planning data but had no path for a client to actually write and
  publish content. Added a CMS with an editor built for non-technical clients.
- **Backend** (`server/src/{model,handlers,store}.rs`): `Content` items (article/page/note) with
  `draft → review → scheduled → published → archived`, markdown body, tags, hero image, SEO
  fields, revision history (20 kept), optimistic concurrency (`version` → `409`), lazy scheduled
  publishing, public read-only endpoints (`/api/public/content…`), and
  `POST /api/auth/accounts` for admin onboarding (returns a one-time temporary password when none
  is supplied). Three permissions added (`content.write`, `content.publish`, `content.delete`)
  and a seeded **Client** role; demo mode seeds `client@studio.local / demo1234`.
- **Content Studio** (`src/pages/StudioPage.vue`, `src/components/ContentEditor.vue`): status
  sidebar with search/filter, markdown editor with toolbar, live preview (Write/Split/Preview),
  autosave, explicit save, conflict banner (reload vs overwrite), revision drawer with preview +
  restore, publish/schedule/unpublish, duplicate/archive/delete, SEO counters, share link, word
  count + reading time.
- **Safe rendering** (`src/markdown.ts`): dependency-free renderer that escapes all text before
  inserting tags; only `http(s)`, root-relative and `#/` links are allowed (no `javascript:`),
  external links get `rel="noopener noreferrer"`.
- **Client portal**: Client role sees only the Content nav and is steered to `/studio`; the
  studio is a standalone wide route, the public reader (`/read`, `/read/{slug}`) renders outside
  the app shell. Staff get a "Content" nav link.
- **Onboarding UI** (`SettingsPanel.vue`): admins create client accounts with a role +
  optional/generated password; the generated password is shown once with a copy button.
- Tests: Rust 58/58 (7 CMS integration suites), frontend 83/83 (markdown safety, studio mock
  parity, client role + onboarding).

## Deployment paths (Docker, Podman, and no-Docker)

- **Single-binary mode**: `STATIC_DIR` makes the Rust server serve the built SPA itself (hash-route
  fallback, MIME types, immutable caching for `assets/*`, path-traversal rejection, JSON 404s kept
  under `/api/*`, app-specific CSP vs the locked-down API CSP). One process needs no nginx.
- **`scripts/install.sh`** builds backend + frontend and installs a systemd service for hosts
  without Docker (user, data dir, generated `ADMIN_EMAIL`/password/`ADMIN_TOKEN`, optional
  `--update` with automatic pre-update backup, `--skip-service` for user-space installs).
- **`scripts/backup.sh` / `scripts/restore.sh`** for the systemd install (JSON validation,
  retention, service stop/start).
- **Docker/Podman**: `docker-compose.yml` (api + web + Caddy TLS profile), multi-stage
  `server/Dockerfile` and `wireframe/Dockerfile`, and direct `podman run` instructions for hosts
  without a compose provider.
- `DEPLOYMENT.md` now leads with the no-Docker path; `deploy.sh` points there when Docker is
  missing. CI builds both container images.
- Tests: Rust 59/59 (static-serving suite), frontend 83/83.

## One Meta connection (Facebook + Instagram), Page URL as handle

- **Why**: the Facebook Page and its Instagram account are served by the same Meta Graph API and
  the same login, so two connection slots ("Facebook" and "Instagram") meant connecting twice for
  one platform. The Instagram slot is gone; a single **Meta** connection covers both.
- **Backend** (`server/src/oauth.rs`): the `facebook` and `instagram` providers were replaced by
  one `meta` provider with the combined scope set (`pages_show_list`, `pages_read_engagement`,
  `pages_manage_posts`, `read_insights`, `business_management`, `instagram_basic`,
  `instagram_manage_insights`). `/me/accounts` (with the `/me/assigned_pages` fallback for
  Business-portfolio Pages) now also requests the Graph `link` field; the connection handle is the
  Page's public profile URL (e.g. `https://www.facebook.com/profile.php?id=…`), so the UI links
  straight to the account instead of storing just the Page name.
- **Migration** (`server/src/store.rs::migrate_platforms`): snapshots are normalized on load —
  a stored `facebook` connection becomes `meta` (its handle upgraded from the Page name to the
  profile URL built from the Page id), the `instagram` slot is dropped, missing slots are restored
  in canonical order (`meta`, `youtube`, `tiktok`), and stored provider tokens follow the rename.
- **Frontend**: `PlatformId` is now `meta | youtube | tiktok`; `PLATFORMS` (the Social Media bar)
  lists the three connectable platforms. Instagram survives as a *planner* platform via
  `FeedPlatformId` / `FEED_META` (its 1:1, 4:5, 9:16 canvases are still used for content previews),
  and the Set up list's legacy `"Facebook"` label resolves to `meta` so existing stores keep
  working. `profileUrl('meta', …)` accepts `facebook.com` profile URLs.
- Tests: Rust 77/77 (including a migration test for old snapshots), frontend 136/136.

## Live content mirror (real page content in the app)

- **Why**: connecting Meta only stored the account; nothing pulled the Page's actual posts, so the
  app never showed the real content. Now a workspace keeps a read-only mirror of what the connected
  accounts publish.
- **Backend** (`server/src/live.rs`): `GET /api/live` serves the mirror (`fetchedAt`, `accounts[]`,
  `posts[]` with caption/media/permalink/likes/comments/shares/views, `error`). `POST
  /api/platforms/meta/sync` now performs a real Graph refresh — `/{page-id}/posts` with engagement
  summaries, the linked Instagram account's `/{ig-id}/media`, and best-effort video views for the
  newest posts (a metric the API does not report stays `0`). The first mirror is pulled in the
  background right after connecting, and a loop refreshes every connected workspace every
  `LIVE_REFRESH_MIN` minutes (default 30). A failed refresh keeps the previous posts and records the
  reason. The store lock is never held across network I/O.
- **Token durability** (`server/src/secrets.rs`, `server/src/store.rs`): provider tokens are now
  persisted **sealed** (ChaCha20-Poly1305, key derived from `ADMIN_TOKEN` with Argon2id + random
  per-store salt) so the scheduled refresh survives restarts; a leaked snapshot contains no usable
  token. Without `ADMIN_TOKEN` tokens stay memory-only, and a changed `ADMIN_TOKEN` fails closed
  (the UI asks to reconnect). Workspace deletion drops both copies.
- **Frontend** (`src/components/LiveSection.vue`): a "Live from Meta" section with account chips,
  a synced-ago stamp, a Sync now button, and three layouts — cards (Dashboard), grid (Feed),
  table with engagement columns (Performance). Posts link to their real permalinks; the empty
  state explains how to connect, and a failed refresh is shown without hiding the content.
- Tests: Rust 98/98 (seal/open, snapshot round-trip, parser, a full fetch pipeline against a fake
  Graph server, Graph error mapping, API, no-token sync error), frontend 143/143 (rendering, limits,
  table, sync, empty/error states).

## Reconnect UX for missing tokens + graph base override

- **Why**: a connection can exist without a usable token (first deploy after tokens became
  persisted, changed `ADMIN_TOKEN`, revoked access). The app showed "connected" and the only path
  was Disconnect → Connect, which is confusing when Sync just answers "reconnect Meta".
- **Backend**: `GET /api/platforms` now includes runtime-only `hasToken` per connection (never
  persisted, never the token itself). `META_GRAPH_BASE` can point Meta account/live calls at
  another Graph-compatible base (staging, tests). Tests cover the connect path against a fake
  Graph (Page profile link as handle, assigned-pages fallback, no-Page guidance) and that
  `hasToken` is reported without leaking the token.
- **Frontend**: `src/oauth.ts` centralizes the provider hand-off (`beginOAuth`, allow-listed
  hosts); the connection dialog shows a "needs a fresh login" warning plus a **Reconnect** button,
  and the live section's empty state offers **Reconnect Meta** in one click. The mock API tracks
  `hasToken` across connect/disconnect.
- Tests: Rust 102/102, frontend 147/147.

## Meta Ads: mirror + management

- **Why**: campaigns, their settings and performance live in Ads Manager; Huuk showed only organic
  content. The same Meta login can mirror ads too, so the planner can see what is being spent and
  (optionally) steer it.
- **Backend** (`server/src/ads.rs`): read mirror of ad accounts (with the Business-portfolio
  fallback), campaigns, ad sets, ads + creatives, settings and last-30-days insights; `GET /api/ads`,
  `POST /api/ads/sync`, background refresh (`ADS_REFRESH_MIN`, default 60, `0` disables). Scopes
  gained `ads_read` + `ads_management`; permission errors map to "reconnect Meta to grant ads
  access". Batched 5 read calls per account (dev tier: 60 points/ad account/300 s).
- **Management (off by default)**: workspace opt-in `POST /api/ads/manage`, gated by
  `platforms.manage` and by the campaign existing in the mirror. Pause/resume/archive, budget
  updates (minor units), deep-copy duplicate, and `POST /api/ads/boost` (paused campaign + ad set +
  ad promoting a Page post). Every write is appended to a workspace audit trail; nothing created by
  Huuk starts spending unprompted.
- **Frontend**: new **Ads** page (nav) with summary strip, campaign table, inline drill-down to ad
  sets/ads and settings, budget dialog, duplicate, opt-in toggle, audit list, and a boost dialog
  that picks a post from the live mirror. Live posts promoted by an ad get a "Boosted" badge, and
  the Dashboard shows active campaigns + simple alerts (schedule ended, budget exhausted).
- Tests: Rust 117 (10 new ads unit tests + 5 API tests), frontend 152 (5 new AdsPage tests).

## A Live page for the mirror

- **Why**: the mirrored content was only visible on Dashboard/Feed/Performance, so a successful
  sync looked like nothing happened while on other pages.
- **Frontend**: new **Live** nav item (`/live`) showing the whole mirror (account stats + every
  post, no row limit), and the Meta popup links to it ("View live content") once media is tracked.
- Tests: frontend 154.

## Complete per-post data on the live mirror

- **Why**: cards only showed likes/comments/shares/views; Meta exposes more (and has retired the
  old impression metrics for the new Pages experience — verified against the API).
- **Backend**: posts now carry reactions by type, total clicks + link clicks, video length and
  average watch time, attachment title and link target; the Page account carries
  `page_post_engagements`, net follows (daily follows − unfollows), `page_views_total` and
  `page_video_views`. Per-post insight calls stay capped (20 newest) and degrade gracefully: an
  invalid metric retries with the always-valid reactions metric.
- **Frontend**: cards show the reaction breakdown, clicks, link clicks, saves/reach (IG) and watch
  time, plus the attachment link; the account chip adds engagements / net follows / page views; the
  table variant gained reactions, clicks and watch-time columns.
- Tests: Rust 125 (parser + pipeline assertions for the new metrics), frontend 155.

## Sessions survive restarts + an interactive brand kit

- **Why**: every API restart logged everyone out (sessions were memory-only, and the frontend
  cleared the token on the resulting 401), and the Brand page's logo slots and moodboard were dead
  placeholders.
- **Sessions**: stored in the snapshot as Blake2b hashes of the bearer token, so a restart keeps
  users signed in while the snapshot never contains a usable token. Expired sessions are pruned on
  load; logout/password-change revoke by hash as before.
- **Brand kit**: `brand.logos` (3 slots) and `brand.moodboard` (up to 12) are now editable in the
  UI — upload PNG/JPG/WebP/GIF (≤4 MB, stored beside the snapshot and served from
  `/api/brand/images/{name}`) or paste an https URL, with remove buttons and live previews.
- Tests: Rust 127 (session persistence + image upload/serve/delete + brand validation),
  frontend 157 (brand image interactions).

## Size rules for brand images

- **Why**: logos and moodboard references need pixel limits so the brand kit (and anything built
  from it) never ships a blurry 16px logo or a 12 MP photo.
- **Rules**: every brand image is 32×32 – 500×500 px (both logos and moodboard);
  PNG/JPG/WebP/GIF, ≤4 MB. Dimensions are read from the file header (PNG/JPEG/GIF/WebP) — no
  decoding.
- **Backend**: `POST /api/brand/images` takes `kind`, enforces the bounds and returns
  `{ url, width, height }`; errors name the slot and the actual size.
- **Frontend**: the Brand page shows each slot's range ("PNG/JPG/WebP/GIF · 32×32 – 500×500 px")
  and pre-checks picked files with the same parser, so the error is instant. Mock API mirrors it.
- Tests: Rust 129 (header parsers + bounds + API upload cases), frontend 167.

## Social Media switcher in the navbar + layout guards

- **Why**: the Social Media bar sat on its own row below the navbar; it belongs next to the profile
  button, and oversized brand images could also blow a card's grid out of shape.
- **Frontend**: `SocialMediaBar` now renders inside the top navbar, right before the avatar, with
  compact chips (title hidden on narrow screens). Grid children get `min-width: 0` and previews use
  a fixed box with `object-fit: contain`, so a 500×500 logo can never stretch the layout.

## Palette as labelled cards + brand colors in charts

- **Why**: the palette was four bare color fields in a row with no hint of where each color lands.
- **Frontend**: the palette is now a 2×2 grid of cards — role title (Primary/Secondary/Tertiary/
  Supporting), swatch, color picker + hex field, an "Appears on: …" line, and a live sample
  (button + link for the primary, the matching chart bar for the others).
- **Theme**: all four slots are published as `--brand-1` … `--brand-4`; chart bars cycle through
  them, so every palette color really does appear in the UI (the primary still drives buttons,
  links, nav and focus rings).

## Brand Identity layout redesign (and the styles that were missing)

- **Bug found while redesigning**: the Brand page's card/slot/upload styles were never actually
  shipped (the SFC has no `<style>` block, so earlier patches silently did nothing) — logo previews
  and the palette rendered unstyled.
- **Palette**: now a live preview strip (button, link, four bars) plus a responsive grid of slot
  cards — a big colour tile per slot (click to pick), role title, one-line usage
  ("buttons, links, nav", "chart bar 2"…), and the hex field. ★ marks the primary colour.
- **Typography**: its own section with a live preview of the selected family ("Aa Bb 123 · สวัสดี"),
  font cards with a ✓ on the active one, and the import control as a dashed "+ Import font" tile.
- All brand-kit styles now live in `src/style.css` (same convention as the rest of the app).

## The Monthly Plan can be set up from the app

- **Why**: a production month starts empty and there was no way to create rows — the planner could
  only edit rows that already existed.
- **Planner**: "+ New topic" (header and empty state) creates a row with sensible defaults (first
  pillar/format/goal, first workflow status, the month's date, active platforms), selects it and
  opens the editor. The editor now also edits **topic, pillar, format, goal, date and time**, and
  rows can be **duplicated** or **deleted** (delete asks first; a row locked by someone else is
  refused, same as editing).
- **API**: `DELETE /api/posts/{id}` (permission `posts.write`, 409 when locked by another user),
  wired through the mock, the HTTP client and the OpenAPI spec.
- Tests: Rust 130, frontend 172 (new planner set-up suite).

## Planner vs Monthly Plan clarity

- **Why**: two month views look like duplicates — "Monthly Planner NN" (the editable master table, nav
  **Plan**) and "Monthly Plan" (the card overview, under **Calendar → Cards**). Both also defaulted
  to month 02 from the old wireframe.
- **Fix**: both now open the **current month** (`/planner` without a month resolves to it, nav "Plan"
  points there too), the cards view says "card view of the same plan — click a card to edit it in the
  planner" and has an **Open the planner** button that keeps the selected month.

## One month view: the planner

- **Why**: "Monthly Plan" (cards) and "Monthly Planner NN" showed the same rows in two places, which
  read as duplicates.
- **Change**: the card view is gone — `/monthly`, its nav/sheet entry, `MonthlyPage.vue`,
  `MonthCards.vue` and their strings were removed. The **Monthly Planner** (nav **Plan**) is the one
  month view and still opens the current month; the README/CHANGELOG keep the history.
- Tests: frontend 171 (the MonthCards-only test went with it).

## Feed Review: the platform switch actually switches

- **Bug**: the page watched the whole `setup` query object; any refetch (window focus, invalidation)
  returned a new object and the watcher reset the platform back to the navbar's one, so a chosen
  platform silently reverted and the preview looked unchanged.
- **Fix**: the watcher keys on the platform *names* and stops once the user picks a platform here.
- **UI**: the platform select became a tab strip with **planned-post counts** per platform
  (`Facebook 1 · Instagram 4 · TikTok 3 · Youtube 1`), the canvas line shows the live size, ratio and
  post count, and each preview tile carries a platform badge and the planned date — so switching is
  visible at a glance.
- Tests: 4 new Feed Review cases (counts, canvas switch, setup refetch does not reset, navbar still
  followed until a choice is made).

## Brand style tokens, Figma-style

- **Why**: only the palette and fonts were adjustable; everything else about the look was fixed —
  while a brand kit should let you change how the product itself looks, like Figma's inspector.
- **Model**: radius (0–24 px), fillOpacity (5–100%), strokeWidth (0–3 px),
  shadow (none/soft/strong) with safe defaults; the brand PATCH validates the ranges.
- **Theme**: radius retunes the three radius tiers, fill retunes solid accent surfaces
  (--accent-strong + matching ink by luminance), stroke retunes card/panel borders, shadow
  swaps the shadow tiers off/on/strong — all live, with CSS fallbacks when the brand is empty.
- **Style panel**: Figma-flavored numeric rows (label + slider + number + unit), a None/Soft/Strong
  segmented control, a live sample, and an **Export** row (Copy CSS / download brand-kit.json).
- Tests: Rust 131, frontend 178.

## Default identity, livecard palette cards, and a JSON round-trip

- **Default identity**: the workspace ships with identity artwork (`public/default-identity.svg`,
  814×827). The Logos section opens with an identity card — the uploaded main logo when there is
  one, otherwise the default artwork with a *Default identity* badge — and a one-click
  **Download default identity** link, so a brand-new workspace is never visually empty.
- **Palette cards**: rebuilt on the Live card pattern. The color itself is the media block (16:9)
  with the role chip top-left and a ★ badge on the primary slot; the body carries the usage caption,
  the hex field and a Copy button; the "Aa" sample picks black or white text by luminance so it
  stays readable on any swatch. Cards follow the stroke/radius/shadow tokens, lift on hover, and the
  grid gap is **20 px**.
- **Import JSON**: the Export row gains **Upload JSON** next to Download JSON. Imports are
  sanitized — known keys only, arrays filtered to non-empty strings, radius/fill/stroke clamped to
  the same ranges the API enforces, shadow limited to none/soft/strong — so a hand-edited or foreign
  file can never wedge the site. Invalid files show an inline error and change nothing.
- Also fixed: the four Style rows carried literal `{{ t(...) }}` aria-labels (Vue does not
  interpolate inside plain attributes); they now bind for real.
- Tests: frontend 181 (3 new: palette card structure, valid import, invalid import).
