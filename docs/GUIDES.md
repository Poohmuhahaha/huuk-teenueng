# Guides — making changes

How-to recipes for the common maintenance tasks, plus the traps that have already bitten this
codebase. If you only read one section, read **Gotchas**.

---

## Add a screen

1. Create `src/pages/MyPage.vue` (use an existing page as the template).
2. Add one entry to `src/core/screens.ts`:
   ```ts
   { path: '/mypage', to: '/mypage', label: 'My Page', match: ['/mypage'],
     component: MyPage, sheet: 'Sheet Name', access: 'computed' }
   ```
   Order in the array = order in the deck = order of the dots.
3. Nothing else. Routes and slides both come from the registry.

To add a nav link to the top bar, edit the `links` array in `AppShell.vue`
(grouped links use `match` prefixes).

## Add a table

Copy `src/components/TxnTable.vue`, then:

1. Register the features you need in `tableFeatures({...})` (sorting, filtering, selection…).
2. Type the columns as `ColumnDef<typeof features, YourRow, unknown>[]` — **features first**.
3. Pass `data` as a `computed` ref (`useTable({ features, columns, data })`).
4. Render with `<FlexRender :cell="cell" />`.
5. Add a case to `src/tests/tables.test.ts`.

Use TanStack Table **v9** APIs (`useTable`, `create*RowModel`, `sortFns/filterFns` slots).
`useVueTable`/`getCoreRowModel`/`sortingFns` are v8 and will not compile.

## Add a query or mutation

In `src/core/queries.ts`:

```ts
export function useMyThing() {
  return useQuery({ queryKey: qk.myThing, queryFn: api.getMyThing })
}
export function useSaveMyThing() {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: api.saveMyThing,
    onSuccess: () => qc.invalidateQueries({ queryKey: qk.myThing }),
  })
}
```

Add the actual fetch to `src/mock/api.ts`. **Never import `src/mock/` from a page/component.**
Symptom of a forgotten invalidation: the write succeeds but the UI doesn't refresh.

## Run against the real backend

A working Rust backend lives in `../server` (Axum + in-memory store, same contract as
`src/mock/api.ts`). See `../server/README.md` for the API reference.

```sh
cd ../server && cargo run            # http://localhost:8787
# production/.env.local
VITE_API_URL=http://localhost:8787   # restart `npm run dev`
```

`src/api/index.ts` switches on `VITE_API_URL`; `src/api/http.ts` is the client. Unset the var to
go back to the mock. When adding an endpoint: implement it in `server/src/handlers.rs`, mirror it
in `src/api/http.ts` **and** `src/mock/api.ts`, then test both sides (`cargo test` /
`npm run test`).

## Add a platform connection

1. Add the id to `PlatformId` in `src/mock/db.ts` and a seed row to `connections`.
2. Add its entry to `PLATFORM_META` in `src/platforms.ts` — name, `setupName` (must match the
   Set up platform list for feed filtering), auth label, `externalLabel`, scopes, `steps`
   (the pipeline shown in the login page), and a one-line `constraint`.
3. Add a login skin for it in `src/components/PlatformLogin.vue` (`SKINS`: host, product, field
   labels, default account, SSO label, 2FA hint).
4. That's it: the Social Media bar, login page, mock API and hooks are generic. The mock endpoints
   live in `src/mock/api.ts` (`connectPlatform` picks token type by id — adjust if the new platform
   differs, e.g. refresh tokens).
5. Add a case to `src/tests/platforms.test.ts`.

## Turn on real OAuth (Meta first)

Set credentials in the server environment and restart it — the login page switches from its
simulated form to a redirect to the provider automatically:

```sh
export META_APP_ID=... META_APP_SECRET=...            # Facebook + Instagram (one Meta app)
export GOOGLE_CLIENT_ID=... GOOGLE_CLIENT_SECRET=...  # YouTube
export TIKTOK_CLIENT_KEY=... TIKTOK_CLIENT_SECRET=... # TikTok
export PUBLIC_URL=http://localhost:8787 FRONTEND_URL=http://localhost:5173
```

Register `{PUBLIC_URL}/api/oauth/{platform}/callback` as the redirect URI in the provider console
(and request the scopes from `PLATFORM_META`). `GET /api/oauth/{platform}` reports the current mode.
The server prints `OAuth : real mode for …` on startup. Never collect provider passwords on our own
domain — `PlatformLogin.vue` hides the demo form in redirect mode for exactly that reason.

## Change who can do what (permissions)

Permissions are config, not code: `setup.roles` (Owner/Editor/Viewer seeds) × ten permission
strings. Edit them in Setup → "Auth & permissions" (matrix + Save, add/remove role) or via
`PATCH /api/setup {roles, authRequired}`. Code side:

- Server: `require_perm(store, headers, perm)` at the top of each mutating handler.
- Mock: `requirePerm(perm)` at the top of each mutating function in `src/mock/api.ts`.
- UI: `const { can } = usePermission()` → `:disabled="!can('posts.write')"` + `:title="t('auth.noPerm')"`.

`authRequired: false` (seed) keeps demo mode open; flip it on to enforce sessions + roles. When you
add a new mutating endpoint/mock function, add it to both enforcement points and to
`PERMISSIONS` in `src/mock/db.ts` if it is a new capability.

## Change the full-page animation

Everything lives in `setFull()` in `src/components/ScreensDeck.vue` (see
`ARCHITECTURE.md §3`). Rules learned the hard way:

- Animate **`width` / `left` on a pinned (`position: fixed`) card** — not `scale` (distorts the
  content) and not `flex-basis` (reflows the whole deck and snaps mid-flight).
- Keep the card's **height identical** in paged and full modes; never scale on Y.
- Fade the other cards with the `.deck.fading` class **during** the animation.
- Compute the exit target from the clamped paged slot (`maxScroll`/`snappedScroll`), or the first
  and last cards will jump when the layout swaps.
- Keep the `fullBusy` guard; stop the previous animation with `.stop()`.
- Keep `prefers-reduced-motion` fallback.

Tune duration/easing in one place: `{ duration: 0.56, ease: [0.22, 1, 0.36, 1] }`.

## Change how a layout responds to full page

Grids inside cards use container queries on `.slide-card` (`container-name: screen`).
Add/adjust rules in the container-query block at the **bottom** of `src/app/style.css`
(≥720 px → 4 columns, ≥1100 px → 6 columns for `grid4`/`grid6`/`cards`).
Paged cards must stay ≤ 3 columns.

## Add a UI string or language

Strings live in `src/i18n.ts` — the `en` and `th` dictionaries must keep **identical key sets**
(a test enforces parity). Add the key to both, then call `t('your.key')` inside the template (or a
computed) so Vue tracks the `lang` ref; for months use `monthName()`/`monthNameShort()`. Language is
a personal preference: the navbar `EN|TH` toggle calls `syncLang()` and stores `localStorage`
`cp.lang` (which wins over `setup.language`, the workspace default used on a first visit); it only
writes `setup.language` when the user has `setup.write`. There is no language control in Setup.

## Sessions and row locks

`src/auth.ts` owns the session (`login(email, password)`, `register`, `changePassword`, `updateProfile`,
`logoutAll`, `restore`, `logout`); the token persists in `localStorage` (`cp.token`) and rehydrates via
`/api/auth/me` on boot. Server-side, accounts live in the store with Argon2 hashes, sessions are
random 64-hex tokens valid for 30 days, and identity is the account (email) — display names are
unique across accounts. Every signed-in user can rename themselves in the avatar popup
(`ProfilePanel` → `PATCH /api/auth/profile`); the directory entry follows so the role survives.
Account administration
(`GET/DELETE /api/auth/accounts`, `users.manage`) lives in the "Workspace settings" popup and lets
an Owner revoke access without touching the
Setup directory; "Sign out everywhere" lives in the same avatar popup. Demo credentials:
`owner@studio.local` / `editor@studio.local`, password `demo1234`.
For row locks use `useLockPost(month)` / `useUnlockPost(month)` from `src/core/queries.ts`; with
`authRequired` on the server stores the **session's** account name as the holder (client-supplied
names are ignored), and the API (and mock) answer `409 post is locked by X` for anyone else;
`PlannerPage` mirrors that with a chip + disabled Save. To gate other UI on a session, check
`currentName` and route users through `LoginModal` (AppShell).

## Add or change tests

```sh
npm run test          # once
npm run test:watch    # while developing
```

Tests use `@vue/test-utils` + happy-dom. Example: `mount(MasterTable, { props: { rows } })`,
`await w.findAll('thead th')[1].trigger('click')`, assert on `w.findAll('tbody tr')`.
Prefer testing behaviour (sorting, filtering, emits) over markup.

---

## Gotchas (things that already broke)

### Deck / interaction

| Symptom | Cause | Fix / rule |
|---|---|---|
| Buttons/inputs in cards don't click | Pointer capture was taken on `pointerdown`, retargeting the click to the deck | Capture **only after** the drag threshold (`onMove`, > 8 px) |
| Deck feels stuck while dragging | Snap (`scroll-snap-type: mandatory`) + smooth scroll fight manual `scrollLeft` | Drag disables snap/smooth until release (`onMove` → `onUp`) |
| Entering full page rewrote the URL to `/setup` | Collapsed slides share `offsetLeft`; the settle handler routed to index 0; entering also clamps scroll | `commit()` bails when `full` is true; `nearestIndex()` skips zero-width slides |
| First card jumps on exit | The paged slot of card 0 is `deck.left + 7` (scroll can't go negative), not `12.5vw + 7` | Compute `pagedLeft` from `snappedScroll` (already done) |
| Tapping a peeking card also triggers its buttons | Click bubbled into the partially visible card | `onSlideClick` runs in **capture** phase and stops the event for non-active cards |
| Card slides on its own after a drag | Stray click after `pointerup` | `onClickCapture` swallows clicks when `moved`; `moved` resets on the next tick |

### TanStack Table v9

| Symptom | Cause | Fix |
|---|---|---|
| `useVueTable` / `getCoreRowModel` not found | v8 API | `useTable` + `tableFeatures` + `create*RowModel` |
| `Type 'X' has no properties in common with TableFeatures` | Generics order | `ColumnDef<typeof features, Row, unknown>[]` |
| Column doesn't sort / first click wrong direction | v9 infers direction from values (numbers descend-first) | Declare `sortFn: 'basic'` for numeric columns, or expect desc-first |
| Data stale after mount | Passed `data.value` | Pass the `computed`/ref itself to `useTable` |
| Type errors on row types | `interface` rows | Use `type` aliases (`RowData` needs an index signature) |

### Motion / animation

| Symptom | Cause | Fix |
|---|---|---|
| Content looks stretched ("ยืด") | `scaleX` on the card | Animate real `width` on a pinned card |
| Animation feels laggy | `clip-path` repaint, or animating layout on the whole page | Pin the card `position: fixed`; animate `width/left`; transform-only elsewhere |
| Card snaps after the animation | Target rect didn't match the final layout position | Compute targets from the actual clamped slot; swap layout in the same frame as unpinning |
| Animation plays twice / overlaps | Double click while animating | `fullBusy` guard; `fullAnim?.stop()` |

### Layout

| Symptom | Cause | Fix |
|---|---|---|
| Full-page content still 3 columns | Container query breakpoint not met (card too narrow) | Check card width ≥ 720 px, or adjust the `@container` block |
| Paged card got too many columns | Viewport media query vs container query | Container queries are last in the file and win — keep that order |
| Grid wider than the card | Missing `min-width: 0` in a flex child | Add it to the child |

### Data

| Symptom | Cause | Fix |
|---|---|---|
| State resets on reload | Mock store is in-memory | Expected in mock mode; swap `mock/api.ts` |
| Write doesn't appear elsewhere | Wrong/missing `invalidateQueries` | Invalidate the affected key(s) in `onSuccess`, e.g. posts **and** metrics |
| Finance amounts not `฿` | Currency is a constant in `FinancePage.vue` | Wire it from `Budget.status` when the backend lands |
| Edit blocked with "post is locked by …" | Another session holds the row lock | Unlock as the holder, or log in as them (avatar menu) |
| Action disabled with "no permission" | `authRequired` is on and the role lacks the permission | Edit the matrix in Setup → Auth & permissions, or log in as a role that has it |
| API answers 401 / 403 after enabling auth | Mutating call without a Bearer token / role lacks permission | Log in; `src/token.ts` feeds the header in `api/http.ts` |
| Thai labels don't switch | Key missing from the `th` dictionary, or `t()` called outside a template/computed | Add the key (parity test), call `t()` reactively |

---

## Code conventions

- `<script setup lang="ts">`, one component per file, PascalCase filenames.
- No CSS framework; class names come from `src/app/style.css`. No comments unless they explain "why".
- Keep the B&W look: use existing tokens/classes instead of new colors or shadows.
- Keep interactions cheap: no per-frame Vue reactivity for animations (use Motion), no polling.
- Run `npm run build` (types) **and** `npm run test` before calling a change done.
