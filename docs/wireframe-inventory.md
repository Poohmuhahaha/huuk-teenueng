# Huuk — Page & UI Inventory (for wireframes)

> สารบัญนี้ลิสต์ **ทุกหน้า + ทุก overlay** ว่า must-have อะไรบ้าง: ส่วนประกอบ, element, ปุ่มทั้งหมด, ฟิลด์, และ Sheet / Popup / Modal / Drawer
> ที่มา: อ่านโค้ดจริง `app/src/pages/*`, `components/{ui,overlays,layout,tables,editor,live}`, copy จาก `core/i18n.ts`

**สัญลักษณ์:** `[Modal]` ซ้อนกลางจอ · `[Sheet/Dialog]` แผงใหญ่ · `[Popup]` เมนูลอย · `[Drawer]` แผงด้านข้าง · `[Full]` เต็มจอ
**ทุกหน้าที่เป็น write:** ถ้าไม่มี permission ปุ่มจะถูก disable + tooltip `auth.noPerm` (FE) และ server ตอบ 403 (BE)

---

## 0) Global chrome — AppShell (มีทุกหน้า ยกเว้น login/register/plans/read/welcome)

**Sections:** top bar · OAuth notice banner · main deck · footer

**Top bar elements**
- Logo `Huuk by teenueng` (→ `/`)
- Nav (staff): 4 dropdown groups — **Plan** (Brand, Monthly, Calendar, Ideas, Hashtags) · **Create** (Feed, Content) · **Promote** (Live, Campaigns, Ads) · **Analyze** (Stats→Dashboard/Performance, Finance)
- Nav (Client role): single link **Content**
- **Members** link (owner-only, appears/disappears)
- Logged-out: **Sign up** (primary) + **Log in** buttons
- Logged-in: **Date picker popup** (nav date → jumps planner month, hidden for Client) · **Avatar button** (letter)

**Avatar menu `[Popup]` (role=dialog)**
- ProfilePanel (see §Overlays): name field + **Save**
- Social Media block: title + per-platform row (status dot · platform name · **connect account / Manage** button)
- Workspace switcher `[Popup]` (role=menu): list rows (name · connected/total · ✓ active) · **Rename this workspace** · **Delete this workspace** · new-workspace input + **Create**
- Actions: **Workspace settings** · **Guide** · **Change password** · **Sign out everywhere** · **Log out**

**OAuth notice banner:** message + **Close** · **OAuth multi-Page picker** `[Modal]`: title, hint, list of pages (handle · Connect this Page), errors

**Overlays hosted here:** Guide `[Modal]` (GuideHero) · Workspace settings `[Modal]` (SettingsPanel) · LoginModal · ChangePasswordModal · PlatformLogin `[Full]`

**ScreensDeck (main):** horizontal slide deck; each slide = one screen with **Go full page / Exit full page** button; unknown route → NotFoundPage. Footer: `Huuk by Teeneung v0.1(Beta)`

**States:** boot loader · route not found

---

## 1) `/login` — LoginPage
**Purpose:** เข้าสู่ระบบ
**Sections:** centered card (brand: logo + workspace name) · form
**Fields:** Email · Password
**Buttons:** **Log in** (submit) · **No account? Register** (→ /register, ถ้า allowRegistration)
**States:** error (401/429) · busy

## 2) `/register` — RegisterPage
**Fields:** Name · Email · Password (min 12, InfoTip “At least 12 characters”)
**Buttons:** **Create account** · **Already have an account? Log in**
**States:** registration closed message · error (409/400)

## 3) `/plans` — PlansPage
**Sections:** header (brand + Back to app / Log in) · title + subtitle · 3 plan cards · footer (teenueng.com · huuk.teenueng.com)
**Plan card elements:** badge “Most popular” · name · tagline · price · feature list · CTA · “Current package”
**Buttons:** **Continue with Free** (free) · **Contact sales** (link) · **Back to workspace** / **Log in**

## 4) `/welcome` — WorkspaceSetupPage (onboarding)
**Sections:** centered card · “Signed in as {name}”
**Fields:** Workspace name (placeholder “e.g. Teenueng”)
**Buttons:** **Create workspace**
**Text:** onboarding title/body/hint

---

## 5) `/brand` — BrandPage  (Brand Identity)
**Sections:** Identity fields · Logos · Palette · Typography · Style · Moodboard
**Identity fields:** Channel name · Positioning · Slogan (bio) · Audience · Voice + tone · **Do** panel (list + input “add rule” + **Add**) · **Don't** panel (list + input + **Add**)
**Logos (3 slots):** Main / Secondary / Social — each: preview + **Upload image** (file) · URL field “or paste an https image URL” · remove **×** · bounds hint · **Resize this image** (ถ้าใหญ่เกิน)
**Palette:** live preview strip (Button/Link/Chart bars) · 4 color tiles (color picker + role label + ★ primary + sample) · hex field · **Copy**
**Typography:** font preview (“Aa Bb 123 · สวัสดี”) · imported fonts (pick + **Remove font**) · suggested fonts · **Import font** (file) · hint
**Style:** Corner radius (range+number) · Accent fill (range+number, %) · Stroke (range+number) · Shadows segmented (**None/Soft/Strong**) · live sample (Button/Link/Chart bars) · **Export**: **Copy CSS** · **Download JSON** · **Upload JSON**
**Moodboard (≤12):** image grid (remove **×**, **Resize this image**) · **Add images** (file) · URL field + **Add**
**Overlays:** none (inline) — confirm none
**States:** loading · error + **Try again** · read-only (no `brand.write`) · oversized-image notice · import invalid

---

## 6) `/planner/:month` — PlannerPage (Monthly Planner)
**Sections:** month tabs (CarouselTabs) · planner bar · master table · row editor (selected) · (empty state)
**Planner bar:** count `N topics · M today` · **+ New topic**
**MasterTable** (see §Components): search, sortable headers, select checkbox, status chip, selection count
**Row editor elements (selected row):**
- Heading: topic + “Locked by {name}”
- Fields: Topic · Pillar (select) · Format (select) · Goal (select) · Date · Time · Status (select) · Hook · Caption · CTA · Hashtag group
- **StatusFunnel** (Start→Design→Dev→Done) interactive
- **Platforms** chips (toggle)
- **CopyBar** (assembled copy-paste text)
**Buttons (row editor):** **Save row** · **Duplicate** · **Delete** (danger) · **Lock/Unlock** · **Request edit of computed field**
**Overlays:** `[Modal]` ProtectedModal (computed-field guard) · row editor is inline card
**States:** loading rows · error + **Try again** · empty + **+ New topic** · lockedByOther (save/dup/del disabled) · error alerts

## 7) `/calendar` — CalendarPage (Smart Calendar)
**Sections:** title · month carousel tabs · week-start toggle · filters · display toggles · CalendarGrid
**Buttons:** **Sunday** / **Monday** (week start) · current month label
**Filters (selects):** Pillar · Platform · Format · Status
**Display toggles (checkboxes):** pillar · platform · status
**Element:** CalendarGrid (WEEK 1–6)

## 8) `/ideas` — IdeasPage (Idea Bank)
**Sections:** ideas table · Capture form
**Table columns:** Topic · Format · Idea · Link · Done (checkbox) · action
**Row button:** **Promote**
**Capture fields:** Topic · Format · Promote to month (select) · Idea · Link
**Button:** **Add idea**
**States:** loading · error (toggle/promote) · no permission

## 9) `/hashtags` — HashtagsPage (Hashtag Library)
**Element:** HashtagGroups (groups + add)
**Wrapper:** disabled fieldset เมื่อไม่มี `hashtags.write`
**Buttons:** **Try again** (error) · add-group/add-tag (inside HashtagGroups)
**States:** loading · error

## 10) `/feed` — FeedPage (Feed Review)
**Sections:** title · SocialMediaBar · platform tabs · feed controls · canvas meta · FeedGrid · LiveSection
**Platform tabs:** per platform + count
**Controls:** “Show feed up to” (date) · Size (select variants)
**Canvas meta line:** `W × H px · ratio · platform · up to month · N planned posts`
**Elements:** FeedGrid (3×3 preview), LiveSection (grid)
**Buttons:** **Try again** (error)

## 11) `/studio/:id?` — StudioPage (Content Studio / CMS)
**Layout:** 2 columns — left list, right editor (or “pick” card)
**Left (list):** title + **+ New** · Search field · status filter tabs (All/Draft/In review/Scheduled/Published/Archived) · list items (title · status chip · excerpt · author · updated · words · tags)
**Right — ContentEditor (`ContentEditor.vue`):**
- Header: Title field · status chip · save-state (Saving…/Conflict/Unsaved/Saved)
- Actions: **Save** · **Publish ▾** (menu: **Publish now**, schedule field, **Schedule**, **Unpublish**) · **Submit for review** / **Back to draft** · **Duplicate** · **History (n)** · **Archive** / **Restore to draft** · **Delete** · view toggle **Write / Split / Preview**
- Conflict banner: **Reload latest** · **Keep mine**
- Live share card: URL + **Copy link** · **Open**
- Fields: URL slug · Type (Article/Page/Note) · Summary · Body (Markdown) + toolbar **B/I/H2/•/1./”/`</>`/🔗** + word count · Tags · Hero image URL · SEO (summary: **SEO title** counter, **SEO description** counter)
- Preview pane: rendered article
- Revisions `[Drawer]`: list (#n · note · author · time) + preview + **Restore this version** + **Close**
**Overlays:** publish `[Popup menu]` · revisions `[Drawer]` · confirm delete/restore (native confirm) · sign-in card (ถ้ายังไม่ล็อกอิน)

## 12) `/live` — LivePage
**Sections:** title (+ InfoTip) · LiveSection (cards)
**LiveSection elements:** header (title · “Synced {time}” · **Sync now**) · connected account chips (Meta/IG stats) · posts grid **or** table · empty/reconnect states
**Buttons:** **Sync now** · **Reconnect Meta** · **Open post** (per item) · **View live content** (from PlatformLogin)
**States:** loading · stale/error · empty (connected/disconnected)

## 13) `/campaigns` — CampaignsPage
**Layout:** left list · right editor
**Top bar:** **New campaign** · “Saved.” · error
**Left list items:** name · status badge · start→end · N linked · budget
**Editor:** Heading (name) + **Save** · **Delete**
**Fields:** Campaign name · Status (select) · Objective · Start date (DatePickerPopup) · End date (DatePickerPopup) · Platforms chips · Pillars chips · Budget (฿) · Goal metric (select) · Goal target · Hashtags (comma) · Owner · Notes (textarea)
**Schedule table:** Content · Date · Status
**Link content:** checkbox list of content items
**Overlays:** DatePickerPopup `[Popup]`

## 14) `/ads` — AdsPage (Meta Ads)
**Sections:** header · opt-in row · account picker + summary tiles · campaign table · expanded detail · audit trail
**Header:** title (+ InfoTip) · “Synced {time}” · **Sync ads**
**Opt-in row:** badge (Management on / Read-only) · **Allow campaign management** / **Turn off management** · InfoTip
**Account picker:** Ad account select (multi) or chip · **Boost**
**Summary tiles:** Spend · Results · CTR · CPC
**Campaign table columns:** Campaign · Objective · Status · Budget · Spend · Results · CTR · CPC · action (**Pause/Resume**)
**Expanded detail:** Settings (dl) · Actions (**Edit budget**, **Duplicate**, **Boost**) · **Ad sets** (name, status, optimization, targeting, promoted, ads list) · empty notes
**Dialogs `[Modal]`:** Edit budget (daily budget + **Save**/**Cancel**) · opt-in confirm (**Confirm**/**Cancel**) · Boost a post (Campaign name · Objective · Daily budget · Days · Countries · Post + **Create paused campaign**/**Cancel**)
**Audit:** “Recent actions” list
**States:** loading · empty (no campaigns / no accounts) · reconnect (**Reconnect Meta**) · read-only (management off) · errors

## 15) `/dashboard` — DashboardPage
**Sections:** title · month tabs · KPI grid (6) · LiveSection · Active campaigns glance · Planned vs posted by pillar · By platform · By status · Top 5 by views
**Elements:** 6 KpiCards (Total, Posted, Pending, WIP, Views, Likes) · BarChart · Chip lists · tables · ProtectedModal
**Buttons:** **Open Ads** · **Edit** (Top-5 row) · ProtectedModal Confirm/Cancel
**Overlays:** `[Modal]` ProtectedModal (Top-5 computed cell)

## 16) `/performance` — PerformancePage
**Sections:** title · Import metrics card · Platform goals table · Views by month chart · Per-post +7 days table · LiveSection(table)
**Import card fields:** Platform (select) · Month (select) · **Import**
**Tables:** Platform goals (Platform/Start/Goal/Now/Delta) · Per-post (Topic/Posted/+7 days/Likes/Views)
**Buttons:** **Import** · **Try again** (error)

## 17) `/finance` — FinancePage
**Sections:** title · BudgetCards (income/expense/balance) · Income by month chart · Expense by month chart · Transactions table · Log transaction form
**Form fields:** Date · Amount · Kind (IN/OUT) · Sub-category
**Buttons:** **Add transaction** (loading = common.loading)
**Element:** TxnTable
**States:** loading ledger · error

## 18) `/members` — MembersPage
**Sections:** title (+ InfoTip) · add-member form · members list
**Add form:** email field (“teammate@example.com”) · **Add** · InfoTip
**List row:** name · email · role chip · Owner badge · You chip · **Remove**
**States:** loading · error · not owner (“Only the workspace owner can manage members.”) · empty · added/removed notices

## 19) `/read` and `/read/:slug` — ReadPage (public)
**Header:** brand “Huuk · Studio” · **All posts** · **Back to app**
**Index:** title “Published” · list (title · excerpt · author · date)
**Article:** meta (author · date) · title · excerpt · hero image · markdown body · tag chips
**States:** loading · empty · not found (title/hint + **All posts**)

## 20) `*` — NotFoundPage
**Elements:** title · message
**Buttons:** **Back to the first screen** · **Open the planner**

---

## Overlays / Sheets / Popups / Modals (รวม)

| # | ชื่อ | ชนิด | เปิดจาก | องค์ประกอบ + ปุ่ม |
|---|---|---|---|---|
| O1 | **SettingsPanel** | `[Sheet/Dialog]` | avatar menu · “Workspace settings” | Year · Owner · Workspace name fields · Option lists (Pillars/Formats/Goals/Statuses/Platforms) + input **Add** · Show editable-cell colors checkbox · Users (list + new user name/role + **Add** + remove ×) · Accounts (list + **Remove account** ×) · Create sign-in account (Name/Email/Role/Initial password + **Create account** + temp password + **Copy**) · Auth & permissions (Require login checkbox · Roles matrix table + **Add role** + **Save permissions**) |
| O2 | **ProfilePanel** | `[Popup]` (ใน avatar menu) | avatar menu | Your profile: name field + **Save** · “Saved” |
| O3 | **LoginModal** | `[Modal]` | top bar Log in/Sign up | title (Log in / Create account) · Name (register) · Email · Password · switch link · **Cancel** · **Log in/Create account** |
| O4 | **ChangePasswordModal** | `[Modal]` | avatar menu | Current password · New password · Confirm new password · **Cancel** · **Confirm** (→ **Close**) |
| O5 | **ProtectedModal** | `[Modal]` | planner/dashboard computed cells | title “Protected cell” · message · “Do not show again for 5 minutes” checkbox · **Cancel** · **Confirm** |
| O6 | **PlatformLogin** | `[Full]` | avatar menu · Live/Ads reconnect | connected view: account/externalId/token/expires/last sync/media tracked panels · Scopes chips · **Open profile** · **Reconnect** · **View live content** · **Sync now** · **Disconnect** · close **×**. Not-connected: profile URL/@handle + **Connect**. Real OAuth: **Continue to {platform}** |
| O7 | **GuideHero** | `[Modal]` | avatar menu “Guide” | guide content + **Close** |
| O8 | **OAuth multi-Page picker** | `[Modal]` | OAuth callback (หลาย Page) | title · hint · page list (**Connect this Page**) · **Close** |
| O9 | **DatePickerPopup** | `[Popup]` | top bar · campaign dates | date grid + **Select date / Clear / Today** |
| O10 | **Ads dialogs** | `[Modal]` ×3 | AdsPage | Edit budget · opt-in confirm · Boost a post (see §14) |
| O11 | **Publish menu** | `[Popup]` | ContentEditor | Publish now · Schedule for (datetime) · Schedule · Unpublish |
| O12 | **Revisions drawer** | `[Drawer]` | ContentEditor | version list + preview + **Restore this version** |

## Shared components (ใช้ซ้ำหลายหน้า)
- **MasterTable** — search field · sortable headers · select-all/row checkboxes · StatusChip · “N selected” hint
- **LiveSection** — see §12 (variants: cards / grid / table)
- **CarouselTabs** — month tabs (Planner/Dashboard/Calendar/…)
- **StatusFunnel · Chip · StatusChip · KpiCard · BarChart · BudgetCards · TxnTable · CalendarGrid · FeedGrid · HashtagGroups · CopyBar · InfoTip · GuideHero · DatePickerPopup · SocialMediaBar · ErrorBoundary**
- **ContentEditor** — see §11

## State ที่ต้องมีทุกหน้า (สำหรับ wireframe)
`loading` · `empty` · `error (+Try again)` · `no-permission (disable/hide + tooltip)` · `reconnect-needed` (หน้าที่ผูก platform)
