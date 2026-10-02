# E11 — Settings, Roles & Permissions

**โมดูล:** `handlers.rs` (get_setup/patch_setup/add_option/add_user/remove_user/add_role/remove_role + require_perm/has_read_access)
**Storage:** `Store.setup: SetupConfig`, `Store.accounts`, `Store.permissions` via roles
**FE:** `components/overlays/SettingsPanel.vue`, `core/queries.ts#usePermission`
**Permission:** `setup.write`, `users.manage`

---

## โมเดล SetupConfig
`language, year, owner, workspaceName, pillars[], formats[], goals[], statuses[], platforms[], showEditableColors, users[], authRequired, allowRegistration, roles[]`

## User Stories

| ID | As a | I want to | So that | Priority |
|---|---|---|---|---|
| US-11-01 | Owner | ดู/แก้ค่าตั้งต้น (ปี, owner, workspaceName) | ตั้งระบบ | Must |
| US-11-02 | Owner | เพิ่มตัวเลือก (pillar/format/goal/status/platform) | ป้อน dropdown | Must |
| US-11-03 | Owner | เพิ่ม/ลบ user ใน directory + กำหนด role | จัดทีม | Must |
| US-11-04 | Owner | สร้าง/แก้ role + ติ๊ก permission | คุมสิทธิ์ | Must |
| US-11-05 | Owner | เปิด/ปิด authRequired | บังคับล็อกอิน | Should |
| US-11-06 | Owner | กำหนดชื่อคอลัมน์/สีที่แก้ได้ | ปรับ planner | Could |
| US-11-07 | ผู้ใช้ | เห็นเฉพาะสิ่งที่ role อนุญาต | ป้องกัน misuse | Must |
| US-11-08 | Owner | เปิด registration / จัดการบัญชี | onboard | Should |

---

## US-11-01/02 — Setup & options

**Read:** `GET /api/setup` (public แต่ redact users/roles เมื่อไม่มีสิทธิ์)
**Write:** `PATCH /api/setup` · `setup.write` — รองรับ language/year/owner/workspaceName/showEditableColors/authRequired/roles
**Add option:** `POST /api/setup/options {list, item}` · `setup.write` — `list ∈ {pillars,formats,goals,statuses,platforms}`

**Main flow (add option)**
1. SettingsPanel เลือก list + พิมพ์ค่า
2. `useAddOption().mutate({list,item})` → `POST`
3. Server: `require_perm("setup.write")` → push ค่า (กันซ้ำตามกติกา) → `SetupConfig`
4. FE invalidate `['setup']`

## US-11-03 — Directory users

- `POST /api/setup/users {name, role}` · `users.manage`
- `DELETE /api/setup/users/{name}` · `users.manage`

**หมายเหตุ:** directory user ผูกกับ permission ผ่าน **ชื่อ** และอาจไม่มี account จริง (ช่องโหว่ G1)

## US-11-04 — Roles & permissions

- `POST /api/setup/roles {name, permissions[]}` · `users.manage` — permission ต้องอยู่ใน `KNOWN_PERMISSIONS` (15 ตัว) ไม่งั้น 400 (`validate_roles`)
- `DELETE /api/setup/roles/{name}` · `users.manage`
- FE role matrix re-seeds เมื่อ server signature เปลี่ยน; ล้มเหลว → revert draft

**Main flow (แก้ role matrix)**
1. ติ๊ก permission ต่อ role
2. `PATCH /api/setup { roles:[...] }` หรือ add/remove role
3. Server validate permission set → บันทึก
4. FE invalidate `['setup']` → `usePermission().can()` คำนวณใหม่ทันที

## US-11-05 — authRequired

`PATCH /api/setup { authRequired }` · `setup.write`
- เปิด = บังคับล็อกอิน (middleware `require_read_auth` เริ่มบล็อก GET ที่ไม่ public)
- ปิด = demo mode เปิด (ทุกคนเห็น/แก้ได้ — ใช้เฉพาะ dev)

## US-11-07 — Permission enforcement (ทุกหน้า)

`usePermission().can(perm)`:
1. `!setup.authRequired` → true ทั้งหมด (demo)
2. หา user ปัจจุบัน → role → permissions.includes(perm)
3. ปุ่ม/ฟิลด์ที่ไม่มีสิทธิ์ถูก disable/hide

Server-side: `require_perm` เป็นด่านจริง (FE เป็นแค่ UX)

---

## Flow — Permission decision (FE + BE)

```mermaid
flowchart TD
  Action["ผู้ใช้กด action"] --> FECan{"usePermission.can(perm)?"}
  FECan -->|no| Disable["disable/hide ปุ่ม"]
  FECan -->|yes| Call["เรียก API"]
  Call --> MW{"read? → require_read_auth"}
  MW --> Perm["require_perm(perm)"]
  Perm --> Admin{"X-Admin-Token?"}
  Admin -->|yes| Allow["อนุญาต (operator)"]
  Admin -->|no| Demo{"authRequired false?"}
  Demo -->|yes| Allow
  Demo -->|no| Sess{"valid session?"}
  Sess -->|no| E401["401"]
  Sess -->|yes| Role{"role.permissions includes perm?"}
  Role -->|no| E403["403"]
  Role -->|yes| Allow
```

## Flow — Setup & roles editing

```mermaid
flowchart TD
  S["SettingsPanel"] --> G["GET /api/setup"]
  G --> Act{"action"}
  Act -->|"แก้ค่า"| P["PATCH /api/setup"]
  P --> P1{"setup.write?"}
  P1 -->|no| P403["403"]
  P1 -->|yes| P2{"roles valid? (KNOWN_PERMISSIONS)"}
  P2 -->|no| P400["400"]
  P2 -->|yes| PSave["บันทึก + invalidate ['setup']"]
  Act -->|"เพิ่ม option"| O["POST /api/setup/options {list,item}"]
  O --> O1{"setup.write?"}
  O1 -->|no| O403["403"]
  O1 -->|yes| OSave["push + invalidate"]
  Act -->|"users"| U["POST/DELETE /api/setup/users"]
  U --> U1{"users.manage?"}
  U1 -->|no| U403["403"]
  U1 -->|yes| USave["บันทึก + invalidate"]
  Act -->|"roles"| R["POST/DELETE /api/setup/roles"]
  R --> R1{"users.manage + permissions valid?"}
  R1 -->|no| R400["400/403"]
  R1 -->|yes| RSave["บันทึก + invalidate → can() คำนวณใหม่"]
```

## Sequence — Onboarding client account (owner)

```mermaid
sequenceDiagram
  autonumber
  actor O as Owner
  participant SP as SettingsPanel
  participant API as handlers.rs
  participant ST as Store

  O->>SP: สร้างบัญชีลูกค้า (name/email/role=Client)
  SP->>API: POST /api/auth/accounts {name,email,role}
  API->>API: validate (ก่อน auth) → require_perm("users.manage")
  alt role ไม่อยู่ / ซ้ำ
    API-->>SP: 400/409
  else ok
    API->>ST: generate cp-<12hex> (ถ้าไม่ส่ง password)
    API->>ST: Argon2id + สร้าง account + ตั้ง directory role
    API-->>SP: { account, temporaryPassword }
    SP->>O: แสดง temp password ครั้งเดียว
    SP->>ST: invalidate ['accounts'] + ['setup']
  end
```
