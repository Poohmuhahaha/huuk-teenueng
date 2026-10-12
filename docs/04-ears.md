# 04 — EARS Notation (เขียน requirements ให้ testable)

> ขยายจาก https://kiro.dev/docs/specs/feature-specs/#requirements-with-ears-notation
> + พจนานุกรมตัวอย่างที่สกัดจาก `server/README.md` ของ Huuk โดยเฉพาะ

## 04.1 รูปแบบ (Syntax)

```
WHEN [condition / event] THE SYSTEM SHALL [expected behavior]
```

- **WHEN** = เงื่อนไขหรือเหตุการณ์ที่กระตุ้น (รวม state ที่มีผล เช่น `authRequired is on`)
- **THE SYSTEM SHALL** = พฤติกรรมที่คาดหวัง — เขียนให้ **สังเกต/วัดได้**
  (status code, message ตรงตัว, state ที่เปลี่ยน, หน้าที่ render)

ตัวอย่างต้นฉบับ Kiro:

```
WHEN a user submits a form with invalid data
THE SYSTEM SHALL display validation errors next to the relevant fields
```

## 04.2 ทำไมต้อง EARS (Benefits)

| ประโยชน์ | หมายความว่า |
|----------|-------------|
| **Clarity** | ไม่กำกวม — ไม่มีคำว่า "เหมาะสม", "เร็ว", "ใช้งานง่าย" ลอยๆ |
| **Testability** | แปลเป็น test case ได้ 1:1 (condition = arrange/act, SHALL = assert) |
| **Traceability** | ตามรอยแต่ละข้อ → test → implementation (ใส่ ID เช่น REQ-007) |
| **Completeness** | บังคับให้คิดทุก condition: happy path + edge + error + permission |

## 04.3 กฎการเขียน (Rules — Huuk edition)

1. **1 requirement = 1 พฤติกรรม** — อย่ายัด 2 SHALL ในข้อเดียว
2. **SHALL ต้องวัดได้** — ระบุ status code + error message ตรงตัว
   (เช่น `409 `post is locked by …``, ไม่ใช่ "แจ้งเตือนอย่างเหมาะสม")
3. **ระบุ mode ที่เกี่ยว** — `WHEN authRequired is on …` /
   `WHEN DEMO_MODE=1 …` เพราะ Huuk พฤติกรรมต่างกัน 2 modes
4. **ระบุ permission** — ถ้าเป็น mutating endpoint ให้มีข้อ 401/403 กำกับเสมอ
5. **ใช้ SHALL / SHALL NOT / SHALL CONTINUE TO**:
   - `SHALL` = ต้องทำ
   - `SHALL NOT` = ต้องไม่ทำ (เช่น `SHALL NOT expose password hashes`)
   - `SHALL CONTINUE TO` = ต้องคงพฤติกรรมเดิม (ใช้ใน bugfix — ดู 08)
6. **ตั้ง ID** (`REQ-001…`) ทุกข้อ เพื่อ trace → tasks → tests

## 04.4 พจนานุกรม EARS ของ Huuk (copy ไปใช้ได้เลย)

### Auth & sessions

```
REQ-A01  WHEN a user registers with a new email and password ≥ 8 chars
         THE SYSTEM SHALL create the account, auto-login, and return a 64-hex-char token
REQ-A02  WHEN a user registers with an email that already exists (case-insensitive)
         THE SYSTEM SHALL reject with 409 `email already registered`
REQ-A03  WHEN a user logs in with an unknown email or a wrong password
         THE SYSTEM SHALL reject with 401 `invalid email or password` (same message both cases)
REQ-A04  WHEN a session older than 30 days is used
         THE SYSTEM SHALL reject with 401, delete the expired session, and require re-login
REQ-A05  WHEN authRequired is on and a mutating request has no/invalid token
         THE SYSTEM SHALL reject with 401 `missing bearer token` / `invalid token`
REQ-A06  WHEN authRequired is on and the caller's role lacks the required permission
         THE SYSTEM SHALL reject with 403 `missing permission: <perm>`
REQ-A07  WHEN a user changes password with a correct old password and new password ≥ 8 chars
         THE SYSTEM SHALL keep the caller's token valid and revoke every other session of that account
```

### Posts & locks

```
REQ-P01  WHEN a post is created with a client-sent `lockedBy`
         THE SYSTEM SHALL ignore it and store null
REQ-P02  WHEN authRequired is on and a signed-in user locks/unlocks/patches a post
         THE SYSTEM SHALL derive the holder from the session and ignore client-sent `user`
REQ-P03  WHEN a different user patches/locks/unlocks a post held by another user
         THE SYSTEM SHALL reject with 409 `post is locked by …`
REQ-P04  WHEN a post is patched with an unknown id
         THE SYSTEM SHALL return 404
```

### CMS content (optimistic concurrency)

```
REQ-C01  WHEN content is updated/published/scheduled/restored with a stale `version`
         THE SYSTEM SHALL reject with 409 and store a revision before the failed write
REQ-C02  WHEN a scheduled item's `scheduledFor` has passed and any content read happens
         THE SYSTEM SHALL lazily flip it to published (no cron required)
REQ-C03  WHEN an anonymous user calls the public delivery API for published items
         THE SYSTEM SHALL succeed without auth and never return non-published items
REQ-C04  WHEN content is deleted without the `content.delete` permission
         THE SYSTEM SHALL reject with 403
```

### Setup / roles / options

```
REQ-S01  WHEN an option/tag duplicates an existing entry
         THE SYSTEM SHALL ignore it and leave the list unchanged
REQ-S02  WHEN a roles replacement is empty, has duplicates, has unknown permissions,
         or drops a role a directory user holds
         THE SYSTEM SHALL reject the whole patch with 400 and half-apply nothing (atomic)
REQ-S03  WHEN the last user / last role / an in-use role is deleted
         THE SYSTEM SHALL reject with 400
REQ-S04  WHEN `GET /api/auth/accounts` is called
         THE SYSTEM SHALL list accounts with live session counts and SHALL NOT expose hashes
```

### Frontend behavior

```
REQ-F01  WHEN VITE_API_URL is unset in DEV
         THE SYSTEM SHALL use the in-memory mock and pages SHALL NOT import @/mock directly
REQ-F02  WHEN a mutation in core/queries.ts succeeds
         THE SYSTEM SHALL invalidate every affected query key so the UI refreshes
REQ-F03  WHEN a computed field is edited
         THE SYSTEM SHALL open ProtectedModal with Cancel as default
REQ-F04  WHEN a new screen is added
         THE SYSTEM SHALL register it only in core/screens.ts (order = deck order)
REQ-F05  WHEN a public page (meta.public) is rendered
         THE SYSTEM SHALL render RouterView without the AppShell
```

## 04.5 Anti-patterns (ตัวอย่างผิด → แก้)

| ❌ ผิด (vague) | ✅ แก้ด้วย EARS |
|---|---|
| ระบบต้องล็อกอินเร็ว | `WHEN a user submits valid credentials THE SYSTEM SHALL return a token within 2s (p95, local)` |
| แจ้งเตือนเมื่อเกิดข้อผิดพลาดอย่างเหมาะสม | `WHEN registration email exists THE SYSTEM SHALL reject with 409 `email already registered`` |
| รองรับหลาย platform | `WHEN Meta OAuth completes with credentials set THE SYSTEM SHALL store handle/externalId/scopes/token/expiry and connect the Page + linked Instagram` |
| ห้ามพัง | `WHEN a Meta refresh fails THE SYSTEM SHALL keep the existing mirror and set error explaining staleness` |

## 04.6 จาก EARS → Test (1:1 mapping)

ทุก `REQ-*` แปลเป็น test ได้ตรงๆ:

| ส่วนของ EARS | ส่วนของ test |
|---|---|
| `WHEN <condition>` | arrange + act (setup state → call) |
| `THE SYSTEM SHALL <behavior>` | assert (status + body + state) |
| ฝั่ง server | `server/tests/api.rs` ผ่าน `tower::oneshot` (ไม่ใช้ network) |
| ฝั่ง frontend | `app/src/tests/*.test.ts` (vitest + happy-dom, query ด้วย role/text/tag) |

ตัวอย่าง: `REQ-P03` → test "patch locked post → 409" ทั้ง 2 ฝั่ง
(backend: status + message; frontend: alert role ปรากฏ)
