# Huuk — Pages & Elements by User Story

> อ้างจาก user story ทั้ง **101** รายการ + โครง compact IA ที่แก้ในโค้ดจริง (`core/screens.ts`, `app/router`, hub pages) แล้ว

## สรุปจำนวน
| ประเภท | จำนวน |
|---|---|
| **Pages (แอป + shell)** | **6** (Home · Plan · Content · Promote · Analyze · Settings) |
| **Pages (public / onboarding, ไม่มี shell)** | **6** (Welcome · Login · Register · Plans · Read · 404) |
| **รวม Pages ทั้งหมด** | **12** |
| Tabs (ภายใน hub) | 14 |
| Overlays (Sheet/Popup/Modal/Drawer — ไม่นับเป็นหน้า) | 12 |

## แผนที่หน้า → story
| # | Page (route) | Tabs | stories | # |
|---|---|---|---|---|
| 1 | Home `/dashboard` | – | US-06-01, US-06-04, US-08-07 | 3 |
| 2 | Plan `/plan` | Monthly / Calendar / Ideas / Hashtags | US-04-01..10, US-05-01..06 | 16 |
| 3 | Content `/content` | Studio / Feed preview | US-10-01..13 (+feed = US-04-10) | 13 |
| 4 | Promote `/promote` | Campaigns / Meta Ads | US-07-01..07, US-09-01..08 | 15 |
| 5 | Analyze `/analyze` | Performance / Finance | US-06-02, US-06-03, US-06-05, US-06-06 | 4 |
| 6 | Settings `/settings` | Brand / Workspace / Members / Connections | US-02-01..08, US-03-01..10, US-08-01..06/08, US-11-01..07, US-01-09..11 | 35 |
| 7 | Welcome `/welcome` | – | US-00-01, US-02-01 | 2 |
| 8 | Login `/login` | – | US-01-02 | 1 |
| 9 | Register `/register` | – | US-01-01 | 1 |
| 10 | Plans `/plans` | – | US-01-08 | 1 |
| 11 | Read `/read`, `/read/:slug` | – | US-12-01..05 | 5 |
| 12 | 404 | – | US-12-05 | 1 |

## รายละเอียดหน้า + Element ที่ต้องมี

### 1) Home — `/dashboard`
**Stories:** US-06-01 · US-06-04 · US-08-07

**Sections / elements**
- Month tabs (CarouselTabs)
- KPI cards ×6 — Total · Posted · Pending · WIP · Views · Likes
- **Live section** — header + `Synced {time}` + connected-account chips (Meta/IG stats)
- Live posts — card grid (thumbnail · kind · caption · likes/comments/shares/views) หรือ empty/reconnect state
- **Active campaigns** — alert list + table (Campaign · Status · Spend) + Open Ads
- Planned vs posted chart (BarChart) · By platform chips · By status chips
- Top 5 by views table

**Buttons**
- Sync now (Live) · Open Ads · Edit (Top-5 row)
- KPI/table sort ไม่มีปุ่ม; ProtectedModal ปุ่ม Cancel/Confirm

**Overlays**
- ProtectedModal (แก้ computed cell)

**States**
- loading · error (+Try again) · empty (ยังไม่มีโพสต์/ยังไม่เชื่อม) · reconnect-needed

### 2) Plan — `/plan`
**Tabs:** Monthly · Calendar · Ideas · Hashtags
**Stories:** US-04-01..10 · US-05-01..06

**Monthly (Planner) — element**
- Month tabs + `N topics · M today`
- Master table: search · sortable headers · select checkbox · Status chip · selected count
- Row editor: Topic · Pillar · Format · Goal · Date · Time · Status · Hook · Caption · CTA · Hashtag group
- StatusFunnel (Start→Design→Dev→Done) · Platform chips · CopyBar (copy-paste)
- Lock chip “Locked by {name}” + note

**Monthly — buttons**
- + New topic
- Save row
- Duplicate
- Delete
- Lock / Unlock
- Request edit of computed field
- Try again

**Calendar — element / buttons**
- Month tabs · week-start toggle (**Sunday**/**Monday**) · current month
- Filters (selects): Pillar · Platform · Format · Status
- Display toggles: pillar · platform · status
- CalendarGrid (WEEK 1–6)

**Ideas — element / buttons**
- Table: Topic · Format · Idea · Link · Done(checkbox) · Promote
- Capture form: Topic · Format · Promote to month(select) · Idea · Link · **Add idea**

**Hashtags — element / buttons**
- HashtagGroups (groups + add tag/group) · Try again

**Overlays / States**
- ProtectedModal · loading/empty/error · no-permission disable · locked-by-other

### 3) Content — `/content`
**Tabs:** Studio · Feed preview
**Stories:** US-10-01..13 (feed = US-04-10)

**Studio — list**
- Search field · status tabs (All · Draft · In review · Scheduled · Published · Archived)
- Item: title · status chip · excerpt · author · updated · words · tags · **+ New**

**Studio — editor**
- Title field · status chip · save-state (Saving… · Conflict · Unsaved · Saved)
- Fields: URL slug · Type (Article/Page/Note) · Summary · Body(Markdown)+toolbar (B/I/H2/•/1./”/`</>`/🔗) · word count · Tags · Hero URL
- SEO (details): SEO title (+counter) · SEO description (+counter)
- Preview pane · Share card (URL + Copy link · Open) · Conflict banner

**Studio — buttons**
- Save · Publish ▾ (Publish now · schedule datetime · Schedule · Unpublish) · Submit for review / Back to draft
- Duplicate · History (n) · Archive / Restore to draft · Delete · view toggle (Write · Split · Preview)
- Conflict: Reload latest · Keep mine · Copy link · Try again

**Feed preview**
- SocialMediaBar · platform tabs(+count) · Show feed up to(date) · Size(select variants)
- Canvas meta line · FeedGrid 3×3

**Overlays / States**
- Publish menu `[Popup]` · Revisions drawer `[Drawer]` (list + preview + Restore this version + Close) · sign-in card · loading/empty/error

### 4) Promote — `/promote`
**Tabs:** Campaigns · Meta Ads
**Stories:** US-07-01..07 · US-09-01..08

**Campaigns — element**
- List: name · status badge · start→end · N linked · budget
- Editor: Campaign name · Status · Objective · Start date · End date · Platforms chips · Pillars chips · Budget(฿) · Goal metric · Goal target · Hashtags · Owner · Notes
- Schedule table (Content · Date · Status) · Link content (checkbox list)

**Campaigns — buttons**
- New campaign
- Save
- Delete

**Meta Ads — element**
- Header + Synced time · opt-in badge (Management on / Read-only)
- Account picker (select/chip) · KPI tiles (Spend · Results · CTR · CPC)
- Campaign table (Campaign · Objective · Status · Budget · Spend · Results · CTR · CPC · action)
- Expanded: Settings dl · Actions · Ad sets (name/status/optimization/targeting/promoted/ads)
- Audit trail (Recent actions)

**Meta Ads — buttons**
- Sync ads · Allow campaign management / Turn off management · Boost
- Pause / Resume · Edit budget · Duplicate · Reconnect Meta

**Overlays / States**
- DatePickerPopup · Ads dialogs `[Modal]` ×3 (Edit budget · opt-in confirm · Boost a post) · loading/empty/reconnect/read-only

### 5) Analyze — `/analyze`
**Tabs:** Performance · Finance
**Stories:** US-06-02 · US-06-03 · US-06-05 · US-06-06

**Performance**
- Import metrics card: Platform(select) · Month(select) · result text
- Platform goals table (Platform · Start · Goal · Now · Delta)
- Views by month chart (BarChart) · Per-post +7 days table (Topic · Posted · +7d · Likes · Views)
- LiveSection (table)

**Performance — buttons**
- Import · Try again

**Finance**
- BudgetCards (Income · Expense · Balance)
- Income by month chart · Expense by month chart · Transactions table (TxnTable)
- Log transaction: Date · Amount · Kind (IN/OUT) · Sub-category

**Finance — buttons**
- Add transaction

**States**
- loading ledger · error · no-permission

### 6) Settings — `/settings`
**Tabs:** Brand · Workspace · Members · Connections
**Stories:** US-02-01..08 · US-03-01..10 · US-08-01..06/08 · US-11-01..07 · US-01-09..11

**Brand tab**
- Identity: Channel name · Positioning · Slogan · Audience · Voice · Do panel(input+Add) · Don't panel(input+Add)
- Logos ×3 (Main/Secondary/Social): preview · **Upload image** · URL field · remove × · bounds hint · Resize this image
- Palette: live strip · 4 tiles(color picker+role+★+sample) · hex field · Copy
- Typography: preview · imported fonts(pick · Remove font) · suggested · Import font
- Style: Corner radius · Accent fill · Stroke (range+number) · Shadows (None/Soft/Strong) · live sample
- Export: Copy CSS · Download JSON · Upload JSON
- Moodboard (≤12): grid(remove × · Resize) · Add images · URL + Add

**Workspace tab (config + permissions + accounts)**
- Year · Owner · Workspace name
- Option lists ×5 (Pillars/Formats/Goals/Statuses/Platforms): chips + input + Add
- Show editable-cell colors (checkbox)
- Users: rows(name · role · remove ×) + new user name/role + Add
- Accounts: rows(name · email · N sessions · remove ×)
- Create sign-in account: Name · Email · Role · Initial password + Create account → temp password + Copy
- Auth: Require login (checkbox) · Roles matrix table + Add role + Save permissions

**Members tab**
- Add by email + **Add** · member rows (name · email · role chip · Owner badge · You chip · Remove) · owner-only notice

**Connections tab**
- Per-platform rows (status dot · name · connect/manage) — SocialMediaBar; opens PlatformLogin

**Overlays / States**
- PlatformLogin `[Full]` (account/token/expires/last sync/media · Scopes · Open profile · Reconnect · Sync now · Disconnect) · read-only hints · loading/error

### Public / Onboarding (ไม่มี shell)

| # | Page | route | stories | elements |
|---|---|---|---|---|
| 7) Welcome | `/welcome` | US-00-01 · US-02-01 | โลโก้ · onboarding title/body · “Signed in as {name}” · Workspace name field · **Create workspace** · hint |
| 8) Login | `/login` | US-01-02 | brand(logo+workspace) · Email · Password · **Log in** · “No account? Register” · error(401/429) |
| 9) Register | `/register` | US-01-01 | Name · Email · Password(min 12) · **Create account** · “Already have an account? Log in” · closed notice |
| 10) Plans | `/plans` | US-01-08 | 3 plan cards(badge · name · tagline · price · features · CTA · Current) · **Continue with Free** · **Contact sales** · Back to app / Log in |
| 11) Read | `/read, /read/:slug` | US-12-01..05 | header(brand · All posts · Back to app) · index list · article(meta · title · excerpt · hero · markdown body · tag chips) · not-found |
| 12) 404 | `*` | US-12-05 | title · message · **Back to the first screen** · **Open the planner** |

## Overlays (Sheet / Popup / Modal / Drawer — ไม่นับเป็นหน้า)

| Overlay | ชนิด | element / ปุ่ม | stories |
|---|---|---|---|
| **Account menu (avatar)** | Popup | ProfilePanel(name+Save) · Social connect rows · Workspace switcher(list+Create/Rename/Delete) · Workspace settings · Guide · Change password · Sign out everywhere · Log out | US-01-03..07, US-02-02..05 |
| **Settings sheet** | Dialog | เหมือน Settings › Workspace (config/options/permissions/accounts) | US-11-01..07, US-01-09..11 |
| **LoginModal** | Modal | mode login/register · Name · Email · Password · switch · Cancel · submit | US-01-01/02 |
| **ChangePasswordModal** | Modal | Current · New · Confirm · Cancel · Confirm/Close | US-01-06 |
| **ProtectedModal** | Modal | title · message · “ไม่ต้องแสดง 5 นาที” · Cancel · Confirm | US-04-03 |
| **PlatformLogin** | Full | account/token/expires/last sync/media · Scopes · Open profile · Reconnect · Sync now · Disconnect · profile URL + Connect | US-08-02..06/08 |
| **OAuth page picker** | Modal | title · hint · page list(Connect this Page) · Close | US-08-03 |
| **Guide** | Modal | guide content · Close | US-00-01 |
| **DatePickerPopup** | Popup | date grid · Select date · Clear · Today | US-04-03, US-07-02 |
| **Ads dialogs ×3** | Modal | Edit budget · opt-in confirm · Boost a post | US-09-03/05/07 |
| **Publish menu** | Popup | Publish now · schedule datetime · Schedule · Unpublish | US-10-06/07/08 |
| **Revisions drawer** | Drawer | version list · preview · Restore this version · Close | US-10-11/12 |

## State ที่ทุกหน้าต้องมี (wireframe)
`loading` · `empty` · `error (+Try again)` · `no-permission (disable/hide + tooltip)` · `reconnect-needed` (หน้าที่ผูก platform)
