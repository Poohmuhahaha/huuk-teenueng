# Huuk — โครงสร้างหน้าใหม่ (compact IA) เพื่อ UX ที่ดีที่สุด

> เป้าหมาย: ยุบページให้เหลือ **6 หน้าหลัก + tabs** (จากเดิม 14 การ์ด), ทำให้ทุก user story (101) มีที่อยู่ชัดเจน, ลดการสลับหน้า, และจัดกลุ่มตามงาน (job) ของผู้ใช้

## หลักคิด (UX rationale)

- **จัดกลุ่มตามงาน ไม่ใช่ตามฟีเจอร์**: ผู้ใช้คิดเป็น "วางแผน → ทำคอนเทนต์ → โปรโมท → วัดผล → ตั้งค่า"
- **ลดหน้า แต่เพิ่ม tab**: ฟีเจอร์ที่ต้องดูคู่กัน (Monthly+Calendar, Campaigns+Ads, Performance+Finance, Brand+Members+Connections) อยู่ในหน้าเดียว
- **Live ไม่ต้องเป็นหน้า**: ย้ายเป็น *section* ใน Home + จัดการ connection ที่ Settings › Connections
- **Hashtags ไม่ต้องเป็นหน้า**: เป็น tab ใน Plan และมี quick access บน navbar
- **ทุก write ยังถูกกั้นด้วย permission เดิม** (FE can() + BE require_perm) และ states เดิม (loading/empty/error/no-perm/reconnect)

## โครงสร้างใหม่ (target)

```
Navbar (staff):  Home · Plan · Content · Promote · Analyze · Settings

Home            ── Dashboard KPIs + Live mirror + Active campaigns + Onboarding hint
Plan            ── tabs: Monthly | Calendar | Ideas | Hashtags
Content         ── Studio (list + editor) ── Publish/Schedule/Revisions
Promote         ── tabs: Campaigns | Ads
Analyze         ── tabs: Performance | Finance
Settings        ── tabs: Brand | Workspace | Members | Connections | Permissions

Public/Onboarding (no shell):  Welcome · Login · Register · Plans · Read/:slug · 404
Overlays (ไม่นับเป็นหน้า):  Account menu · Settings sheet · Guide · Login modal ·
                            Change password · Platform connect (full) · Protected modal ·
                            Date picker · Ads dialogs · Publish menu · Revisions drawer
```

## Mapping: Page → Tab → User stories

| หน้า | Tab / section | Epics | # stories | story ids |
|---|---|---|---|---|
| **Welcome (public)** | onboarding | E00 | 1 | US-00-01 |
| **Account** | Sign in / Sign up + account menu | E01 | 11 | US-01-01, US-01-02, US-01-03, US-01-04, US-01-05, US-01-06, US-01-07, US-01-08, US-01-09, US-01-10, US-01-11 |
| **Settings** | Workspace & Members | E02 | 8 | US-02-01, US-02-02, US-02-03, US-02-04, US-02-05, US-02-06, US-02-07, US-02-08 |
| **Settings** | Brand | E03 | 10 | US-03-01, US-03-02, US-03-03, US-03-04, US-03-05, US-03-06, US-03-07, US-03-08, US-03-09, US-03-10 |
| **Settings** | Connections | E08 | 8 | US-08-01, US-08-02, US-08-03, US-08-04, US-08-05, US-08-06, US-08-07, US-08-08 |
| **Settings** | Workspace & Permissions | E11 | 8 | US-11-01, US-11-02, US-11-03, US-11-04, US-11-05, US-11-06, US-11-07, US-11-08 |
| **Plan** | Monthly | E04 | 10 | US-04-01, US-04-02, US-04-03, US-04-04, US-04-05, US-04-06, US-04-07, US-04-08, US-04-09, US-04-10 |
| **Plan** | Ideas & Hashtags | E05 | 6 | US-05-01, US-05-02, US-05-03, US-05-04, US-05-05, US-05-06 |
| **Analyze** | Performance & Finance | E06 | 6 | US-06-01, US-06-02, US-06-03, US-06-04, US-06-05, US-06-06 |
| **Promote** | Campaigns | E07 | 7 | US-07-01, US-07-02, US-07-03, US-07-04, US-07-05, US-07-06, US-07-07 |
| **Promote** | Ads | E09 | 8 | US-09-01, US-09-02, US-09-03, US-09-04, US-09-05, US-09-06, US-09-07, US-09-08 |
| **Content** | Studio | E10 | 13 | US-10-01, US-10-02, US-10-03, US-10-04, US-10-05, US-10-06, US-10-07, US-10-08, US-10-09, US-10-10, US-10-11, US-10-12, US-10-13 |
| **Read (public)** | Delivery | E12 | 5 | US-12-01, US-12-02, US-12-03, US-12-04, US-12-05 |
| **Home** | Live section | E08 | 8 | US-08-01, US-08-02, US-08-03, US-08-04, US-08-05, US-08-06, US-08-07, US-08-08 |

**รวม 109 mapping** จาก 101 stories (มี E08 ที่โผล่ทั้ง Home และ Settings → ซ้ำ 0 รายการ)

## โครงคร่าวต่อหน้า (wireframe skeleton)

- **Home** — KPI cards (6) · **Live section** (last N posts + Sync) · **Active campaigns** card (link ไป Promote) · onboarding/guide hint · Date picker
- **Plan** — Tab bar (Monthly/Calendar/Ideas/Hashtags) · **Monthly**: master table + row editor (fields+StatusFunnel+CopyBar+Lock) · **Calendar**: month grid + filters · **Ideas**: table + Capture · **Hashtags**: groups
- **Content** — Left list (search + status tabs + items) · **Editor**: title/status/save-state · actions (Save/Publish▾/Duplicate/History/Archive/Delete) · fields + markdown toolbar + SEO · preview · **Feed preview** section
- **Promote** — Tab bar (Campaigns/Ads) · **Campaigns**: list + form + schedule + link content · **Ads**: opt-in + account + KPI tiles + campaign table + detail + audit
- **Analyze** — Tab bar (Performance/Finance) · **Performance**: import metrics + goals table + views chart + per-post table · **Finance**: budget cards + charts + txn table + log form
- **Settings** — Tab bar (Brand/Workspace/Members/Connections/Permissions) · Brand kit · Workspace+options · Members list · Connections (platform rows + connect/manage) · Roles matrix + accounts

## เดิม → ใหม่ (14 → 6)

| เดิม (การ์ด) | ใหม่ |
|---|---|
| Brand | Settings › Brand |
| Planner (Monthly) | Plan › Monthly (+ Calendar รวมเป็น tab) |
| Calendar | Plan › Calendar |
| Ideas | Plan › Ideas |
| Hashtags | Plan › Hashtags (quick access บน navbar) |
| Feed | Content › Feed preview (section) |
| Studio (CMS) | Content › Studio |
| Dashboard | Home |
| Performance | Analyze › Performance |
| Finance | Analyze › Finance |
| Live | Home › Live section + Settings › Connections |
| Campaigns | Promote › Campaigns |
| Ads | Promote › Ads |
| Members | Settings › Members |

## Public / Onboarding (แยก ไม่มี shell)

| หน้า | บทบาท | stories |
|---|---|---|
| Welcome | สร้าง workspace แรก | + แนะนำ onboarding |
| Login / Register | เข้าสู่ระบบ/สมัคร | E01 |
| Plans | เลือกแพ็กเกจ | E01 (plan) |
| Read /:slug | หน้าอ่าน public | E12 |
| 404 | ทางตัน → planner | – |
