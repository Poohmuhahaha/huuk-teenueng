# E09 — Meta Ads Mirror & Management

**โมดูล:** `ads.rs`, `handlers.rs` (get_ads/sync_ads/set_ads_manage/set_ad_campaign_status/budget/duplicate/create_ad_boost)
**Storage:** `Workspace.ads: AdsData` (accounts/campaigns/adsets/ads/insights/audit/error), `Workspace.ads_manage`
**FE:** `pages/AdsPage.vue` (+ `DashboardPage`/`LiveSection` แสดงผล)
**Permission:** `platforms.manage` (ทั้งหมด) + **gate** `ads_manage`

---

## แนวคิด
- Mirror ข้อมูลโฆษณา Meta (read) + จัดการได้ (write) เมื่อ owner เปิด opt-in `ads_manage=true`
- ทุก mutation ต้องผ่าน `manage_context`: campaign ต้องอยู่ใน mirror + `ads_manage` ต้อง true
- background `spawn_ads_loop` (ADS_REFRESH_MIN, default 60) refresh เฉพาะ meta connected
- มี audit trail (`AdsAudit`) ทุก action

## User Stories

| ID | As a | I want to | So that | Priority |
|---|---|---|---|---|
| US-09-01 | Owner | ดูภาพรวมบัญชีโฆษณา/แคมเปญ/โฆษณา | เห็นสถานะโฆษณา | Must |
| US-09-02 | Owner | Sync ข้อมูลโฆษณา | ดึงล่าสุด | Must |
| US-09-03 | Owner | เปิด/ปิดสิทธิ์จัดการแคมเปญ | ควบคุมความเสี่ยง | Must |
| US-09-04 | Owner | เปิด/ปิด/archive แคมเปญ | ควบคุมการยิง | Should |
| US-09-05 | Owner | ปรับงบ (daily/lifetime) | คุมค่าใช้จ่าย | Should |
| US-09-06 | Owner | duplicate แคมเปญ | ต่อยอดเร็ว | Could |
| US-09-07 | Owner | boost โพสต์เป็นแคมเปญ | เพิ่มreach | Could |
| US-09-08 | Owner | ดู audit trail | ตรวจสอบย้อนหลัง | Should |

---

## US-09-01 — View mirror

`GET /api/ads` (read) → `AdsView` = `AdsData` + `canManage`
- แสดง empty/error/reconnect state; account picker; summary tiles; campaign→adset→ad tree; insights

## US-09-02 — Sync

`POST /api/ads/sync` · `platforms.manage` → ต้อง Meta connected → `refresh_workspace_ads`
- snapshot oauth+token ใต้ lock → เรียก Graph **นอก lock** → เขียนกลับใต้ lock (เก็บ audit ไว้)
- error → เก็บข้อมูลเดิมไว้ + set `workspace.ads.error`; handler map เป็น 400
- FE `useSyncAds` → invalidate `['ads']`

## US-09-03 — Toggle management opt-in

`POST /api/ads/manage {enabled}` · `platforms.manage`
- ตั้ง `workspace.ads_manage`
- FE: `canWrite = mayManage && view.canManage` — ปุ่มจัดการทั้งหมดถูกล็อกจนกว่าจะเปิด

## US-09-04 — Campaign status

`POST /api/ads/campaigns/{id}/status {status: ACTIVE|PAUSED|ARCHIVED}` · `platforms.manage`
**Main flow**
1. `manage_context(id)` → campaign ต้องมีใน mirror (400) + `ads_manage` true (400)
2. actor = email หรือ `"operator"`
3. Graph POST เปลี่ยนสถานะ → อัปเดต mirror → append `AdsAudit`
4. FE invalidate `['ads']`

## US-09-05 — Budget

`POST /api/ads/campaigns/{id}/budget {dailyBudget?, lifetimeBudget?}` · `platforms.manage`
- ผ่าน gate เดียวกัน; ปฏิเสธเฉพาะค่าที่เป็น 0 (ไม่มี upper bound) → 400
- Graph POST → mirror + audit

## US-09-06 — Duplicate

`POST /api/ads/campaigns/{id}/duplicate` · `platforms.manage` → Graph `/{id}/copies` → คืน `{ campaignId }`
- FE invalidate `['ads']`

## US-09-07 — Boost post → campaign

`POST /api/ads/boost {name, objective, dailyBudget, days, countries[], storyId}` · `platforms.manage`
- page_id มาจาก Meta connection `external_id` (ว่าง → 400)
- สร้าง campaign → adset → ad ตามลำดับ (3 ขั้น)
- `days` ถูก clamp 1–90 เงียบ ๆ; `special_ad_categories` hardcode `[]`

## US-09-08 — Audit

ทุก mutation สำเร็จ append `AdsAudit { at, actor, action, target, detail }` (ใหม่สุดก่อน, cap)

---

## Gate diagram — Ads mutation

```mermaid
flowchart TD
  Req["Ads mutation"] --> Perm["require_perm(platforms.manage)"]
  Perm --> Ctx["manage_context(campaignId)"]
  Ctx --> Known{"campaign อยู่ใน mirror?"}
  Known -->|no| E400a["400 unknown campaign"]
  Known -->|yes| Gate{"workspace.adsManage == true?"}
  Gate -->|no| E400b["400 เปิด campaign management ก่อน"]
  Gate -->|yes| Acct["accounts.first() (ข้อจำกัด)"]
  Acct --> Graph["Graph POST (token ใน query)"]
  Graph --> Mirror["อัปเดต mirror + AdsAudit"]
  Mirror --> Inv["FE invalidate ['ads']"]
```

## Sequence — Manage campaign status

```mermaid
sequenceDiagram
  autonumber
  actor O as Owner
  participant AP as AdsPage
  participant API as handlers.rs
  participant A as ads.rs
  participant ST as Store
  participant G as Meta Graph

  O->>AP: เปิด Ads → เปิด opt-in (ถ้ายังไม่เปิด)
  AP->>API: POST /api/ads/manage {enabled:true}
  API->>ST: ads_manage = true
  API-->>AP: { canManage: true }

  O->>AP: กด Pause แคมเปญ
  AP->>API: POST /api/ads/campaigns/:id/status {status:PAUSED}
  API->>API: require_perm("platforms.manage")
  API->>A: manage_context(id)
  A->>ST: campaign มีใน mirror? ads_manage true?
  alt ไม่ผ่าน gate
    A-->>API: error
    API-->>AP: 400
  else ผ่าน
    A->>G: Graph POST /:id {status:PAUSED}
    alt Graph error
      G-->>A: error
      A-->>API: error
      API-->>AP: 400
    else สำเร็จ
      G-->>A: ok
      A->>ST: อัปเดต mirror + append AdsAudit
      API-->>AP: 200 { ok:true }
      AP->>AP: invalidate ['ads']
    end
  end
```

## Flow — Ads lifecycle (view → sync → manage → boost)

```mermaid
flowchart TD
  Open["/ads"] --> Get["GET /api/ads → AdsView { ..., canManage }"]
  Get --> Conn{"Meta connected?"}
  Conn -->|no| Reconnect["แสดงปุ่ม reconnect (E08)"]
  Conn -->|yes| Show["แสดง accounts/campaigns/ads/insights/audit"]
  Show --> Sync["POST /api/ads/sync (platforms.manage)"]
  Sync --> SGate{"Meta connected?"}
  SGate -->|no| S400["400"]
  SGate -->|yes| SFetch["refresh_workspace_ads (Graph นอก lock)"]
  SFetch --> SWrite["mirror + audit; error → เก็บของเดิม + ads.error"]
  Show --> Manage["เปิด opt-in: POST /api/ads/manage {enabled}"]
  Manage --> Act{"action"}
  Act -->|"status"| St["POST /campaigns/:id/status"]
  Act -->|"budget"| Bu["POST /campaigns/:id/budget"]
  Act -->|"duplicate"| Du["POST /campaigns/:id/duplicate"]
  Act -->|"boost"| Bo["POST /ads/boost"]
  St --> Chk{"manage_context ผ่าน?"}
  Bu --> Chk
  Du --> Chk
  Bo --> Chk
  Chk -->|no| E400["400"]
  Chk -->|yes| Do["Graph call + mirror + audit + invalidate ['ads']"]
```
