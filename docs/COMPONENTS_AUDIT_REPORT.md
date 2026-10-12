# Components Audit Report — Huuk (41 components × dual-agent)

**Date:** 2026-10-10 · **Scope:** `app/src/components/` — 41 `.vue` files
**Method:** 7 groups × 2 agents (UX perspective A + Technical perspective B) = 14 agents / 82 audit-slots. One merged report.
**Groups:** G1 charts(8) · G2 dashboard(7) · G3 common(11) · G4 overlays(8) · G5 layout(3) · G6 tables(2) · G7 editor+live(2)
**Design baseline:** `AGENTS.md` + `app/README.md:63-92` (Tailwind v4 `@theme` `var(--*)`, type `text-2xs–5xl` @ `line-height:1.6` else exact `text-[..]`, radii only `rounded-none`/`rounded-full`, `border` always +color, every `<button>` `w-full`/`w-auto`, no `<style>` except `@keyframes`/`@container`, `h1–h4` + `text-*` ⇒ `leading-*`, UNLAYERED hooks `deck/slide/spacer/slide-card/card-scroll/grid*/tbl/row/*-page/*-scroll/p-drawer*` kept bare, TanStack v9 `useTable+tableFeatures+create*RowModel`, `ColumnDef<typeof features,Row,unknown>[]`, computed ref not `.value`, numeric desc-first unless `sortFn:'basic'`).
**Repo doc gap (both agents agree):** `docs/GUIDES.md` and `docs/ARCHITECTURE.md` do not exist (only `docs/00-11-*.md`); `app/README.md:80,100` links are dead. `AGENTS.md` "pointer capture >8px" has zero implementation (deck uses native scroll+snap by design, locked by `deck.test.ts:50-60`).

## Verdict summary

| # | Component | Design | Issue | Priority |
|---|-----------|--------|-------|----------|
| 1 | BarChart | N (`rounded-t-[4px]`) | overflow crush, 4% zero-floor, tooltip-only a11y, NaN/negative | P1 |
| 2 | LineChart | N (`rounded-[2px]`) | no y-scale, unnamed role=img, ragged labels→undefined, empty-series | P1 |
| 3 | DonutChart | N (`rounded-[2px]`) | unnamed role=img, 4-color cycle collide, dup-key, narrow flex | P1 |
| 4 | HBarChart | Y (reference) | 2% zero sliver, rows not focusable | P3 |
| 5 | FunnelChart | Y | raw value no locale, 3% zero sliver, narrow label col, dup-key | P2 |
| 6 | StackedBarChart | N (2 breaches + dead `hollow`) | undecimated labels, no legend totals, ragged silent fill | P1 |
| 7 | Sparkline | Y | stretch distortion, single-point invisible, blank empty svg | P2 |
| 8 | SocialMediaBar | N (hardcoded white) | button-in-label nesting, monochrome dots, pending→disconnected lie, Manage null | P1 |
| 9 | KpiCard | Y-ish | dead hover arrow (`opacity-0` hover self), `outline-none`, `leading-[1.3]` deviation | P2 |
| 10 | DashboardDetail | Y-ish | dead `close` emit, unknown kpi fallthrough + empty h3, unbounded drill list | P1 |
| 11 | BudgetCards | Y (by delegation) | negative `฿-500` no tone, hardcoded EN labels, pre-load `฿0`/`฿NaN` flash | P2 |
| 12 | StatusFunnel | Y-ish | inline `cursor:pointer`, unconditional `advance` emit, no `role=group`, no disabled | P2 |
| 13 | StatusChip | Y | Start/Design/Dev identical — pipeline lost in read views | P3 |
| 14 | MonthToggle | Y-ish | year dead-end, `w-[34px]` + baked `mb-5`, glyph chevrons | P3 |
| 15 | HashtagGroups | Y-ish | global busy/error blocks all groups, inline `flex:1`, no empty-state, EN strings | P1 |
| 16 | Chip | Y | no tooltip/truncate, empty pill, interactive hacked by callers | P3 |
| 17 | IconButton | Y-ish | `w-9` literal deviation, no focus-visible | P3 |
| 18 | InfoTip | partial | no outside/ESC dismiss, dup `flex-none`, `role=note` weak, `w-[18px]` deviation | P2 |
| 19 | HubTabs | Y-ish | tablist without arrow-key contract, no aria-controls | P2 |
| 20 | GuideHero | N | 2× inline style, hardcoded EN, `/system-map.png` fragile | P1 |
| 21 | FeedGrid | N | `rounded-[10px]`, `bg-black/55 text-white`, `.feed`+grid conflict, EN empty, no list semantics | P1 |
| 22 | ErrorBoundary | Y | sticky fallback across routes, `console.error` prod, retry loop, raw message leak | P2 |
| 23 | DropdownMenu | N | `rounded-[10px]/[8px]`, `<span>` trigger mouse-only, empty-items box, z-scale drift | P1 |
| 24 | DatePickerPopup | partial | `rounded-[8px]`, footer `flex-1` vs `w-auto` conflict, `leading-[1.2]`, DOW hardcoded, no grid nav/trap | P1 |
| 25 | CopyBar | N (1 inline style) | `style=background:var(--wash)`, EN strings, no aria-live | P1 |
| 26 | TxnForm | Y | dup hardcoded ids, no `<form>`, no min/step, stale `dDate`, EN strings | P1 |
| 27 | SettingsPanel | Y-ish | `window.confirm` native, silent scalar revert, `h2` no `leading-*`, `th` no scope | P1 |
| 28 | ProtectedModal | Y | no focus trap/initial/return, global keydown always-on, double-confirm emit, `text-[18px]` | P2 |
| 29 | ProfilePanel | Y-ish | error as `text-muted` no `role=alert`, whitespace-only save, no maxlength | P2 |
| 30 | PlatformLogin | partial | backdrop bypasses `busy`, errors `text-muted`, `w-8` no w-auto, no trap, README#6 chrome dispute | P1 |
| 31 | LoginModal | Y-ish | backdrop/Cancel bypass `busy` (ESC correct), password no `required`, error no `role=alert`, no labelledby | P1 |
| 32 | Drawer | N | `rounded-[12px]`, zero ESC/trap/focus/scroll-lock, untitled no name, hardcoded Close, no dismissable | P0 shared |
| 33 | CommandPalette | N | `rounded-[12px/8px/4px]`, no ESC, no combobox/listbox ARIA, dead `active` class, `outline-none`, EN strings | P1 |
| 34 | ChangePasswordModal | Y-ish | backdrop/Cancel bypass busy, empty-password passes match check, no alert/status roles, no autofocus | P1 |
| 35 | AppShell | Y-ish | `bg-[#ffffffeb]` hardcoded, avatar `w-9` + `title=Owner menu`, `w-full+flex-none` contradiction, notice no live region, dead `.wide`, Settings-leads-deck asymmetry | P2 |
| 36 | ScreensDeck | Y | dead `:month` branch, `startsWith` over-match (`/plans`→`/plan`), passive wheel `preventDefault` risk, index unclamped on role flip, no dots/progress | P2 |
| 37 | Subnavbar | partial | `bg-[rgba(255,255,255,0.92)]`, scoped `<style>` violation, invalid `?tab=` drift, tab ARIA incomplete | P2 |
| 38 | MasterTable | Y | empty no-state, nested checkbox keyboard double-activates select, unguarded `[0]`, selection pruned on sort, loading double-render | P1 |
| 39 | TxnTable | partial | empty bare header, mouse-only sort no aria, dead `cursor-pointer` rows, missing `category` col, `sortFn:basic` vs desc-first rule, NaN amount, header/body misalign | P1 |
| 40 | ContentEditor | Y-ish | `rounded-[8px]`, null→blank, SEO counter/maxlength lie, `window.prompt`, stale `keepMine`, drawer no ESC, history-mode URL | P1 |
| 41 | LiveSection | partial | `rounded-[6px]`, `row flex` hook conflict, nested `<a>` invalid, regex token match, error else-if masking, unsafe `slice`, `fmt(undefined)`, frozen clock, per-render ads join, EN th | P1 |

---

## G1 — Charts (ui/)

### LABEL — BarChart.vue
- **User Journey:** Owner/editor on Home (`/dashboard`) compares categorical magnitudes (pillars/platforms/weekday); secondary Finance income-vs-expense, Performance growth; catalog demo.
- **User Flow:** Props `items:{label,value}[]` → empty → "No data yet." (`BarChart.vue:15`); else `h-[110px]` bars (`:17`) + `title` tooltip (`:19`) + `label (value)` under (`:22`).
- **Use-cases and Function:** Categorical comparison: `pillarBars/platformBars/weekdayBars` (`DashboardPage.vue:379,388,410`), `monthlyIn/monthlyOut` (`FinancePage.vue:69-70`), `growth` (`PerformancePage.vue:109`), catalog (`ComponentsPage.vue:671`).
- **Design:** N — shell ok (`border border-line rounded-none bg-surface`, no `<style>`), breach `rounded-t-[4px]` (`:18`).
- **Edge-cases:** Empty handled; loading covered by parents; >10 items crush (`flex-1` no scroll); 4% floor makes 0==tiny (`:9`); negative→positive stub; NaN→broken layout; tooltip-only, no role/aria, color-only.
- **Location:** `app/src/components/ui/BarChart.vue`
- **Specifics Location:** defect `:18`; scale `:6,9`; usages `DashboardPage.vue:379,388,410`, `FinancePage.vue:69-70`, `PerformancePage.vue:109`, `ComponentsPage.vue:671`; barrel `ui/index.ts:4`.
- **Description and Solution:** Keep as categorical workhorse; sanitize inputs, token radius, scroll guard, readable values (`tabular-nums`).
- **Issue that they found:** (1) radius breach (2) no overflow strategy (3) zero-vs-tiny identical (4) screen-reader invisible.
- **Improvement:** `rounded-none`; `overflow-x-auto` + `min-w-[28px]`/cap; `role=img` + `aria-label`; `safe=v=>Number.isFinite(v)&&v>0?v:0`.

### LABEL — LineChart.vue
- **User Journey:** Home trend read ("Trend · views & likes by day", "Views · last 6 months").
- **User Flow:** `labels+series[]` → empty labels → "No data yet." (`:46`); SVG polyline + area wash (`:50-57`), `<title>` per point, legend totals, x decimated ≤10 (`:39-40`).
- **Use-cases and Function:** Multi-series trend: daily Views+Likes, 6-month Views (`DashboardPage.vue:361-374`), catalog (`ComponentsPage.vue:677`).
- **Design:** N — shell ok, `var(--brand-N)` theme-safe; breach `rounded-[2px]` (`:62`); `text-[10.5px]` exact-form ok.
- **Edge-cases:** Empty series + labels → bare grid; ragged `values>labels` → tooltip "undefined" (`:56`); single point centers; negative below chart; NaN poisons polyline; `height` unvalidated; `role=img` no name (`:48`); no y ticks; 5th series collides (4 brands).
- **Location:** `app/src/components/ui/LineChart.vue`
- **Specifics Location:** `:62,:66,:48,:15,21-26,56`; usages `DashboardPage.vue:361,373`, `ComponentsPage.vue:677`.
- **Description and Solution:** Add y max/0 ticks + `aria-label` totals; guard lengths; clamp y.
- **Issue that they found:** radius, no scale, unnamed img, undefined tooltip, bare-grid edge.
- **Improvement:** dot→`rounded-none`; tick labels; `:aria-label` totals; `labels[i]??''`; guard `!labels.length||!series.length`; `height=max(40,h)`.

### LABEL — DonutChart.vue
- **User Journey:** Home "Share of views · by platform" beside HBar (`DashboardPage.vue:392-401`).
- **User Flow:** `items` → empty/total0 → "No data yet." (`:32`, best guard) → 148px donut (`:34-42`) + center TOTAL + legend value+pct (`:46-53`).
- **Use-cases and Function:** Part-to-whole: platform share (`:399`), catalog (`:674`).
- **Design:** N — shell ok; breach `rounded-[2px]` (`:48`).
- **Edge-cases:** Empty+zero handled; bare `s.label` keys collide; negative→invalid dasharray; `opacity(i)` index fade arbitrary; `role=img` unnamed; 5+ segments reuse 4 brands; `flex` no-wrap squeezes ≤300px.
- **Location:** `app/src/components/ui/DonutChart.vue`
- **Specifics Location:** keys `:37,47`; radius `:48`; math `:11,16`; usage `:399`, catalog `:674`.
- **Description and Solution:** Index keys, sanitize ≥0 finite, name svg, stack on narrow.
- **Issue that they found:** radius, unnamed img, 4-color collision, narrow squeeze, dup-key.
- **Improvement:** `rounded-none`; `aria-label` total+top; `max-sm:flex-col`; 5th+ palette step.

### LABEL — HBarChart.vue
- **User Journey:** Home ranked reads (pillar avg, platform views, campaign spend, Top-5) (`DashboardPage.vue:384,395,430,460`).
- **User Flow:** `items` → empty → "No data yet." (`:21`) → label + track + localized value, row `title`.
- **Use-cases and Function:** Ranked list-with-magnitude; catalog (`:676`).
- **Design:** Y — reference compliant (`rounded-full` allowed, `var(--brand-*)`, exact `text-[..]`, keys `${label}-${i}`).
- **Edge-cases:** Unbounded rows grow page; 2% floor overstates 0 (`:9`); negative→sliver; NaN→`NaN%`; rows not focusable; color-only.
- **Location:** `app/src/components/ui/HBarChart.vue`
- **Specifics Location:** scale `:6,9`; usages `DashboardPage.vue:384,395,430,460`, `ComponentsPage.vue:676`.
- **Description and Solution:** True 0-width for zeros; keyboard parity.
- **Issue that they found:** zero sliver; no keyboard rows.
- **Improvement:** `v<=0?'0%':…`; `tabindex=0` + `aria-label`; `tabular-nums`; optional `maxRows`.

### LABEL — FunnelChart.vue
- **User Journey:** Home "Pipeline · by status" (`DashboardPage.vue:405-406`).
- **User Flow:** `steps` → empty → "No data yet." (`:19`) → label + bar + raw value + % of first (`share()` `:12-14`).
- **Use-cases and Function:** Sequential conversion; catalog (`:675`).
- **Design:** Y — compliant (`rounded-full` only); nit raw `{{s.value}}` no locale (`:26`).
- **Edge-cases:** `first=0` guarded; 3% floor phantom (`:10`); label col `minmax(56,92px)` clips; `%` col 40px ok; no tooltips; out-of-order steps silent; bare `s.label` key.
- **Location:** `app/src/components/ui/FunnelChart.vue`
- **Specifics Location:** key `:21`; share `:7,12-14`; value `:26`; usage `DashboardPage.vue:406`.
- **Description and Solution:** Locale values, zero-floor, wider label col, per-step retention delta.
- **Issue that they found:** no `toLocaleString`, phantom 0-bar, narrow col, no drop-off text, order-dependent >100%.
- **Improvement:** `toLocaleString('en-US')`; index keys; share vs max or sort; `role=img` summary.

### LABEL — StackedBarChart.vue
- **User Journey:** Home "Pace · planned vs posted by week" (`DashboardPage.vue:366-376`).
- **User Flow:** `labels+series[]` → empty → "No data yet." (`:32`) → columns by total/maxTotal (`:20,36`), `hollow` at 0.18 opacity, legend no totals.
- **Use-cases and Function:** Part-of-column-over-time; catalog (`:678`).
- **Design:** N — `rounded-t-[4px]` (`:36`) + `rounded-[2px]` (`:50`); dead `hollow` class (`:37,50`, no definition, effect is inline opacity).
- **Edge-cases:** Ragged `??0` silent; x-labels undecimated crush; legend no totals; sub-pixel slices; tooltip-only; bare `c.label` keys collide; negative→invalid.
- **Location:** `app/src/components/ui/StackedBarChart.vue`
- **Specifics Location:** radii `:36,50`; dead `:37,50`; keys `:35,46,49`; usage `:369`.
- **Description and Solution:** LineChart decimation + totals; remove dead binding; sanitize.
- **Issue that they found:** 2 radius breaches, dead class, crush, hover-only values, silent fill.
- **Improvement:** `rounded-none`; delete `:class={hollow}` (keep opacity + dashed outline); `labelStep`; legend totals; warn on ragged.

### LABEL — Sparkline.vue
- **User Journey:** Ambient trend inside every `KpiCard` (Views/Likes from `daily.*`); catalog Reach demo.
- **User Flow:** `values[]` → polyline 96×26 stretched (`:22-26`); empty → blank svg (guarded by `KpiCard v-if=spark.length` `:23`).
- **Use-cases and Function:** Decorative inline trend; parent owns label+value.
- **Design:** Y — token-clean (`currentColor`, `aria-hidden=true`, no style).
- **Edge-cases:** Single value → invisible one-point polyline (`:13` div-by-zero guarded but renders nothing); all-zero flat bottom; `preserveAspectRatio=none` distorts slope; `NaN`/negative clip/poison; no text alternative.
- **Location:** `app/src/components/ui/Sparkline.vue`
- **Specifics Location:** math `:9-18`; embeds `KpiCard.vue:23,34,44`; usages `DashboardPage.vue:352,354`, `ComponentsPage.vue:679`.
- **Description and Solution:** Keep; fix aspect + single-point + empty contract.
- **Issue that they found:** distortion across widths; blank empty svg standalone; single-point invisible.
- **Improvement:** `xMidYMid meet` or fixed width; svg-level `v-if`; `length===1` → `<circle>`; KpiCard `delta` aria.

### LABEL — SocialMediaBar.vue
- **User Journey:** Non-client Feed (`/content`) platform scope + connect entry (`FeedPage.vue:86` gated `!isClient`).
- **User Flow:** `PLATFORMS` pills → checkbox toggle w/ snap-back (`:23-28`) → dot status → connected ? profile link + Manage→`PlatformLogin` : connect button; trailing Manage (`:58-59`).
- **Use-cases and Function:** Workspace filter + OAuth entry; test `platforms.test.ts:15,24`; catalog (`:688`).
- **Design:** N — pills `rounded-full` ok, buttons `w-auto` ok; breach hardcoded `bg-white/border-white/text-white/accent-white` (`:41,42,44`) unthemable; `bg-success/warning/danger` monochrome (`#0a0a0a`) indistinguishable; `<button>` inside `<label>` (`:40-57`) invalid nesting; `font-[550]/[650]` arbitrary.
- **Edge-cases:** Empty PLATFORMS → dead Manage(`null`); pending misreports disconnected (`statusOf` default); `flex-wrap` overflow best-in-set; checkbox `aria-label` good but dot `title`-only; title `max-sm:hidden` loses context; `activePlatforms[0]??null` → null dialog.
- **Location:** `app/src/components/ui/SocialMediaBar.vue`
- **Specifics Location:** token `:42,44`; nesting `:40-57`; snap `:23-28`; null `:59`; usages `FeedPage.vue:86`, `ComponentsPage.vue:688`.
- **Description and Solution:** Sibling checkbox+button (no nesting); tokenize whites; text status for AT; pending skeleton.
- **Issue that they found:** theming bypass, nested interactives, colliding dots, pending lie, mobile title loss, null Manage.
- **Improvement:** `bg-accent-ink` tokens; split pill; `sr-only` status + `role=group`; guard empty; keep title on mobile.

---

## G2 — Dashboard (ui/)

### LABEL — KpiCard.vue
- **User Journey:** Home overview/ads scan → click tile → drill-down (`openKpi` → `DashboardDetail kind=kpi`).
- **User Flow:** 6 interactive + 3 static (`DashboardPage.vue:347-357`) + 6 static ads (`:421-426`); `to` variant unused.
- **Use-cases and Function:** 3-mode tile (`to` RouterLink / `interactive` div role=button emits `select` / static); `label/value/hint/spark` (`:19-45`, Sparkline `:23,34,44`).
- **Design:** Compliant tokens, `rounded-none`, no style; nits `leading-[1.3]` overrides theme 1.6; hover arrow `opacity-0 hover:opacity-50` on svg itself → unhoverable/dead (`:24,35`); `focus-visible:outline-none` weakens indicator.
- **Edge-cases:** `spark=[]` hidden; `0` renders; `to+interactive` → link wins; Enter/Space ok; static no focus (correct).
- **Location:** `app/src/components/ui/KpiCard.vue`
- **Specifics Location:** branches `:19-45`; arrow `:24-27,35-38`; usages `DashboardPage.vue:347-357,421-426`, `BudgetCards.vue:13-15`, `ComponentsPage.vue:672`.
- **Description and Solution:** Reveal affordance to keyboard/touch; name the button for AT.
- **Issue that they found:** dead hover affordance; no accessible name; weak focus.
- **Improvement:** `group` + `group-hover:opacity-50` (or always `opacity-40`); `aria-label=label`; restore outline/ring.

### LABEL — DashboardDetail.vue
- **User Journey:** Home side-rail `+ New topic` / `Import metrics` / KPI click → inline detail (wide card or bottom sheet).
- **User Flow:** `kind=new` → form → `addTopic()` → `emit done`; `import` → `imp.mutate({platform,month})`; `kpi` → `kpiRows` + cross-link (`/analyze?tab=performance` or `/plan?tab=monthly`); `''` → hint.
- **Use-cases and Function:** 248-line 4-kind component; gates `can(posts.write)/can(metrics.import)`; watchers default date/platforms; i18n `t()`.
- **Design:** Compliant (buttons `w-auto rounded-full`, labels `for/id`, `role=alert/status`, no style); nits `h3 text-2xl leading-[1.2]` overrides 1.6 scale; inline `style=margin-top:0` (`:246`); pill `border` branches always colored in practice.
- **Edge-cases:** Empty guards + date/time fallbacks; unknown kpi → empty h3 + wrong sort branch (`:118,127-141`); `kpiRows` unbounded pushes cross-link off-fold; `toLocaleString` ok.
- **Location:** `app/src/components/ui/DashboardDetail.vue`
- **Specifics Location:** props `:15-16`; watcher `:50-61`; addTopic `:70-101`; kpi `:120-148`; usages `DashboardPage.vue:481,493`, `ComponentsPage.vue:748`.
- **Description and Solution:** Honor or remove `close` contract; cap drill list + count.
- **Issue that they found:** dead `close` emit (parent listens, never fires); unbounded list.
- **Improvement:** emit `close` after add / dismiss btn, or remove type; `ul` in `max-h-* overflow-auto` + `h3 ({{len}})`; unknown kpi → `[]` + fallback title.

### LABEL — BudgetCards.vue
- **User Journey:** Finance side summary beside charts + ledger.
- **User Flow:** `useTxns()` → `income/expense/balance` (`FinancePage.vue:33-35`) → `<BudgetCards currency=฿>` (`:63`) → 3 static KpiCards → `TxnForm` via Log button.
- **Use-cases and Function:** Pure presentational `money(n)=cur+n.toLocaleString` (`:6-8`); `grid3` wrapper.
- **Design:** Compliant by delegation; `grid3 grid` non-conflicting.
- **Edge-cases:** `0`→`฿0`; negative→`฿-500` no danger tone; `NaN`→`฿NaN` pre-load (parent guards table not cards); EN labels unlocalized; `฿` caller-hardcoded.
- **Location:** `app/src/components/ui/BudgetCards.vue`
- **Specifics Location:** `:6-8,:12-16`; usages `FinancePage.vue:16-17,33-35,63`, `ComponentsPage.vue:406,673`.
- **Description and Solution:** Deficit tone + i18n + loading guard.
- **Issue that they found:** deficit indistinguishable; untranslated; `0` flash.
- **Improvement:** `Intl.NumberFormat` + negative `text-danger`/`hint=over budget`; `t()` labels; `isPending` → `—`/skeleton + `Number.isFinite` guard.

### LABEL — StatusFunnel.vue
- **User Journey:** Plan/Content draft Workflow editor (`PlannerPage.vue:385`) + gallery.
- **User Flow:** `order=[Start,Design,Dev,Done]` as Chips, `dark` iff current → click/Enter/Space → `advance(s)` → `draft.status=s`.
- **Use-cases and Function:** Controlled stepper, backward jumps allowed, no internal state.
- **Design:** Delegates to Chip; `role=button tabindex aria-pressed` (`:14`); nit inline `cursor:pointer` (→`cursor-pointer`); relative `./Chip.vue` import vs `@` rule.
- **Edge-cases:** Unknown current → all light; same-stage re-emits (dirty write); always clickable (parent guards save not funnel); 4 isolated buttons no group label (bare `<span>` `:11`).
- **Location:** `app/src/components/ui/StatusFunnel.vue`
- **Specifics Location:** `:7-18`; usages `PlannerPage.vue:384-385`, `ComponentsPage.vue:441,597`.
- **Description and Solution:** Group semantics + disabled mode + regress confirm.
- **Issue that they found:** no group; no readonly; redundant emit.
- **Improvement:** `role=group aria-label=Status`; `if(s!==current) emit`; `disabled` prop (`aria-disabled`, no tabindex, dim).

### LABEL — StatusChip.vue
- **User Journey:** Read projections (MasterTable status col `:120`, gallery).
- **User Flow:** `:status` → `<Chip :dark=Done>` (`:9`).
- **Use-cases and Function:** Single-purpose badge; terminal Done highlighted.
- **Design:** Compliant via Chip.
- **Edge-cases:** Start/Design/Dev identical; unknown → plain label; fixed `Status` type; no tooltip.
- **Location:** `app/src/components/ui/StatusChip.vue`
- **Specifics Location:** `:5-10`; usages `MasterTable.vue:120`, `ComponentsPage.vue:440,594`.
- **Description and Solution:** 3-tone mapping (todo/active/done), keep Done stable.
- **Issue that they found:** WIP indistinguishable in tables.
- **Improvement:** Wash/mid-tone for Design/Dev or `variant` prop; zero call-site change.

### LABEL — MonthToggle.vue
- **User Journey:** Month stepper on Home/Planner/Calendar/Performance.
- **User Flow:** `v-model 1-12` → `shift(±1)` clamp (`:9-13`) → parent refetch; bounds disable; `aria-live=polite` name.
- **Use-cases and Function:** Controlled stepper; `role=group aria-label=Month`, per-btn labels.
- **Design:** Mostly ok (tokens, `rounded-full`); nits fixed `w-[34px]` vs w-rule (justified icon exception), `‹›` glyphs vs SVG, baked `mb-5` (`:17`), `leading-none` on `text-xl` exception.
- **Edge-cases:** No year wrap (by design, undiscoverable); invalid model unclamped; 34px <44px touch; `disabled` short-circuits.
- **Location:** `app/src/components/ui/MonthToggle.vue`
- **Specifics Location:** shift `:9-13`; group `:17-21`; usages Dashboard `:324`, Planner `:292`, Calendar `:169`, Performance `:74`, gallery `:431,697`.
- **Description and Solution:** Boundary tooltips + bigger targets + display clamp.
- **Issue that they found:** Jan/Dec dead-end; small targets.
- **Improvement:** `title` on disabled bounds; `w-10 h-10`; clamp display; optional `wrap` prop (document no-wrap intentional); move `mb-5` to callers.

### LABEL — HashtagGroups.vue
- **User Journey:** Hashtag library (`HashtagsPage.vue:19-27` gated `can(hashtags.write)`).
- **User Flow:** `grid2` cards + `Chip #tag` + draft input + Add → `submit` trims/strips `#`, `busy` guard, clears on resolve, shared `role=alert` error.
- **Use-cases and Function:** Per-group add-tag; `add(groupId,tag)` prop; read-only when `add` undefined (gallery `:690`).
- **Design:** Compliant (cards `rounded-none`, Add `w-auto rounded-full`, `aria-label`, `grid2/row` hooks); nit inline `flex:1` (`:40` →`flex-1`).
- **Edge-cases:** Empty/`#`-only no-op; `##x`→`#x`; no dup/case check; global busy disables all groups; global error misattributed; `[]` no empty-state; `fieldset:disabled` double-gate harmless.
- **Location:** `app/src/components/ui/HashtagGroups.vue`
- **Specifics Location:** `:12-29,:33-48`; usages `HashtagsPage.vue:12-15,25-26`, `ComponentsPage.vue:421,690`.
- **Description and Solution:** Per-group busy/error + inline placement + dup hint + empty slot; contract unchanged.
- **Issue that they found:** global state bleed; no empty-state; no pre-check; inline style; EN strings.
- **Improvement:** `busy/error: Record<groupId,…>`; error under originating card; client dup check (case-insensitive); `v-if=!groups.length` i18n paragraph; `flex-1`; `t()` placeholder/Add.

---

## G3 — Common (ui/)

### LABEL — Chip.vue
- **User Journey/Flow/Use-cases:** Compact pill for platforms/tags/status/setup (`StatusChip:9`, `HashtagGroups:37`, `StatusFunnel:12`, `SettingsPanel:343`, `PlannerPage:390`); `label+dark`.
- **Design:** Y (`rounded-full`, paired borders, `text-[12.5px]` exact, no style); nit `my-[2px] mr-1` bakes spacing.
- **Edge-cases:** `""` → empty pill; long wraps (no truncate); boolean tone only; no tooltip.
- **Location/Specifics:** `ui/Chip.vue:6`; callers above.
- **Description and Solution:** Own interactivity (button variant) or document presentational-only.
- **Issue:** Semantics delegated — callers hack `role=button/tabindex/cursor` (`PlannerPage:390-394`).
- **Improvement:** `clickable/selected/disabled` props; button renders `w-auto` + `aria-pressed` + focus-visible; `:title=label` + `max-w-full truncate`; `v-if=label` or non-empty contract.

### LABEL — IconButton.vue
- **Journey/Flow/Use-cases:** Circular toolbar (`Studio:127`, `Planner:403-448`, `CopyBar:35`); `tone default|primary|danger`, `label`=title+a11y, slot icon, `click` emit.
- **Design:** Mostly Y (`rounded-full`, paired borders, `title+aria-label`); deviation fixed `w-9 h-9` vs w-rule (sanctioned exception + `flex-none` overrides base `button{width:100%}`); no focus-visible.
- **Edge-cases:** Empty slot → label-only circle; `disabled:opacity-40`; call-site SVGs `aria-hidden` correct; empty `label` → unnamed.
- **Location/Specifics:** `ui/IconButton.vue:14-18`.
- **Description and Solution:** Focus indicator + label contract.
- **Issue:** Keyboard focus invisible; literal w-rule conflict.
- **Improvement:** `focus-visible:border-accent` + outline/ring; document `w-9` exemption; require non-empty `label`.

### LABEL — InfoTip.vue
- **Journey/Flow/Use-cases:** `(i)` helper popover (`Studio:120`, `Members:81,97`, `Live:11`); click toggles `open`; `role=note`.
- **Design:** Partial (`rounded-full/rounded-none`, `max-w-[min(280px,72vw)]`); nits dup `flex-none` (`:14`), fixed `w-[18px]`, `text-xs+leading-none` overrides 1.6 (circle exception), weak `role=note`.
- **Edge-cases:** No outside/ESC dismiss; no `aria-controls/expanded→describedby`; `text=""` empty bubble; `right-0` safe.
- **Location/Specifics:** `ui/InfoTip.vue:14-16`; test `infotip.test.ts:4-8`.
- **Description and Solution:** Reuse `useDismiss` (`DropdownMenu:10`, `DatePicker:44-48`) + ESC.
- **Issue:** Sticky popover; weak SR semantics.
- **Improvement:** `useDismiss(root,close)` + ESC; `role=tooltip`, `aria-expanded`, unique id; dedupe class.

### LABEL — HubTabs.vue
- **Journey/Flow/Use-cases:** Hub pill tabs; `Subnavbar(tabs)` → `model-value` → `router.replace({query:{tab}})` (`Subnavbar:16-25,39`); presentational.
- **Design:** Y (`w-auto rounded-full`, paired borders); `my-1 mb-4` overridden by `Subnavbar:44-47`.
- **Edge-cases:** `[]` → empty div (guarded `:38`); wrap grows tall; unknown model → none selected.
- **Location/Specifics:** `ui/HubTabs.vue:14-26`; `layout/Subnavbar.vue:38-39`; `core/screens.ts:30-35`.
- **Description and Solution:** Full tab keyboard or honest group semantics.
- **Issue:** `tablist/tab+aria-selected` without arrow nav, roving tabindex, `aria-controls`.
- **Improvement:** Roving `tabindex 0/-1` + Left/Right/Home/End → `update:modelValue`, or downgrade to `role=group` + `aria-pressed`; overflow-x variant.

### LABEL — GuideHero.vue
- **Journey/Flow/Use-cases:** Onboarding hero in `AppShell:474`; static steps+warnings + `/system-map.png`.
- **Design:** N — inline `text-align:center` (`:17`) + `margin/padding` (`:24`) vs utilities-only; single-col `.steps`; hardcoded EN (no `t()`).
- **Edge-cases:** Missing png → broken img (no lazy/size/onerror); no props; generic alt.
- **Location/Specifics:** `ui/GuideHero.vue:15,17,24`.
- **Description and Solution:** Props + i18n + utilities.
- **Issue:** i18n + utilities violations; fragile asset.
- **Improvement:** `steps/warnings/img` props, `t('guide.*')`, `text-center mt-1.5 pl-[18px]`, `sm:grid-cols-3`, `loading=lazy` + `onerror` hide; verify `app/public/system-map.png`.

### LABEL — FeedGrid.vue
- **Journey/Flow/Use-cases:** Feed mosaic (`FeedPage:114`, `posts/platform/variant`); aspect tiles + badge + `WxH` label.
- **Design:** N — `rounded-[10px]` (`:23`), `bg-black/55 text-white` (`:24`), `.feed` hook + `grid …` conflict (`:22`); type pairing ok.
- **Edge-cases:** Handled unknown platform/variant, empty topic `?`; empty list EN literal (`:30`); tiles `div title` not focusable, no list semantics.
- **Location/Specifics:** `ui/FeedGrid.vue:13-14,22-24,30`; test `feedgrid.test.ts:39,47`.
- **Description and Solution:** Tokenize + un-conflict hook + list semantics.
- **Issue:** Radius + token + shell-conflict + i18n.
- **Improvement:** `rounded-none`; badge → `bg-ink/55 text-accent-ink` (token); inner wrapper owns `grid`, outer bare `feed` (or drop hook); `t('feed.empty')`; `role=list/listitem` + tile `aria-label`.

### LABEL — ErrorBoundary.vue
- **Journey/Flow/Use-cases:** Deck safety net (`App.vue:55-59` wraps ScreensDeck/RouterView); `onErrorCaptured` → fallback retry/reload (`:9-21`, `role=alert`).
- **Design:** Y (`rounded-none border-line`, `w-auto rounded-full`).
- **Edge-cases:** Async rejections not captured (correct, callers handle e.g. `FeedPage:87`); retry same subtree → loop; error persists across routes; raw `message` leaks internals; prod `console.error`.
- **Location/Specifics:** `ui/ErrorBoundary.vue:9-17,25-32`.
- **Description and Solution:** Remount key + route reset + quiet prod.
- **Issue:** Retry loop; stale deck-block.
- **Improvement:** `gen++` + `<slot :key=gen/>`; `watch(route.fullPath→error=null)`; DEV-gate log; generic message + collapsible detail/error ID.

### LABEL — DropdownMenu.vue
- **Journey/Flow/Use-cases:** Click menu (`Studio:131-137` status, gallery); `MenuItem` (`core/menu.ts`); `pick` guards disabled → `select(id)`; `useDismiss` outside+ESC.
- **Design:** Panel `rounded-[10px]` + item `rounded-[8px]` breaches (menu-radius convention vs rule); rest ok.
- **Edge-cases:** `[]` → empty floating box; long no-truncate; `z-[60]` vs InfoTip `200` vs DatePicker `50` drift; `danger` ok.
- **Location/Specifics:** `ui/DropdownMenu.vue:6,10,21-22,27`.
- **Description and Solution:** Button trigger + roles + empty guard.
- **Issue:** `<span @click>` trigger mouse-only (no tabindex/role/`aria-haspopup/expanded`/arrows); empty box.
- **Improvement:** `<button aria-haspopup=menu :aria-expanded>`; `role=menu/menuitem` + arrows; `v-if=open&&items.length`; unify z-scale.

### LABEL — DatePickerPopup.vue
- **Journey/Flow/Use-cases:** Campaign dates (`CampaignsPage:231,235`); trigger → snapshot view → grid → `pick/clear/today` emits `YYYY-MM-DD|null`.
- **Design:** Mixed — trigger `w-full border-0 border-b-ink` fragile; dialog `rounded-none`; day `rounded-[8px]` breach; footer `w-auto/flex-1` redundant; nav `text-[12.5px]+leading-[1.2]` off-scale.
- **Edge-cases:** Handled null/invalid→today, year rollover, null padding, ESC/`@click.self`/cleanup; missing trap/initial focus/day nav/min-max/`aria-selected/today`; DOW `['S',…]` + nav EN while title uses `t()`.
- **Location/Specifics:** `ui/DatePickerPopup.vue:15,21-26,44-48,92,98,100-114`; test `datepicker.test.ts:4-47`.
- **Description and Solution:** i18n + grid keyboard + trap + min/max.
- **Issue:** Partial i18n; no focus management.
- **Improvement:** `monthName`/`t()` DOW/nav; `aria-selected`/`aria-current=date`; roving focus + trap; `min/max` disables; `rounded-none`; resolve footer `flex-1` vs `w-auto`.

### LABEL — CopyBar.vue
- **Journey/Flow/Use-cases:** Planner assembled copy (`PlannerPage:398` compact); `full` block vs `compact` IconButton; `clipboard.writeText` → Copied 1.5s / failed alert.
- **Design:** N — inline `background:var(--wash)` (`:27`) over `bg-surface` (→`bg-wash`); rest ok (`rounded-none`, full `w-full` `:32`).
- **Edge-cases:** Handled denial/insecure-origin alert, timer cleanup, `disabled` when `!text`, full `pre-wrap` vs compact ellipsis; misses EN fallback `(empty — fill…)` + `Copy/Copied` (`:30`); no `aria-live`.
- **Location/Specifics:** `ui/CopyBar.vue:5,10-23,27,30,32,48-50`.
- **Description and Solution:** Utility bg + i18n + live region.
- **Issue:** Theming bypass + untranslated.
- **Improvement:** `bg-wash`; `t(planner.copyAssembled/copyCta/copied/copyEmpty/clipboardFail)`; `aria-live=polite`.

### LABEL — TxnForm.vue
- **Journey/Flow/Use-cases:** Finance Log (`FinancePage:78,87` dual-host detail+Drawer); `add()` → `useAddTxn.mutateAsync` → reset + `done` → `closeLog`.
- **Design:** Y — labels `text-sm font-[650]`, bare inputs (base skin), submit `w-auto rounded-full`, error `role=alert`, perm `title=noPerm`.
- **Edge-cases:** Handled perm + double-submit + `date&&amount>0` + category fallback; gaps EN error/defaults (`:31,39`), `type=number v-model.number=0` no `min/step` (0 invite), future dates, no currency hint, hardcoded `id=txn-*` dup if both hosts mount transiently (`:77-88` mutually exclusive but risky).
- **Location/Specifics:** `ui/TxnForm.vue:12,20-33,39,55-71`.
- **Description and Solution:** i18n + constraints + unique ids.
- **Issue:** Untranslated; weak numeric/date; dup-id risk; no `<form>`.
- **Improvement:** `t('fin.*')`; `<form @submit.prevent>` + `type=submit`; `min=0.01 step=0.01 inputmode=decimal`, `max=today`; `idPrefix` prop + `useId`.

---

## G4 — Overlays (8)

### LABEL — SettingsPanel.vue
- **User Journey:** Owner Settings hub (`/settings workspace`) or avatar sheet; edits year/owner/workspace, 5 option `LISTS` (`:154-160`), users, roles matrix, accounts.
- **User Flow:** `SettingsPage:15` / `AppShell:483` → drafts → `@change` → `saveSetup.mutateAsync` (revert on reject) → nested `Drawer` accounts/perms.
- **Use-cases and Function:** Scalars + lists + team + roles/perms + accounts + `authRequired`; gates `can(setup.write/users.manage)`.
- **Design:** Y utilities/no style, buttons `w-auto/w-full rounded-full`, colored borders; nits `h1/h2 text-xs/sm` no `leading-*` (`:311,335,357,385`); perm `tbl` ok; `th` no `scope`.
- **Edge-cases:** Handled pending, `—` empty, year 1970–9999 revert, last-user/self/role-in-use blocks, `role=alert` errors; unhandled native `window.confirm` (`:59-61,140,227,238`) no trap/i18n; no ESC (parent Drawer also none).
- **Location/Specifics:** `overlays/SettingsPanel.vue:59-61,63-76,311,335,392,441,448`.
- **Description and Solution:** Themed `alertdialog` confirm + surface validation + announce copy.
- **Issue:** Native confirm bypasses DS/a11y; EN hardcoded (`Remove user`, `Role is in use`); silent revert; missing leading/scope.
- **Improvement:** In-app confirm + `t(setup.confirm*)`; `:disabled=saveSetup.isPending` + inline `role=alert`; `leading-[1.6]` on h2; `scope=col`; `aria-live` temp-password.

### LABEL — ProtectedModal.vue
- **Journey/Flow/Use-cases:** Computed-field guard (Dashboard Top-5, Planner Copy-Paste) per README#3; Cancel default; optional 5-min suppress (`sessionStorage`); `open` watcher resets `hide`.
- **Design:** Y (`role=dialog aria-modal labelledby`, `h3 text-[18px]+leading-[1.2]`, `rounded-none`, `w-auto rounded-full`).
- **Edge-cases:** Handled ESC/backdrop/`hide` reset; unhandled trap/initial/return focus; global keydown always-mounted; no `aria-describedby`; suppress no-op silent; double-confirm emits twice.
- **Location/Specifics:** `overlays/ProtectedModal.vue:10-26,30-31`; hosts `Dashboard:496`, `Planner:452`.
- **Description and Solution:** Minimal focus management + scoped ESC.
- **Issue:** Keyboard/AT incomplete; wasteful listener.
- **Improvement:** `nextTick` focus Cancel; Tab trap 440px card; return focus; `aria-describedby` message; ESC only when open; disable after first confirm; tokenize `text-[18px]` or document.

### LABEL — ProfilePanel.vue
- **Journey/Flow/Use-cases:** Self rename (`AppShell:373`, gallery `:758`); `currentUser` watch seeds `name`; `changed` gates Save → `updateProfile.mutate(trim)` → Saved; logged-out hint.
- **Design:** Y (bare input, `w-full rounded-full`, `for/id`); deviation error `text-muted` (`:37-39`) should be `text-danger`.
- **Edge-cases:** Handled logged-out, disabled unless changed+!pending, trim, stale-Saved reset; unhandled whitespace-only sends `""`, no maxlength, error no `role=alert`, success no live.
- **Location/Specifics:** `overlays/ProfilePanel.vue:11-23,28-41`.
- **Description and Solution:** Danger alert + empty guard.
- **Issue:** Error looks like hint + unannounced; empty save relies on backend.
- **Improvement:** `text-danger font-semibold + role=alert`; success `role=status`; `if(!trim) return` + `:disabled=!trim`; `maxlength=80 autocomplete=nickname`.

### LABEL — PlatformLogin.vue
- **Journey/Flow/Use-cases:** Meta/YT/TikTok connect (`AppShell:379`, `SocialMediaBar:60`); mock paste-handle vs OAuth continue; connected sync/reconnect/disconnect; gates `can(platforms.manage)`; tests `platformlogin.test.ts:38-127`.
- **Design:** Disputed — README#6 "never themed, own chrome" vs current themed `bg-surface` pills (`:149-152,181-196`); heading `div` not heading; close `w-8 h-8` no w-rule (`:150`); connected errors `text-muted` (`:176-179`).
- **Edge-cases:** Handled ESC/backdrop/prefill/handle `role=alert`/perm banner/loading/race guard; unhandled backdrop ignores `busy` (ESC/× block) → dismiss-mid-flight (`:146-151,78-80`); no trap/autofocus; `z-[60]` collides guide/pick, below Drawer `1000`.
- **Location/Specifics:** `overlays/PlatformLogin.vue:27,46-80,93-96,146-230`.
- **Description and Solution:** Guard backdrop + promote errors + resolve chrome direction + focus.
- **Issue:** Busy inconsistency; severity mis-style; doc-vs-code; focus.
- **Improvement:** `@click.self=!busy&&close`; errors → `text-danger role=alert`; product call on chrome (or amend README); autofocus handle + trap + `aria-labelledby h2`; × add `w-auto`.

### LABEL — LoginModal.vue
- **Journey/Flow/Use-cases:** Auth entry (`AppShell:361-362,427`); login vs register toggle; clears PII each open (`:23-26`); `submit→login/register→success+close`.
- **Design:** Y (`role=dialog aria-modal`, `h3+leading`, `rounded-none`, `w-auto rounded-full`); toggle link over-styled harmless.
- **Edge-cases:** Handled autofocus, ESC-blocked-when-busy, re-entry guard, autocomplete; unhandled backdrop/Cancel ignore busy (`:63,83` vs ESC `:33-35` correct); password no `required` (`:76-77` vs name/email have); error no `role=alert` (`:78`); no trap/return; dual `ref=firstField` fragile.
- **Location/Specifics:** `overlays/LoginModal.vue:17-35,52-53,63-64,69-79,83`.
- **Description and Solution:** Uniform busy-guard + required + alert + shared modal helper.
- **Issue:** Inconsistent exits; native validation gap; unlabeled dialog (no `aria-labelledby`); password kept across mode toggle.
- **Improvement:** `@click.self=!busy&&close`, Cancel `:disabled=busy`; password `required minlength=8` (check `server/README` policy); error `role=alert`; `aria-labelledby`; clear password on toggle; `useModalEsc` + trap shared with ChangePassword/Protected.

### LABEL — Drawer.vue
- **Journey/Flow/Use-cases:** Shared sheet primitive (Planner/Dashboard/Campaigns/Studio/Ideas/Finance/Settings inner); `v-model` → backdrop + `aside` → ×/backdrop/programmatic; `side/width/contained/inline` + slots; `inline` → `role=region`.
- **Design:** Partial N — `rounded-tr/br/tl-[12px]/[12px]` (`:18,29-30`) vs none/full rule; × `border-0` literal fail + hardcoded `Close`; `<style>` only `@keyframes` allowed-ok; `p-drawer*` hooks correct.
- **Edge-cases:** Unhandled ESC (all Drawer sheets lack it), trap/initial/return, scroll-lock, untitled no name (`Planner:325`, `Studio:174` expose unnamed dialog), `inline region` no label, backdrop unblockable during saves; handled `max-w-[94vw]`, bottom `max-h-[80vh]`, `width` ignored for bottom by design.
- **Location/Specifics:** `overlays/Drawer.vue:2,18-42,45-57`; 7+ hosts listed above.
- **Description and Solution:** Fix once in primitive — highest leverage.
- **Issue:** Missing ESC/focus/labeling affects 7+ flows; radii + EN deviations.
- **Improvement:** ESC handler + trap + initial `.p-drawer-x`/first + return + `aria-labelledby` (or require `aria-label`) + `dismissable` prop + non-contained scroll-lock; `rounded-none`; `t('common.close')`; add `drawer.test.ts` (ESC/backdrop/labelledby/inline).

### LABEL — CommandPalette.vue
- **Journey/Flow/Use-cases:** ⌘K launcher (`Planner:454`, gallery `:797`); substring filter → arrows wrap + Enter/click → `select+close`; `CommandItem{id,label,group?,shortcut?}`.
- **Design:** N radii `rounded-[12px/8px/4px]` (`:63,76,85`); dead `active` class (`:77` no stylesheet); `kbd border-current` ok; input `border-0 border-b` ok; buttons `w-full` ok.
- **Edge-cases:** Handled empty→all, "No results" (`:87`), index reset, wrap `move`, Enter-empty no-op, autofocus; unhandled ESC (backdrop-only `:62` — keyboard opener can't exit); no combobox/listbox ARIA (dialog only, input no `combobox/expanded/controls/activedescendant`, items no `option/selected/id`); hover hijacks index; no async/group semantics.
- **Location/Specifics:** `overlays/CommandPalette.vue:27-58,62-87`.
- **Description and Solution:** ESC + listbox ARIA + radii decision.
- **Issue:** No keyboard dismiss; missing ARIA; dead class; radii.
- **Improvement:** `Escape→open=false` via shared `useModalEsc`; `combobox+listbox/option+aria-selected+id`; `focus:outline` restore or ring; `rounded-none` (or document palette exception); i18n `aria-label` + "No results" + `role=status`.

### LABEL — ChangePasswordModal.vue
- **Journey/Flow/Use-cases:** Self rotation (`AppShell:423`); old/new/confirm → mismatch local → `changePassword` → wipe secrets + `done` → Confirm→Close swap.
- **Design:** Y shell (`z-50`, `h3+leading`, `w-auto rounded-full`); deviation error/success no `alert/status` (`:67-68`).
- **Edge-cases:** Handled ESC-blocked-busy, re-entry guard, mismatch short-circuit, wipe, done-swap; unhandled backdrop/Cancel ignore busy (`:57,70` vs submit `:disabled` correct); no autofocus; no trap/return; no `required/minlength` (correct autocomplete present); `""===""` passes match → server round-trip.
- **Location/Specifics:** `overlays/ChangePasswordModal.vue:9-11,22-32,37-50,57-76`.
- **Description and Solution:** Mirror LoginModal guards + required + roles + shared composable.
- **Issue:** Dismiss-during-flight; empty passes client; unannounced; no focus.
- **Improvement:** Backdrop `!busy` + Cancel `:disabled=busy`; `if(!new) error=t('auth.passwordRequired')` or `required minlength`; autofocus `#cp-old`; `role=alert/status`; extract `useModalEsc/autofocus/trap` for 4 fixed modals.

---

## G5 — Layout (3)

### LABEL — AppShell.vue
- **User Journey:** Authed `/` → first screen (`router:51`) → `App:54-60` shell (navbar/Subnavbar/slot/footer); logged-out → LoginPage; no-workspace → setup; Client → `/content`.
- **User Flow:** Header (logo, navLinks, DatePicker, avatar) → Subnavbar (`:433`) → notice → `main>slot` (`:462-464`) → footer → overlays (pick/guide/settings/auth/Platform).
- **Use-cases and Function:** Compact IA (1 link/hub, Client `/content` `:322-336`); workspace switch/create/rename/delete + global invalidation (`:72-132`); OAuth callback + pick preserving query (`:177-205`); `navDate` jump (`:218-222`); avatar menu (Profile/connect/ws/settings/guide/auth) + doc-click/ESC layers (`:287-305`).
- **Design:** Compliant pills/menus/hooks, no style, PlatformLogin unthemed; violations hardcoded `bg-[#ffffffeb]` (`:346,365`), avatar `w-9 h-9` no w-rule (`:368`), `w-full flex-none` contradiction (`:361-362,414,435,442`), `border-0` nav pills, `title=Owner menu` wrong for non-owners.
- **Edge-cases:** Handled deleted-ws fallback, OAuth rest-preserve, pick re-entry guard, client-steer skips standalone/public, prefix `isActive` (fragile `/planX`); unhandled notice no `aria-live` (6s timeout), dialogs no trap/return, per-press single-layer ESC correct.
- **Location/Specifics:** `layout/AppShell.vue`; mount `App.vue:54-60`; `Subnavbar :433`; defects `:346,354,368,388,434-437`; `wide` dead (`:463`, no CSS/route); README dead link `:80`.
- **Description and Solution:** Tokenize + label + live region + trap + harden match.
- **Issue:** Hex breaks theming; avatar mislabeled; notice silent; no focus mgmt.
- **Improvement:** `bg-surface/90` + blur; avatar `w-auto size-9`, `:title=currentUser?.name??'Account menu'` + aria; notice `role=status aria-live=polite`; trap+return on profile/guide/settings/pick; `isActive: path===to||startsWith(to+'/')`; `w-full→w-auto` on flex/dialog btns; remove/implement `.wide`; comment Settings-leads-deck + fix README link; Vitest Client links==1 + invalidate on switchWs.

### LABEL — ScreensDeck.vue
- **Journey/Flow/Use-cases:** 6-card staff / 1-card Client deck (`screens.ts:27-36` order); drag/wheel/arrows/expand → slide → URL follows; deep-link/back/navDate snap; `?tab=` in-hub; `deckFull` reflow; 404 card.
- **Design:** Animation compliant — real `left/width/height` on `fixed` (`:80-88,126-134`), no scaleX/clip-path, `fullBusy+stop()` (`:56,63,74,146,150`), reduced-motion snap, `will-change`; hooks `deck/spacer/slide/slide-card/card-scroll` preserved; no style; keyboard input-safe (`:342-350`, `deck.test:35-48`).
- **Edge-cases:** Handled programmatic guard (`syncing+userScrolled+beginSync≥900ms+scrollend`), `nearestIndex` skips 0-width + first-clamp, `padOf` live padding, peek-route-then-grow, ESC exits full, wheel throttle 420ms, `basis-[402px]/88vw` + `min-h-[420px]` (short-viewport overflow), `--shell-bars 187/157px`.
- **Location/Specifics:** `layout/ScreensDeck.vue`; mount `App.vue:58`; registry `core/screens.ts`; state `core/deck.ts:5`; tests `deck.test:28-85`; `pageProps :month` dead (`:33-37`, no `:month` route); `tabindex=0 role=region` no dots (`:357`); no pointerCapture/>8px (native scroll by design).
- **Description and Solution:** Docs-vs-code drift is the fix (or implement drag); clamp index; kill dead prop; leak/affordance.
- **Issue:** `>8px capture` required but absent; `index` unclamped on visible shrink; `:month` never matches; `scrollend{once}` leak on timeout; no dots/progress, `region` no roledescription.
- **Improvement:** Correct AGENTS/brief to native-scroll OR implement threshold drag; `watch(visible→clamp+snap)`; delete/wire `:month`; `removeEventListener` in `done()` + clear timer on unmount; `aria-roledescription=carousel` + per-slide `i+1 of N` + dot nav (`w-auto`, `aria-current`); harden `startsWith` to segment; non-passive wheel or drop preventDefault; prefix test `/plans`≠`/plan` + rapid goTo commits last only.

### LABEL — Subnavbar.vue
- **Journey/Flow/Use-cases:** Hub-tab glue (`HUB_TABS subnav.ts:13-43`); `screenOf(path)→tabs→activeTab(?tab?first)→HubTabs→router.replace({query:{...q,tab}})` (`:13-25`); hides on tab-less (`:38`); never hover-preview.
- **Design:** Shell ok (`border-b border-line`); violations `bg-[rgba(255,255,255,0.92)]` (`:38`), scoped `<style :deep>` (`:43-48`) vs no-style rule; HubTabs `w-auto rounded-full`; tab ARIA incomplete.
- **Edge-cases:** Handled empty-hide, same-id no-op, query preserve, vertical pass-through, wrap-vs-scroll; unhandled invalid `?tab=` drift (UI first, URL invalid, no write-back); `|dx|>=|dy|` swallow + `shift+wheel` inconsistency; `overscroll-x:none` claimed not set here.
- **Location/Specifics:** `layout/Subnavbar.vue:13-25,32-34,38,43-48`; `AppShell:433`, `ComponentsPage:767`; `core/subnav.ts:53-63`.
- **Description and Solution:** Token + URL normalize + tab keyboard + CSS ownership.
- **Issue:** Hardcoded bg; drift; tab-only-click; style block.
- **Improvement:** `bg-surface/90`; move `margin:0;justify-center` to HubTabs utilities, delete `<style>`; optional one-shot normalize invalid `?tab=`; `aria-controls` + arrows in HubTabs (or `navigation`+`aria-current`); set `overscroll-behavior-x:none` here.

---

## G6 — Tables (2, TanStack v9)

### LABEL — MasterTable.vue
- **User Journey:** Planner month list → sort Topic/Pillar/Date/Status → check → row → Drawer editor + ProtectedModal.
- **User Flow:** `PlannerPage:305` (when `posts>0`) → row click/Enter/Space → `select` → `pick()` → Drawer; checkbox select-all/indeterminate; `StatusChip`.
- **Use-cases and Function:** Read-only `Post[]` projection, client sort, multi-check, `actions` slot magnifier; no writes (README#84 honored).
- **Design:** Y (`border-faint rounded-none`, exact `text-[11.5px]`, no style, `tbl` hook, `aria-sort/tabindex/aria-label/indeterminate`); v9 pass (`useTable+tableFeatures+createSortedRowModel` `:4-6,14-18,33`; `ColumnDef<typeof features,Post,unknown>[]` `:20`; computed ref whole `:32-33`; `select/platforms enableSorting:false`).
- **Edge-cases:** Empty no-state (caller guards, demo `:659` bare header); sort tested (`tables.test:29-36`, ISO dates ok); no pagination; computed via ProtectedModal; `loading` prop dead (parent never passes); `getHeaderGroups()[0]` unguarded; row `tabindex` + inner checkbox double-activates Space/Enter; `watch(visibleIds)` prunes on refetch (sort preserves); loading renders text+stale rows (no v-else); 3rd click clears sort unmentioned; null date/platforms covered.
- **Location/Specifics:** `tables/MasterTable.vue`; usages `PlannerPage:305`, `ComponentsPage:659`; defects `:86-87,91,109-124,112-115,43-47`.
- **Description and Solution:** Empty row + keyboard guard + header guard + loading/select contract.
- **Issue:** Minor robustness only; M3 keyboard opens editor while checking; M2 bulk dead-end pruned on sort.
- **Improvement:** P1 empty `<tr colspan=6>`; ignore row keydown when target is checkbox (`closest(input[type=checkbox])` / `@keydown.space.stop`); `getHeaderGroups()[0]?.headers??[]`; `v-else` loading + `aria-busy`; emit `update:checked` or remove boxes; `aria-checked=mixed`; date `sortUndefined:1`.

### LABEL — TxnTable.vue
- **Journey/Flow/Use-cases:** Finance ledger (`FinancePage:74` even when empty) → sort Date/Amount/In-Out/Sub → `TxnForm` writes elsewhere; read-only `Txn[]`, `amount={currency}{locale}` (`:22`); no select/pagination/edit.
- **Design:** Mostly Y (`rounded-none tbl` no style) but rows `cursor-pointer hover` with no handler (`:45`) + headers unconditional `cursor-pointer` no focus/keyboard (`:37-38`); v9 pass in form (`:3-6,12-16,18,28-29`); `amount sortFn:basic` (`:21`) correct per test (`tables.test:61-69` desc-first) — TECH notes fixture pre-ordered weakness + unused `alphanumeric`.
- **Edge-cases:** Empty unhandled (bare header, worse than Master); sort mouse-only (no aria/tabindex/keydown vs Master `:91-98`); unbounded ledger; no loading prop (caller `:73` owns); `Number(getValue()).toLocaleString` NaN-hostile; `currency` required no default; OUT unsigned no sign/color; `category` field never rendered (info loss); long `sub` wraps; header Amount left vs body right misalign (`:37 vs :46`).
- **Location/Specifics:** `tables/TxnTable.vue:10,18-26,33,37-38,44-50`; usages `FinancePage:74`, `ComponentsPage:660`.
- **Description and Solution:** Empty row + sort-a11y parity + remove dead pointer + category + amount guard.
- **Issue:** Bare empty ledger; mouse-only sort + misleading pointer; dropped category; T1 sort-rule/test weakness.
- **Improvement:** P1 empty `<td colspan=4>`; port Master `th` (`aria-sort/tabindex/Enter/Space` + helper + focus-visible); drop `cursor-pointer` (or add real `select`+keyboard); add `category` col (or merge `category·sub`); `Number.isFinite` fallback `—` + OUT `−`/`text-danger`; header `amount` right; `[0]?.` guard; `loading?/emptyText?` parity.

---

## G7 — Editor + Live (2)

### LABEL — ContentEditor.vue
- **User Journey:** Studio (`StudioPage:166,176` detail/Drawer) article/page/note authoring → autosave 1.5s / Save Draft → publish/schedule/unpublish → link/revisions/restore/duplicate/archive/delete.
- **User Flow:** `useContentItem+Revisions(id)` → dirty-watch adopt-or-conflict → deep autosave → `save(version)` optimistic concurrency → `ensureSaved` gates publish/schedule/setStatus → conflict bar reload/keepMine; toolbar `wrapSelection/prefixLines/insertLink` + `previewHtml=renderMarkdown`.
- **Use-cases and Function:** Markdown split/edit/preview, metadata slug/kind/excerpt/tags(≤20)/hero/SEO (word/time), lifecycle draft/review/archived, publish menu, revisions drawer, live URL+copy; perms `canEdit=canWrite&&!archived`, no `lockedBy` (version concurrency, correct), no ProtectedModal (SEO spans read-only, correct); queries invalidate `content+public-content` (`queries:583-589,622-672`).
- **Design:** Compliant `@` imports, no mock runtime (type-only `@/mock/db` ok, optionally `@/api/contract`), no style, buttons `w-*`, colored borders, `role=alert/status/dialog/tablist/toolbar`; breach `rounded-[8px]` hero (`:524`); title `text-xl` intentional.
- **Edge-cases:** Covered conflict/dirty/autosave-pause/archived/schedule-guard/clipboard/tag-dedup/empty-revisions; fragile conflict regex (`:168`), `JSON.stringify` dirty order, publish popover no outside/ESC + stays open on error, hero `<img>` no onError, title/slug unvalidated; null-content (`:362`) → blank div (deleted/unknown id); SEO `len/60` vs `maxlength=200` + `len/160` vs `500` lie (`:505,510`); `window.prompt` link (`:339`); `keepMine` stale `serverVersion` (`:198-202`); revisions `aside.fixed` no ESC/trap; `publishedUrl=location.pathname+#/read/` breaks history deploy.
- **Location/Specifics:** `components/editor/ContentEditor.vue:13,23-31,59-91,95-129,132-230,242-300,304-342,386-394,427-555`; usages `StudioPage:13,166,176`, gallery `:410`.
- **Description and Solution:** Null-state > SEO > radius > popover > freshness > drawer > URL; no query/perm change.
- **Issue:** Illegal radius; dead-blank id; SEO limits lie; prompt link; stale overwrite; drawer a11y; URL derivation; regex conflict.
- **Improvement:** `rounded-none`; `v-else` "Content not found"+`/studio` link; `maxlength=60/160` or `len/200+hint`; inline popover input; re-read `itemQ.data.version` in `keepMine`; ESC+focus+`aria-modal`; `Router.resolve(/read/+slug).href`; match 409/code not string; popover dismiss + `aria-expanded`; hero `@error` hide.

### LABEL — LiveSection.vue
- **Journey/Flow/Use-cases:** Read-only Meta/IG mirror (`Live:12` cards, `Feed:115` grid/6, `Performance:125` table, `Dashboard:416`/3, gallery `:768`); `useLive/usePlatforms/useAds` parallel; `metaConnected/needsReconnect/tokenProblem(regex /reconnect/)`; `onSync→mutate(meta)` + `onReconnect→beginOAuth(meta)`; `limit` slice; `boostedBy` ads join; accounts chips + cards/grid/table + reactions/`mmss`/badges/caption/link/date.
- **Design:** Compliant `@`, no mock (strongest), invalidations `platforms+live+metrics` (`queries:482-491`), gates `mayManage`, buttons `w-auto`, colored borders, no style, `h2` no unpaired text, `tbl`; breaches `rounded-[6px]` pills (`:161-162` →`rounded-full`), `row flex…` hook conflict (`:106,115`), nested `<a linkUrl>` in `<a permalink>` invalid (`:155+178`).
- **Edge-cases:** Covered limit=0, missing permalink/thumbnail, zero-hide, `syncedLabel(null)=never`, reaction fallbacks, reconnect guard; fragile `tokenProblem` string-coupled (`:27`), `createdAt.slice` throws (`:180,205`), `fmt()` `en-US` + throws undefined, `Date.now()` frozen "x min ago", `boostedBy` O(posts×ads) per render, error `v-else-if` masks concurrent (`:128-138` live masks sync), hardcoded `Platform/Post/Date` th (`:190`) vs `t()` elsewhere, `:alt=caption` empty, hrefless `<a>` focusable.
- **Location/Specifics:** `components/live/LiveSection.vue:6-50,53-91,106-220`; usages above; barrel `live/index.ts:1`; tests `live.test:7,16,86`.
- **Description and Solution:** Validity > tokens > error stacking > date/code contract > memo > clock/locale; consumers unchanged (`variant/limit` correct); no perm/invalidation bug.
- **Issue:** Invalid HTML; token violations; masked errors; crash-prone dates; frozen time; perf join.
- **Improvement:** Outer `<div/article>` + explicit Open link (as table `:214`) / `span+@click.stop`; `rounded-full`; bare-`row` or drop hook; independent `v-if role=alert/status` errors; `String(??'').slice||'—'` + `fmt(n??0)` + `Number.isFinite`; server `errorCode:TOKEN_EXPIRED` + regex fallback; `computed Map<postId,count>`; `useNow(60s)` + `toLocaleString(undefined)`; i18n 3 th; `:alt` fallback `caption||platformLabel`.

---

## Execution Plans (Extra Agent)

Invariants every phase: `bun run build` + `bun run test` green (`vue-tsc -b` required; `cargo test --manifest-path server/Cargo.toml <filter>`; `cd app && bun run test -- <file>` / `bun run typecheck`).

### Phase 0 — Quick wins (<30m)
GuideHero inline→utilities (`:17,24`); CopyBar `style→bg-wash` (`:27`); StatusFunnel `cursor:pointer→cursor-pointer` (`:14`); HashtagGroups `flex:1→flex-1 min-w-0` (`:40`); DashboardDetail `margin-top:0→mt-0` (`:246`); ErrorBoundary `console.error→DEV-gated` (`:11`); AppShell `bg-[#ffffffeb]→bg-surface/90` (`:346`); Subnavbar `rgba→bg-surface/90` (`:38`).

### Phase 1 — Design-system sweep (radius+color+width)
`rounded-none` for: BarChart `:18`, LineChart `:62`, Donut `:48`, Stacked `:36,50`, FeedGrid `:23`, Dropdown `:22,27`, DatePicker `:107-108`, CommandPalette panel/item/kbd, Drawer sheets, ContentEditor hero `:524`, LiveSection `:161-162` (pills→`rounded-full`); SocialMediaBar `accent-white→accent-accent` + semantic dots (`:42,44`); footer `flex-1` vs `w-auto` resolve; `w-9/w-8/w-[34px]/w-[18px]` document as fixed-control exceptions (keep `flex-none`).

### Phase 2 — A11y
Shared `useOverlay()` first (trap/initial/return/ESC, `role=dialog aria-modal` + labelledby) — Drawer P0; charts `role=img+aria-label+tabindex` (no title-only); TxnTable headers → real buttons + `aria-sort`, drop dead row `cursor-pointer` (or add select+keyboard); MasterTable row-keydown ignores checkbox targets + `[0]?.` guard; HubTabs roving arrows/Home/End or `group+aria-pressed`; Dropdown trigger `<button aria-haspopup/expanded>` + menu roles + ESC; DatePicker/CommandPalette combobox (`combobox/expanded/controls/activedescendant`, `listbox/option/selected`) + ESC + labelled input; KpiCard restore ring (remove `outline-none`).

### Phase 3 — Data robustness
`num()=Number.isFinite?max(0,n):fb` + `max(1,…)` all charts; clamp negatives; normalize ragged; unique `:key=${i}:${label}` (Stacked, HashtagGroups, Settings, Bar); zero→`0` not sliver (Bar 4%, HBar 2%, Funnel 3%); Sparkline single→`<circle>`; TxnTable decide desc-first (drop `basic` on amount or document) + add `category` col + coerce NaN→`—` + right-align header; MasterTable key `checked` by `original.id`, no reset on sort, `indeterminate+aria-checked=mixed`, `v-else` loading + `aria-busy`, date `sortUndefined:1`; Live/Content `String(??'').slice||'—'`, `fmt(n??0)`.

### Phase 4 — i18n (`t()`)
GuideHero `guide.*`; FeedGrid `feed.empty`; TxnForm `txn.*`; CopyBar `common.copy/studio.copied`; BudgetCards `budget.*`; DatePicker DOW→`Intl`/computed `t('date.dow')`; LiveSection `live.platform/post/date`; CommandPalette `common.noResults` + `aria-label`; Subnavbar/MonthToggle/InfoTip labels; tests assert role/text not classes.

### Phase 5 — Overlay/Drawer foundation (before per-overlay edits)
Drawer implements Phase-2 foundation + `side/contained/bottom` ESC + `dismissable` + scroll-lock + `t('common.close')` + `rounded-none`; ProtectedModal consumes + `alertdialog`; Login/ChangePassword/PlatformLogin backdrop `@click.self=busy?noop:close` + `required minlength=8` + `aria-labelledby` + error `role=alert` (+success `status`); ProfilePanel danger alert + trim-reject whitespace + maxlength; SettingsPanel `window.confirm→Drawer/ProtectedModal`; CommandPalette/DatePicker wire foundation; add `drawer.test.ts` (ESC/backdrop/labelledby/inline).

### Phase 6 — Per-component logic (blast-radius order)
Chip `as=button` + StatusFunnel guard `s!==current` + `disabled`; DashboardDetail wire/remove `close` + unknown-kpi `null+v-if` + scroll list + count; KpiCard remove/gate dead arrow; MonthToggle drop `mb-5` to callers; HashtagGroups per-group `busyBy/errorBy`; SettingsPanel confirm flow; PlatformLogin keep unthemed chrome; ErrorBoundary `gen++` remount + route-watch reset; ContentEditor `keepMine` re-read rev, prompt→popover, null `v-else`, SEO single-source, 409/code match, hero onError, popover dismiss; LiveSection de-nest link, memo `boostedBy Map`, `useNow(30-60s)`, stack errors, safe dates; AppShell `w-full→w-auto` flex/dialog, drop/implement `.wide`, `role=status+aria-live` notice; ScreensDeck delete `:month`, `===||startsWith('/')`, non-passive wheel or drop prevent, clamp visible, dots/progress + `aria-roledescription=carousel`; Subnavbar `?tab=` allowlist sync, delete `<style>`, verify `overscroll-x:none`; SocialMediaBar `label→div` + independent checkbox; InfoTip `note→status/tooltip+aria-expanded`.

### Suggested batches
A (P0+P1 tokens): `typecheck` + tests. B (Foundation): Drawer/`useOverlay`/ProtectedModal/Chip + manual Tab/ESC. C (Tables+Charts): keyboard/sort/NaN/empty tests (`-- MasterTable -- TxnTable -- Chart -- Sparkline`). D (Overlays): auth/demo logins (`DEMO_MODE=1`) intact. E (i18n+logic): final `bun run build && bun run test`.
