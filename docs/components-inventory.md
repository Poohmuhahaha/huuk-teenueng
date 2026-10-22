# Huuk — Component & Module Inventory (ครบ)

> **56 `.vue` + 22 `.ts` (non-test)** — แบ่งเป็น Shared components · Screen(pages) · App · Core/API/Mock

## สรุป
| ประเภท | จำนวน |
|---|---|
| Shared components (`components/`) | 30 |
| Screen components (`pages/` + `App.vue`) | 26 |
| Core / API / Mock modules (`.ts`) | 22 |
| **รวม `.vue`** | **56** |

## 1) Shared components — 30
| Component | group | หน้าที่ | Props | Emits | ใช้ที่ |
|---|---|---|---|---|---|
| `BarChart` | ui | กราฟแท่ง | items:{label,value}[] | — | Home, Analyze |
| `BudgetCards` | ui | การ์ดสรุป Income/Expense/Balance | income·expense·balance·currency | — | Analyze›Finance |
| `CalendarGrid` | ui | ปฏิทิน WEEK 1–6 | year·month·posts·weekStart·show* | — | Plan›Calendar |
| `CardBase` | ui | การ์ดมีเดียพื้นฐาน (ยังไม่ใช้) | title·meta·coverLabel | — | — |
| `CarouselTabs` | ui | แท็บเลื่อนแนวนอน | items[]·v-model | update:modelValue | Home, Plan, Calendar |
| `Chip` | ui | ป้าย/tag (toggle) | label·dark? | — | หลายหน้า |
| `CopyBar` | ui | คัดลอกข้อความ | text | — | Plan›Monthly |
| `DatePickerPopup` | ui | ปุ่ม+ปฏิทินเลือกวันที่ | v-model·disabled·id·title | update:modelValue | Home(topbar), Promote |
| `ErrorBoundary` | ui | ดัก error ลูก | — | — | App |
| `FeedGrid` | ui | พรีวิวฟีด 3×3 | posts·platform·variant? | — | Content›Feed |
| `GuideHero` | ui | เนื้อหา Guide | — | — | AppShell |
| `HashtagGroups` | ui | กลุ่มแฮชแท็ก+add | groups·add? | — | Plan›Hashtags |
| `HubTabs` | ui | แถบแท็บ hub | tabs[{id,label}]·v-model | update:modelValue | ทุก hub |
| `InfoTip` | ui | ปุ่ม Info | text | — | Ads/Brand/Campaigns/Live/Members/Studio/Register |
| `KpiCard` | ui | การ์ด KPI | label·value | — | Home |
| `SocialMediaBar` | ui | เลือก/เชื่อมแพลตฟอร์ม | — | — | Content›Feed, Settings›Connections |
| `StatusChip` | ui | ป้ายสถานะ | status | — | MasterTable |
| `StatusFunnel` | ui | Start→Design→Dev→Done | current | advance(Status) | Plan›Monthly |
| `AppShell` | layout | shell หลัก (navbar/avatar menu/overlays) | — | — | App |
| `ScreensDeck` | layout | deck สไลด์หน้าจอ | — | — | App |
| `ChangePasswordModal` | overlays | modal เปลี่ยนรหัสผ่าน | open | close | AppShell |
| `LoginModal` | overlays | modal login/register | open·initialMode? | close·success | AppShell |
| `PlatformLogin` | overlays | full-screen connect/manage | platform | close | AppShell, SocialMediaBar |
| `ProfilePanel` | overlays | แผงโปรไฟล์ใน avatar menu | — | — | AppShell |
| `ProtectedModal` | overlays | เตือนแก้ computed cell | open·field | cancel·confirm | Home, Plan›Monthly |
| `SettingsPanel` | overlays | workspace/permissions/accounts | — | — | AppShell, Settings |
| `MasterTable` | tables | ตารางหลัก (search/sort/select) | rows·loading? | select(Post) | Plan›Monthly |
| `TxnTable` | tables | ตารางรายรับรายจ่าย | rows·currency | — | Analyze›Finance |
| `ContentEditor` | editor | เอดิเตอร์ CMS | id | — | Content›Studio |
| `LiveSection` | live | section live mirror | variant?·limit? | — | Home, Content›Feed, Live, Analyze |

## 2) Screen components (pages) — 26
| Component | kind | route / ที่อยู่ | หน้าที่ |
|---|---|---|---|
| `App.vue.vue` | root | `app/App.vue` | boot gate: restore → login gate → workspace gate → shell |
| `DashboardPage.vue` | screen | `/dashboard (Home)` | Home hub — KPI · Live · campaigns · charts · top5 |
| `PlanPage.vue` | hub | `/plan` | Plan hub with tabs |
| `ContentPage.vue` | hub | `/content` | Content hub with tabs |
| `PromotePage.vue` | hub | `/promote` | Promote hub with tabs |
| `AnalyzePage.vue` | hub | `/analyze` | Analyze hub with tabs |
| `SettingsPage.vue` | hub | `/settings` | Settings hub with tabs |
| `PlannerPage.vue` | screen | `/plan › Monthly` | Monthly planner — master table + row editor (embed) |
| `CalendarPage.vue` | screen | `/plan › Calendar` | Smart Calendar |
| `IdeasPage.vue` | screen | `/plan › Ideas` | Idea bank |
| `HashtagsPage.vue` | screen | `/plan › Hashtags` | Hashtag library |
| `StudioPage.vue` | screen | `/content › Studio` | CMS list + editor (embed) |
| `FeedPage.vue` | screen | `/content › Feed` | Feed review |
| `CampaignsPage.vue` | screen | `/promote › Campaigns` | Campaign manager (embed) |
| `AdsPage.vue` | screen | `/promote › Meta Ads` | Meta Ads mirror + manage |
| `PerformancePage.vue` | screen | `/analyze › Performance` | Metrics import + goals + per-post |
| `FinancePage.vue` | screen | `/analyze › Finance` | Income/expense ledger |
| `BrandPage.vue` | screen | `/settings › Brand` | Brand identity / CI |
| `MembersPage.vue` | screen | `/settings › Members` | Workspace members (owner-only) |
| `LivePage.vue` | screen | `(orphan — /live→/dashboard)` | เต็มหน้า Live (ซ้ำกับ Home LiveSection) |
| `LoginPage.vue` | public | `/login` | Login gate |
| `RegisterPage.vue` | public | `/register` | Signup |
| `PlansPage.vue` | public | `/plans` | Package chooser |
| `WorkspaceSetupPage.vue` | public | `/welcome` | First-run workspace creation |
| `ReadPage.vue` | public | `/read · /read/:slug` | Public reader |
| `NotFoundPage.vue` | public | `*` | 404 |

## 3) Core / API / Mock modules — 22
| Module | หน้าที่ |
|---|---|
| `core/auth.ts` | session auth (US-012) — email+password accounts |
| `core/queries.ts` | TanStack Query hooks ทั้งแอป |
| `core/session.ts` | session state (auth/http/query cache) |
| `core/token.ts` | token storage (isolated) |
| `core/workspace.ts` | active workspace storage |
| `core/screens.ts` | screen registry (6 hubs) |
| `core/i18n.ts` | copy dictionary (English-only) |
| `core/theme.ts` | brand-driven theming (CI) |
| `core/platforms.ts` | platform metadata + active platforms |
| `core/oauth.ts` | OAuth hand-off (state + redirect) |
| `core/images.ts` | brand image rules (slots/bounds) |
| `core/markdown.ts` | markdown renderer (escape-first) |
| `core/deeplink.ts` | hash deep-link fixups |
| `core/navdate.ts` | global navbar date |
| `core/protected.ts` | computed-cell guard suppression |
| `app/router/index.ts` | routes จาก hub registry + legacy redirects |
| `app/main.ts` | boot (clear cache between sessions) |
| `api/index.ts` | data-source switch (mock/http) |
| `api/contract.ts` | data-source contract |
| `api/http.ts` | HTTP client → Rust backend |
| `mock/api.ts` | mock async API |
| `mock/db.ts` | domain model (workbook) |

## หมายเหตุ
- `LivePage.vue` ตอนนี้ **ไม่มี route** (ยุบเป็น section ใน Home) — เป็น orphan ควรลบหรือเก็บเป็น reference
- `CardBase.vue` ยังไม่ถูกใช้
- hub pages (`PlanPage/ContentPage/…`) เก็บ child screen ไว้เป็น tabs (embedded props/emit)
