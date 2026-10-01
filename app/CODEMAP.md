# CODEMAP — read the code in this order, fix it yourself

Goal: go from "it renders" to "I can change anything" in one pass.
Each step names the file, what you learn, and what breaks if you skip it.

> Deeper reference lives in `docs/`: **ARCHITECTURE** (how it works, incl. the deck and
> the full-page animation), **COMPONENTS** (file-by-file reference), **GUIDES** (how-tos
> and gotchas), **CHANGELOG** (what was built and why).

## 0. Run it first (5 min)

```sh
cd production
npm install  # (repo root `bun run dev` does backend + frontend together)
npm run dev         # open the URL it prints; slide every card
npm run test        # 11 tests — tables (render/sort/filter/select) + tab carousel
npm run typecheck   # vue-tsc only
npm run build       # typecheck + production bundle; must stay green
```

If `dev` works but `build` fails, the bug is a **type** error — read the first
`error TS...` line; it names file, line, and reason. Fix from the top down.

## 1. Entry — how the app boots (10 min)

| Order | File | What you learn |
|---|---|---|
| 1 | `index.html` | Mount point `#app`, page title |
| 2 | `src/app/main.ts` | Plugin order: router → TanStack Query (`QueryClient`) → mount |
| 3 | `src/app/App.vue` | Root = `AppShell` + `ScreensDeck`. Nothing else lives here |
| 4 | `src/app/style.css` | Every visual rule. Tokens at top (`--ink`, `--wash`…), then `.topbar`, `.tbl`, `.cards`, `.modal`, the deck (paged + full-page), viewport media queries, and **container queries last** |

Fix-it: wrong colors/spacing/layout → `style.css` only. Nothing else holds CSS.

## 2. Routes + the slide deck — the map (10 min)

5. `src/core/screens.ts` — **the registry**: slide order, paths, match prefixes, sheet metadata.
   Router routes and deck slides are both derived from it. Add a screen here, nowhere else.
   (`/planner/:month` is one screen; months are switched inside the page.
   The Guide is not here — the navbar button opens it as an overlay.)
6. `src/app/router/index.ts` — generates routes from the registry (hash history).
7. `src/components/ScreensDeck.vue` — **navigation is a card slide, not a page swap**:
   all screens render as bordered cards on one snap track; drag / touch / ← → / dots slide it
   one card at a time (`scroll-snap-stop: always` — it can never rest between cards).
   Desktop viewport is the **1:3:3:1** ratio: `[1 peek][3 card][3 card][1 peek]`
   (`.slide` = 37.5vw, `scroll-padding-inline` = 12.5vw; phone = `[3 card][1 peek]`).
   URL follows the settled card (`router.replace`), nav links slide to a card (`watch` route).
   Order of the deck = order in `screens.ts` (user journey J1–J10).
   Every card carries a **Go full page / Exit full page** button (top-right, rendered by the deck);
   full mode hides the other cards and the active card fills the viewport.
   The reveal is a **horizontal squeeze** (FLIP): the card animates `translateX` + `scaleX` only,
   via `animate()` from **motion-v** (Framer Motion's Vue engine) — never scales on Y.

Fix-it: blank page / 404 → a path in `screens.ts` is misspelled or a page import is wrong.
Wrong slide order → reorder `screens`. Slide not snapping → `.deck`/`.slide` CSS.
Nav click doesn't move the deck → the route's `match` prefix doesn't match its path.

## 3. Data, bottom-up (20 min)

Read in this order — each layer only talks to the one below it:

| Order | File | What you learn |
|---|---|---|
| 7b | `src/api/index.ts` | Data-source switch: mock by default, HTTP when `VITE_API_URL` is set (Rust backend in `../server`) |
| 7c | `src/auth.ts` + `src/token.ts` | Real auth: `login(email, password)`, `register`, `changePassword`, `logoutAll`, `restore()`; token persisted in `localStorage`, attached by `api/http.ts`; token module separate to avoid the http↔auth import cycle |
| 7d | account admin | `GET/DELETE /api/auth/accounts` (Workspace settings popup, `users.manage`) — list name/email/sessions, delete revokes sessions; directory entry stays |
| 7d | `src/i18n.ts` | EN/TH dictionaries (identical keys), `t()`, `monthName()`; language is a personal pref in `localStorage` (`cp.lang`), seeded from `setup.language`, toggled in the navbar |
| 7e | Profile & setup popups | avatar click → compact popup (`ProfilePanel`: rename) + "Workspace settings" popup (`SettingsPanel`); neither is a deck card; register success reopens the popup with a welcome banner |
| 7e | `usePermission()` in `src/core/queries.ts` | `can('posts.write')` — demo mode open, otherwise role → permissions from `setup.roles`. Enforcement twins: `require_perm` (server) / `requirePerm` (mock) |
| 8 | `src/mock/db.ts` | Domain **types** (`Post`, `Txn`, `Status`…) + **seed data**. `Post`/`Txn` are `type` aliases (not `interface`) — TanStack Table v9 requires that |
| 9 | `src/mock/api.ts` | Async functions over the store (`listPosts`, `updatePost`, `promoteIdea`…). Returns copies like a real API. **This file is the only one you replace when the backend arrives** |
| 10 | `src/core/queries.ts` | TanStack Query hooks (`usePosts`, `useUpdatePost`…). Pages must never import `mock/` directly. Mutations invalidate their query keys — stale UI almost always means a missing invalidation here |

Fix-it: wrong seed values → `db.ts`. UI doesn't refresh after save → the mutation in `queries.ts` isn't invalidating the right key. API-shaped problem → `api.ts`.

## 4. Components — small to big (30 min)

Primitives first (no logic, just props):

11. `Chip.vue`, `KpiCard.vue`, `StatusChip.vue` — presentational only.
12. `CarouselTabs.vue` — M0 snap tabs; `v-model` in, `update:modelValue` out; slide/drag/arrows.
13. `StatusFunnel.vue`, `CopyBar.vue` (clipboard), `ProtectedModal.vue` (Cancel-default guard),
    `LoginModal.vue` (pick a user from `setup.users` → `login()`), `PlatformLogin.vue` (full-screen
    social login: chrome → form → 2FA → consent → pipeline → connected).
14. Display blocks: `BarChart.vue`, `BudgetCards.vue`, `FeedGrid.vue` (per-platform aspect ratios),
    `HashtagGroups.vue`, `GuideHero.vue`,
    `CalendarGrid.vue` (date math lives here — `lead` blanks, `days in month`, dots).

Then the two hard ones — read slowly:

15. `MasterTable.vue` — TanStack Table v9 pattern used everywhere:
    `tableFeatures({ rowSortingFeature, sortedRowModel, columnFilteringFeature, filteredRowModel })`
    → `ColumnDef<typeof features, Post, unknown>[]` (**TFeatures first** — the #1 gotcha)
    → `useTable({ features, columns, data })` with `data` as `computed`, never `.value`.
    Sorting/filtering state is internal; template reads (`getHeaderGroups`, `getRowModel`) are reactive.
    Checkboxes are hand-rolled (`checked: Set`, reassigned — never mutated in place).
16. `TxnTable.vue` — same pattern, smaller. Note the amount column declares `sortFn: 'basic'`
    because v9 infers first direction from values (**numbers go descending-first** — see test).
17. `AppShell.vue` — navbar (active link via `isActive`), EN|TH toggle (a `saveSetup` mutation →
    `syncLang`), avatar menu (session user, Log in/out → `LoginModal`), full-screen `.shellgrid`
    holding the `ScreensDeck`.

Fix-it: table won't sort/filter → `features` registration or column def in that table file.
Checkbox UI stale → the `Set` was mutated instead of reassigned. Wrong active nav →
`isActive`/`match` in `AppShell.vue`. Card border/paging → `.slide`, `.slide-card`, `.deck` in `style.css`.

## 5. Pages — in journey order (30 min)

Read in user order, not alphabetical — each page reuses patterns from the last:

18. `ProfilePanel.vue` + `SettingsPanel.vue` (avatar popups: profile rename, year/owner/lists,
    users, accounts, permissions). The Guide overlay lives in `AppShell.vue` (`GuideHero` in
    `.guide-overlay`, opened from the avatar popup).
    button — not a deck card).
19. `BrandPage.vue` (`:value` + `@change`-save pattern) → `DashboardPage.vue` (first `useDashboard()` aggregation + guard modal → route jump)
20. `CalendarPage.vue` (filters + `CalendarGrid`) → `PlannerPage.vue` (month switch inside the page)
21. `PlannerPage.vue` — the big one: local month ref (prop initialises it) → `usePosts(m)` → `MasterTable @select`
    → local `draft` copy → `Save` mutation; assembled Copy-Paste is a `computed`; platform chips
    toggle the draft array; guard modal demo on the computed field; row lock chip + Lock/Unlock
    (`useLockPost`/`useUnlockPost`) and Save disabled when another user holds the row.
22. `FeedPage.vue` (two month-queries combined, date/platform filter, canvas size + variant → `FeedGrid`)
23. `IdeasPage.vue` (toggle + promote flow) → `HashtagsPage.vue` (thin wrapper over component)
24. `PerformancePage.vue` (`plus7` follow-ups, goal deltas, monthly views, **Import from API** panel)
25. `FinancePage.vue` (budget sums, monthly IN/OUT charts, `TxnTable` + add form)

Fix-it: page shows old data → check its `useQuery` key and the invalidating mutation.
Editor edits vanish → you bound inputs to query data instead of the local `draft` (see PlannerPage).
Wrong month → the `month` ref in PlannerPage.

## 6. Verify like CI does

```sh
bun run test      # 59 tests: tables, carousel, platform bar, feed canvas, i18n, auth, collab, permissions
bun run build     # types + bundle; ship only on green
cargo test        # in ../server — 33 API integration tests incl. auth, permissions, OAuth
```

New-table checklist (copy `TxnTable.vue`): `tableFeatures` with what you need →
`ColumnDef<typeof features, Row, unknown>[]` → `data` as `computed` → template reads
`getHeaderGroups()`/`getRowModel()` → add a test in `src/tests/`.

## Cheat sheet

| Symptom | Open this |
|---|---|
| Colors/spacing/responsive | `src/app/style.css` |
| Blank/404 route | `src/app/router/index.ts` |
| Wrong seed content | `src/mock/db.ts` |
| Save doesn't refresh UI | `src/core/queries.ts` (invalidation) |
| Table misbehaves | that table's `features` + column defs |
| Wrong nav/sheet label | `src/components/AppShell.vue` |
| Thai text missing / not switching | `src/i18n.ts` (key parity) — `t()` must be called in a template/computed |
| Save blocked on a row | That row's `lockedBy` (US-014) — unlock as the holder, see `src/auth.ts` |
| Action disabled "no permission" / API 401-403 | `authRequired` + `setup.roles`; enforcement in `require_perm` (server) / `requirePerm` (mock); UI via `usePermission()` |
| Can't log in | Real auth now: email + password (`owner@studio.local` / `demo1234` seeded); register in `LoginModal`; accounts live in the server store (Argon2), directory roles in `setup.users` |
| Type error on build | first `error TS` line, top-down |
| Behavior question | `src/tests/tables.test.ts` — the executable spec |
