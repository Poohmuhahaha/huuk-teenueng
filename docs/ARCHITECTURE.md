# Architecture

How the app is put together. For file-by-file reading order see [`../app/CODEMAP.md`](../app/CODEMAP.md).

```
index.html
└─ src/app/main.ts            app boot: router + VueQueryPlugin
   └─ src/app/App.vue         AppShell + ScreensDeck
      └─ src/components/AppShell.vue   navbar / guide overlay / avatar menu
         └─ src/components/ScreensDeck.vue  the card deck (all screens)
            └─ src/pages/*.vue             one page per screen
               └─ src/components/*.vue     shared UI pieces
                  └─ src/core/queries.ts        TanStack Query hooks
                     └─ src/api/index.ts   data-source switch (mock ↔ HTTP)
                        ├─ src/mock/api.ts async mock API (default)
                        ├─ src/api/http.ts HTTP client → Rust backend (../server)
                        └─ src/mock/db.ts  domain types + seed data
```

## 1. Data flow (bottom-up)

| Layer | File | Rules |
|---|---|---|
| Types + seed | `src/mock/db.ts` | `Post`, `Txn`, `Metric`… are **type aliases** (required by TanStack Table v9's `RowData`); seed mirrors the workbook |
| Data source | `src/api/index.ts` | Picks mock or HTTP from `VITE_API_URL` (unset = mock). Both modules expose the same functions, so pages never change |
| Mock API | `src/mock/api.ts` | Async functions over the in-memory store (`listPosts`, `updatePost`, `promoteIdea`…) — the default, and the interface the Rust backend mirrors |
| HTTP client | `src/api/http.ts` | Same signatures, calls the Rust backend (`../server/README.md`). Enable with `VITE_API_URL`, e.g. `http://localhost:8787` |
| Query hooks | `src/core/queries.ts` | All reads/writes go through here (`usePosts`, `useUpdatePost`, `useDashboard`…). Mutations invalidate their query keys |
| Pages | `src/pages/*.vue` | Never import `src/mock/` directly. Bind to query data; edit local drafts, then mutate |

Query keys live in `qk` (`src/core/queries.ts`). **Stale UI after a write is almost always a missing invalidation** in the mutation's `onSuccess`.

## 2. Screens & navigation

`src/core/screens.ts` is the single registry:

```ts
{ path: '/brand', to: '/brand', label: 'Brand', match: ['/brand'],
  component: BrandPage, sheet: 'Brand Identity', access: 'standalone' }
```

- `src/app/router/index.ts` derives all deck routes from it (`/` redirects to the first card preserving
  query params; `/profile` and `/setup` are aliases that redirect to `/`, kept for links and the
  OAuth callback).
- **Profile & setup is not a card**: it opens from the avatar as a compact popup (`ProfilePanel`)
  plus a full "Workspace settings" popup (`SettingsPanel`).
- `ScreensDeck.vue` derives the slide order from it (10 cards, journey order J1–J10).
- The **Guide is intentionally not a card** — the avatar-menu Guide entry opens `GuideHero` in an
  overlay (`AppShell.vue`).

### Deck navigation model

The deck is a native horizontal scroll container with CSS snap:

```
desktop: [1 peek][3 card][3 card][1 peek]     phone: [3 card][1 peek]
.slide    flex-basis 37.5vw (75vw ≤640px)      one card = 3 columns of the 1:3:3:1 viewport
.deck     scroll-padding-inline 12.5vw (0 ≤640px)   creates the 1-column peeks
          scroll-snap-stop: always                  one gesture = exactly one card
```

Two-way sync between scroll and URL:

- **Route → deck:** `watch(route.path)` → `screenIndexOf()` → `scrollToIndex(i)` (smooth).
- **Deck → route:** on scroll settle (150 ms debounce) `commit()` → `router.replace(screen.to)`.

Guards you must keep if you touch this code:

| Guard | Why |
|---|---|
| `syncing` flag | ignore scroll events caused by programmatic scrolling (no route ping-pong) |
| `commit()` bails when `full.value` | entering full page clamps the scroll and would otherwise rewrite the URL to card 0 |
| `nearestIndex()` skips `offsetWidth === 0` slides | collapsed slides share an `offsetLeft` and would win the tie |
| `scrollToIndex()` clamps at 0 and uses `padOf(el)` | first card sits flush left; others sit at the 1-column peek |

Inputs: drag/touch (`onDown/onMove/onUp`), ← → keys, dots, navbar links, tapping a peeking card
(`onSlideClick`, capture phase so its inner controls don't fire), `Escape` exits full page.

**Drag gotcha:** pointer capture is deliberately deferred until the pointer moves > 8 px.
Capturing on `pointerdown` retargets the subsequent `click` to the deck, which breaks every
button/input inside the cards. Do not "fix" this by capturing earlier.

## 3. Full-page animation (the important one)

Triggered by the top-right **Go full page / Exit full page** button on every card
(`ScreensDeck.vue` → `setFull(on)`).

What it does, in order:

1. `fullBusy` guard (no re-entry), stop any previous animation.
2. Measure `first = card.getBoundingClientRect()` and the deck rect.
3. Compute the `target` rect:
   - **enter:** `{ left: deck.left + 6, width: deck.width - 12 }`
   - **exit:** the card's exact paged slot, taking scroll clamping into account
     (`maxScroll` / `snappedScroll`) so the first card isn't pulled right and end cards land flush.
4. **Pin** the card: `position: fixed` at its current rect, `z-index: 20`, `will-change: width, left`.
   Pinning isolates reflow to the card itself — the rest of the page never re-lays out.
5. Fade the other cards **at the same time**:
   - enter → `deck.classList.add('fading')` (CSS: `.deck.fading .slide:not(.active) { opacity: 0 }`)
   - exit → `full.value = false` + `nextTick` + `scrollToIndex(..., false)` immediately, so the
     others fade in while the card shrinks.
6. `await raf()` so the pinned state paints first (no flash), then animate with Motion:
   ```ts
   animate(card, { left: [first.left, target.left], width: [first.width, target.width] },
     { duration: 0.56, ease: [0.22, 1, 0.36, 1] })
   ```
   Real `width` is animated on purpose: the content **reflows with the card** (no horizontal
   distortion), which is the "natural" look the design settled on.
7. On finish: enter sets `full.value = true` (collapses the others, already faded), then unpin and
   clear all inline styles. Exit already released the layout in step 5.
8. `prefers-reduced-motion` (or missing Motion) → instant toggle, same guards.

**Height is identical in both modes** — the animation is horizontal only. Never add vertical scaling.

### Layout adapts with the card (container queries)

`.slide-card` is a container (`container-type: inline-size; container-name: screen`).
The grids respond to the *card's* width, not the viewport, so they change mid-animation:

| Card width | Effect |
|---|---|
| narrow (paged, ~37.5vw / 75vw phone) | grids capped at **3 columns** (the agreed paged look) |
| ≥ 720 px (full page, small screens) | `.grid4`/`.grid6`/`.cards` → **4 columns** |
| ≥ 1100 px (full page, desktop) | `.grid6`/`.cards` → **6 columns** |

The full-page viewport is the 8-column system (`1:3:3:1` total); grid counts (2/3/4/6) all divide
into 8. If you add a new grid class, add its full-page rule to the container query block at the
bottom of `src/app/style.css`.

## 4. Layout system

- Global shell is full-screen (`.shellgrid`, no max-width). The paged viewport shows the
  **1:3:3:1** ratio: 1-col peek · 3-col card · 3-col card · 1-col peek.
- `.grid2 / .grid3` are literal 2/3-column grids; `.grid4 / .grid6` are **capped at 3** when the
  card is narrow and expand via the container queries above.
- Breakpoints (viewport): `≤860px` grids drop to 2, `≤560px` to 1; `.slide` becomes 75vw and the
  scroll padding disappears at `≤640px`.

## 5. Styling

All CSS lives in `src/app/style.css` (no CSS framework, no scoped styles beyond inline layout hints):

- Tokens at the top: `--ink`, `--muted`, `--faint`, `--wash`, `--line`, `--radius`.
- B&W only. Do not introduce color.
- Sections in order: app frame/topbar → primitives (buttons/fields/chips) → tables → cards/feed →
  modal → deck (including full-page mode) → shell grid → media queries → **container queries**.

## 6. Testing

```
src/tests/tables.test.ts    # MasterTable + TxnTable (render, sort, filter, select, checkbox)
src/tests/carousel.test.ts  # CarouselTabs (drag threshold, ends, arrow keys)
src/tests/platforms.test.ts # Social Media bar + mock platform api
src/tests/feedgrid.test.ts  # platform canvas sizes + rendered aspect ratios
src/tests/i18n.test.ts      # dictionary parity, month switching, t() fallback
src/tests/collab.test.ts    # auth, users, locking, metric import (mock parity)
```

Run with `npm run test`. Tests are the executable spec for tricky behaviour — add cases there when
you change tables, the tab carousel, or the data contract. (The deck/card animation needs a real
browser and is verified manually.)

## 7. Known limits (wireframe scope)

- Data is in-memory: state resets on reload/restart; sessions are server-side in memory; no realtime
  between clients; row locks have no TTL.
- Platform OAuth is simulated (see §8) and metric import generates deterministic pseudo-values.
- i18n translates navigation, Setup, Guide and month names; long body copy stays English.
- Language: `src/i18n.ts` dictionaries + the browser's `cp.lang` preference (seeded from
  `setup.language` on first visit) — adding a language means adding a third dictionary with
  identical keys.

## 8. Platform connections (wireframe OAuth)

The navbar platform switcher lets the user pick the working platform and see/maintain its
connection. The flow is credential-optional: with no OAuth credentials in the environment the login
page runs its **mock** simulation; with credentials the server returns a real authorize URL and the
browser completes OAuth **on the provider's own domain** (see `server/README.md § Real OAuth`).

| Layer | File | Contents |
|---|---|---|
| Metadata + UI store | `src/platforms.ts` | `PLATFORM_META` per platform (display name, `setupName`, auth label, external-id label, scopes, pipeline steps, constraints), the `activePlatforms` checkbox list, `togglePlatform()` (min 1, registry order), and the `activePlatform` computed (first checked = primary) |
| Mock state | `src/mock/db.ts` | `PlatformId`, `ConnectStatus`, `PlatformConnection`, seeded `connections` (Instagram/Facebook connected; YouTube/TikTok disconnected) |
| Mock API | `src/mock/api.ts` | `listPlatforms`, `connectPlatform` (sets long-lived/refresh token + expiry), `disconnectPlatform`, `syncPlatform` (bumps `lastSync` + `mediaCount`) |
| Hooks | `src/core/queries.ts` | `qk.platforms`, `usePlatforms`, `useConnectPlatform`, `useDisconnectPlatform`, `useSyncPlatform` |
| Social Media bar | `src/components/SocialMediaBar.vue` | Horizontal checkbox row below the navbar (`Social Media` label), one toggle per platform with status dot; clicking a platform name opens its connection modal |
| Connect flow | `src/components/PlatformLogin.vue` | Full-screen login page. **Mock mode**: browser chrome → demo login form (per-platform labels + SSO) → 2FA → consent → animated pipeline → connected summary. **Redirect mode**: the same chrome shows a "Continue to …" button (credentials are entered on the provider's page), then `window.location` takes the browser to the authorize URL; the callback returns to `/#/profile?oauth=…&status=…` (via the `/setup` alias with the query preserved) where `AppShell` shows a notice and refreshes the connections. Gated on `platforms.manage` |

`activePlatform` is global state by design: Feed Review follows it today (its platform selector
syncs to the primary checked platform). If another page should follow it, import the computed and
`watch` it. `activePlatforms` holds every checked platform (first checked = primary).

**Real backend later:** reimplement the four `*Platform(s)` functions in `src/mock/api.ts` — the
`steps` array in `PLATFORM_META` maps 1:1 to the OAuth/worker stages, and `PlatformConnection`
already carries the fields a real integration needs (`externalId`, scopes, token type, expiry,
last sync, media count). The modal's simulation becomes the real redirect + callback handling.

## 9. i18n, sessions, permissions and row locks

Cross-cutting features every screen can rely on:

- **Language (`src/i18n.ts`)** — module-level `lang` ref, `en`/`th` dictionaries (key parity enforced
  by a test), `t(key)`, `monthName(m)`, `monthNameShort(m)`. The navbar `EN|TH` toggle is a
  **personal preference**: it calls `syncLang()` immediately and persists to `localStorage`
  (`cp.lang`), which then wins over `setup.language` (the workspace default used on a first visit).
  The toggle only writes `setup.language` when the user holds `setup.write`. Templates call
  `t()`/`monthName()` directly so Vue tracks the ref.
- **Auth (`src/auth.ts` + `src/token.ts`)** — real email + password SaaS auth: `login(email, password)`,
  `register({name, email, password})`, `changePassword(old, new)`, `updateProfile(name)`, `logoutAll()`,
  `restore()`, `logout()`. The server stores Argon2 hashes (accounts are internal — never serialized) and issues
  random 64-hex session tokens with a 30-day TTL. **Identity is the account**: sessions key on the
  email and permissions resolve email → name → directory role; display names are unique across
  accounts (duplicate → 409 `name already taken`), so two accounts cannot share a role. The token is
  persisted in `localStorage` (`cp.token`) and rehydrated on boot via `restore() → GET /api/auth/me`;
  the token module exists so `api/http.ts` can attach `Authorization: Bearer` without an import
  cycle. `LoginModal` hosts login/register; `ChangePasswordModal` (avatar popup) rotates the password
  and revokes other sessions; "Sign out everywhere" (`POST /api/auth/logout-all`) revokes them all.
  The workspace settings popup lists registered accounts (`GET /api/auth/accounts`, `users.manage`) with live
  session counts and can delete one (`DELETE /api/auth/accounts/{email}`) — deleting revokes its
  sessions while the directory entry stays. Seeded demo accounts: `owner@studio.local` /
  `editor@studio.local`, password `demo1234`. `SetupConfig.users` remains the role directory
  (name + role), not the credential store.
- **Profile (`PATCH /api/auth/profile`)** — any signed-in account can rename itself from the avatar
  popup (`ProfilePanel.vue`); the matching directory entry is renamed with it so the role survives,
  and the token/session is untouched. Registering opens the popup with a welcome notice.
- **Permissions (`setup.roles` + `usePermission()`)** — `Role = {name, permissions[]}` with ten
  permissions spanning setup/users/posts/locks/import/finance/brand/ideas/hashtags/platforms.
  `usePermission().can(perm)` returns `true` when `authRequired` is off (demo mode), otherwise it
  resolves the current user's role. Pages disable their write actions and title them
  "no permission". The Rust server and the mock both enforce it: 401 without a session, 403 without
  the permission. Setup hosts the editor: require-login toggle, roles × permissions matrix, add and
  remove role.
- **Row locks (US-014)** — `Post.lockedBy`, independent of the permission system. The server
  enforces it (`409 post is locked by <user>` on PATCH from anyone else; lock/unlock are idempotent
  for the holder). **When `authRequired` is on the holder is derived from the session**, never from
  the client-supplied `user`, and creating a post ignores any `lockedBy` payload; in demo mode the
  wireframe keeps the client name. PlannerPage mirrors the state: chip, disabled Save when locked
  by others, Lock/Unlock button (needs `posts.lock`). `useLockPost`/`useUnlockPost` invalidate the
  month.
