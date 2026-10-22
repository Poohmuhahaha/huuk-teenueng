# Component & page reference

Every file under `src/` and what it owns. "Sheet" is the workbook sheet the screen transcribes.

## Shell & navigation

| File | Responsibility | Notes |
|---|---|---|
| `app/App.vue` | Renders `AppShell` + `ScreensDeck` | No logic |
| `components/layout/AppShell.vue` | Navbar (Plan…Finance), EN\|TH toggle, **Sign up / Log in** buttons when logged out, avatar **profile popup** (`ProfilePanel` + Workspace settings / Guide / Change password / Sign out everywhere / Log out), workspace **settings popup** (`SettingsPanel`), guide overlay, footer | Guide and settings are **overlays**, not cards. Popup closes on outside click / `Esc`; opens `LoginModal` in the chosen mode. Register success reopens the profile popup with a welcome banner |
| `ProfilePanel.vue` | Avatar popup content: own name (editable, `PATCH /api/auth/profile`), role, Save/saved/error, login hint | Used inside the avatar popup; no props |
| `SettingsPanel.vue` | Workspace settings popup content: Year/Owner, option lists, users, accounts (name/email/live sessions, remove), Auth & permissions matrix; read-only hint when signed in without rights | Former `SetupPage` minus the profile panel; no props |
| `LoginModal.vue` | Email + password auth dialog with login/register modes | Props `open`, `initialMode`; emits `close`, `success(mode)`; calls `login()`/`register()` from `src/core/auth.ts`; demo hint |
| `ChangePasswordModal.vue` | Rotate the password (current + new + confirm) | Props `open`; emits `close`; calls `changePassword()`; server revokes other sessions |
| `PlatformLogin.vue` | Full-screen social login page: browser chrome → platform login form → 2FA → consent (scopes) → pipeline → connected summary | Props `platform: PlatformId \| null`; emits `close`; per-platform skins live in `SKINS`; gated on `platforms.manage` |
| `components/layout/ScreensDeck.vue` | The slide deck: all screens as cards, drag/keys/dots, URL sync, full-page animation | See `ARCHITECTURE.md §2–3` |
| `components/ui/CarouselTabs.vue` | M0 snap tab strip (months 01–12, weeks W1–W6) | `v-model`; supports click, drag (40 px threshold), ← → keys |
| `core/screens.ts` | Screen registry (path, label, match prefixes, component, sheet metadata) | Feeds router + deck. Add screens **only here** |
| `app/router/index.ts` | Routes derived from `screens.ts` (hash history) | `/` → first deck card preserving query; `/profile` and `/setup` aliases redirect to `/` (OAuth callback landing) |

## Shared components (`components/ui/`, `components/tables/`, `components/overlays/`, `components/editor/`, `components/live/` — barrel: `@/components/*`)

| File | Props | Emits | Used by |
|---|---|---|---|
| `Chip.vue` | `label`, `dark?` | — | Everywhere (tags, statuses) |
| `KpiCard.vue` | `label`, `value` | — | Dashboard, Finance |
| `StatusChip.vue` | `status` | — | MasterTable |
| `StatusFunnel.vue` | `current` | `advance(status)` | Planner row editor |
| `CopyBar.vue` | `text` | — | Planner (assembled caption + CTA + tags, one-click copy) |
| `ProtectedModal.vue` | `open`, `field` | `cancel`, `confirm` | Dashboard, Planner (computed-field guard) |
| `BarChart.vue` | `items: {label,value}[]` | — | Dashboard, Performance, Finance |
| `BudgetCards.vue` | `income`, `expense`, `balance`, `currency` | — | Finance |
| `CalendarGrid.vue` | `year`, `month`, `posts`, `weekStart`, `showPillar/Platform/Status` | — | Calendar |
| `CardBase.vue` (`ui/`) | `title`, `meta?`, `coverLabel?`, default slot | — | Planner cards |
| `FeedGrid.vue` | `posts`, `platform: PlatformId`, `variant?: number` | — | Feed (canvas per platform: aspect-ratio tiles, wide ratios span 2 columns) |
| `HashtagGroups.vue` | `groups` | `add(groupId, tag)` | Hashtags |
| `GuideHero.vue` | — | — | Guide overlay (system map + steps + warnings) |
| `MasterTable.vue` | `rows: Post[]`, `loading?` | `select(post)` | Planner |
| `TxnTable.vue` | `rows: Txn[]`, `currency` | — | Finance |

### Table components (TanStack Table v9)

Both follow the same pattern — copy it when adding a table:

```ts
const features = tableFeatures({
  rowSortingFeature, sortedRowModel: createSortedRowModel(),
  sortFns: { alphanumeric: sortFn_alphanumeric },
  // + columnFilteringFeature, filteredRowModel, filterFns (MasterTable only)
})
const columns: ColumnDef<typeof features, Row, unknown>[] = [ /* TFeatures first! */ ]
const data = computed(() => props.rows)
const table = useTable({ features, columns, data })   // computed, not data.value
```

Template reads `table.getHeaderGroups()`, `table.getRowModel().rows`, `row.getAllCells()`,
and renders with `<FlexRender :cell="cell" />` / `:header="header"` (v9 shorthand).
`MasterTable` adds: topic search (column filter), checkbox selection (`Set` **reassigned**, never
mutated), and `select` emit on row click. `TxnTable` declares `sortFn: 'basic'` on the amount
column (v9 infers a descending-first direction for numeric columns).

## Pages (one per screen / sheet)

| Page | Route | Sheet | Read/write | Highlights |
|---|---|---|---|---|
| `BrandPage.vue` | `/brand` | Brand Identity | write | Identity fields, logo slots, do/don't rules, palette, fonts, moodboard placeholder |
| `PlannerPage.vue` | `/planner/:month` | 01–12 | **write master** | Month ref (prop initialises), MasterTable → local `draft` → save mutation; row editor (hook/caption/CTA/tags), status funnel, platform chips, assembled CopyBar; guard modal demo; **row lock** chip + Lock/Unlock (save blocked when held by another user) |
| `CalendarPage.vue` | `/calendar` | Smart Calendar | read | Month tabs, Sun/Mon start, 4 filters, display toggles, `CalendarGrid`, gap/overload hint |
| `FeedPage.vue` | `/feed` | Feed Review | read | Platform + show-up-to-date + **canvas size** (primary/variants), 9-post preview with real aspect ratios |
| `DashboardPage.vue` | `/dashboard` | Dashboard | read | Month tabs, 6 KPIs, planned-vs-posted bars, platform/status splits, Top-5 with guarded Edit → Planner |
| `PerformancePage.vue` | `/performance` | Performance | semi | **Import metrics from API** (platform + month), platform goal deltas, views by month, per-post +7-day table |
| `IdeasPage.vue` | `/ideas` | Content idea Bank | write | Backlog table, done toggle, promote → dated Planner row |
| `HashtagsPage.vue` | `/hashtags` | Hashtag # | write | Grouped tag sets, add tag |
| `FinancePage.vue` | `/finance` | Finance | write | Budget cards, monthly IN/OUT charts, ledger (`TxnTable`), transaction form |

`GuidePage.vue` was removed when the Guide became an overlay. `PlatformConnectModal.vue` was
replaced by the full-screen `PlatformLogin.vue`. Every write control is gated by `can(...)` (see
`ARCHITECTURE.md §9`).

## Composable / data layer

| File | Contents |
|---|---|
| `src/i18n.ts` | `Lang`, reactive `lang`, `syncLang()`, `en`/`th` dictionaries (identical keys), `t(key)`, `monthName(m)`, `monthNameShort(m)` |
| `src/auth.ts` | Real auth: `login(email, password)`, `register({name, email, password})`, `changePassword(old, new)`, `updateProfile(name)`, `logoutAll()`, `restore()`, `logout()`; exposes `currentUser`, `isLoggedIn`, `currentName`, `currentRole`; re-exports `token` from `src/token.ts` |
| `src/token.ts` | Bearer token ref persisted to `localStorage` (`cp.token`); `setToken()`; separate module so `api/http.ts` can read it without an import cycle |
| `src/platforms.ts` | `PlatformId`, `PlatformMeta` (auth labels, scopes, pipeline steps, constraints, **feed canvas sizes + variants**), `PLATFORM_META`, `PLATFORMS`, `platformIdBySetupName()`, `activePlatforms` (checkbox list), `togglePlatform()`, `activePlatform` computed |
| `src/core/queries.ts` | `qk` key factory; `useSetup`, `useSaveSetup`, `useAddOption`, `useAddUser`, `useRemoveUser`, `useAddRole`, `useRemoveRole`, `useAccounts`, `useDeleteAccount`, `useUpdateProfile`, `usePermission`, `usePosts`, `useUpdatePost`, `useAddPost`, `useLockPost`, `useUnlockPost`, `useIdeas`, `useAddIdea`, `useToggleIdea`, `usePromoteIdea`, `useTags`, `useAddTag`, `useMetrics`, `useImportMetrics`, `useTxns`, `useAddTxn`, `useBrand`, `useSaveBrand`, `useDashboard`, `usePlatforms`, `useConnectPlatform`, `useDisconnectPlatform`, `useSyncPlatform` |
| `src/mock/api.ts` | Async mock endpoints (incl. auth/users/roles/locks/import); every mutation starts with `requirePerm(perm)`; swap for HTTP client |
| `src/api/http.ts` | HTTP client hitting the Rust server; same signatures; attaches `Authorization: Bearer` to mutations from `src/token.ts` |
| `src/mock/db.ts` | Types (`Post` with `lockedBy`, `SetupConfig` with `users`/`authRequired`/`roles`, `User`, `Role`, `Permission` + `PERMISSIONS`, `Idea`, `HashtagGroup`, `Metric`, `Txn`, `Brand`, `PlatformGoal`, `PlatformConnection`, `ConnectStatus`, `PlatformId`), helpers (`plus7`, `weekOfMonth`, `uid`, `fmtNum`), constants (`MONTHS`, `MONTH_NAMES`), seed data |

## Styles

`src/app/style.css` — single global stylesheet. Sections: tokens → app frame → primitives → tables →
cards/feed → modal → deck (paged + full mode) → shell grid → viewport media queries → container
queries. Read `ARCHITECTURE.md §5` before adding rules.
