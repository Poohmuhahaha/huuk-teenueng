# 10 — Glossary, Decision Tree, Checklists

> เปิดไฟล์นี้ค้างไว้ข้างๆ ตอนทำงาน — รวมสิ่งที่ต้องเช็กบ่อยๆ ที่เดียว

## 10.1 Glossary

| ศัพท์ | ความหมาย |
|---|---|
| **Spec** | specification — artifact ที่ formalize กระบวนการ dev (requirements/design/tasks) |
| **Feature Spec** | spec สำหรับสร้างฟีเจอร์ใหม่ (vs Bugfix Spec) |
| **Bugfix Spec** | spec สำหรับแก้บั๊กแบบกัน regression (`bugfix.md` แทน `requirements.md`) |
| **Quick Spec** | Feature Spec แบบวิ่งรวดเดียว ไม่มี approval gates |
| **requirements.md** | user stories + acceptance criteria (EARS) — *what* |
| **design.md** | architecture + sequences + API contract — *how* |
| **tasks.md** | implementation tasks ย่อย + dependencies — *do* |
| **bugfix.md** | bug analysis: repro + current/expected/unchanged — ดู 08 |
| **EARS** | Easy Approach to Requirements Syntax (`WHEN… THE SYSTEM SHALL…`) |
| **SHALL / SHALL NOT / SHALL CONTINUE TO** | ต้องทำ / ต้องไม่ทำ / ต้องคงพฤติกรรมเดิม |
| **Approval gate** | จุด review คั่นระหว่าง phases (Quick Spec ไม่มี) |
| **Wave** | กลุ่ม tasks อิสระที่รันขนานกันได้ (Wave 1 → 2 → N) |
| **Sync Files** | สั่งให้ tasks sync กับ requirements/design ที่เปลี่ยน |
| **4-place rule** | กฎ Huuk: endpoint ใหม่/เปลี่ยน = แก้ 4 ที่พร้อมกัน |
| **Swap point** | `app/src/api/index.ts` — จุดสลับ mock ↔ HTTP |
| **Deck** | navigation แบบ slide deck (URL ตาม slide) |
| **Hub / tab** | hub = การ์ดใน deck (6 อัน), tab = หน้าย่อยใน hub (`HUB_TABS`) |
| **Demo / production mode** | `DEMO_MODE=1` (open, seeded) vs ไม่มี (auth, no demos, closed reg) |
| **`x-permission`** | ฟิลด์ใน `openapi.yaml` ระบุ permission ของ endpoint |

## 10.2 Decision tree (เลือก workflow)

```
จะสร้างของใหม่? ──ใช่──→ รู้ behavior ชัด / arch ยืดหยุ่น? ──ใช่──→ Requirements-First (02)
     │                          └──ไม่ (มี arch/constraints มาก่อน)──→ Design-First (03)
     │
     └──ไม่ (ของพัง)──→ บั๊ก critical / เคย regression / root ไม่ชัด? ──ใช่──→ Bugfix Spec (08)
                               └──ไม่ (typo/one-liner)──→ quick fix ในแชท
                                          +
              ฟีเจอร์เล็ก + เข้าใจดี + ไม่แตะ auth/permissions/locks? ──ใช่──→ Quick Spec (07)
```

## 10.3 Checklists

### Requirements gate (ก่อนไป design)

- [ ] ทุกข้อเป็น EARS + วัดได้ (status/message ตรงตัว)
- [ ] ทุกข้อมี ID (REQ-*) และไม่มี 2 SHALL ในข้อเดียว
- [ ] mutating ทุกข้อมี 401/403 คู่; `{id}` ทุกข้อมี 404; lock/version มี 409
- [ ] demo vs production แยกชัดทุกข้อที่เกี่ยว
- [ ] edge ครบ: empty, duplicate, last-user/role, unknown permission, malformed JSON
- [ ] ผ่าน Analyze checklist (07 §07.3)

### Design gate (ก่อนไป tasks)

- [ ] ทุก requirement มี component/endpoint รองรับ (trace ครบ)
- [ ] API table + openapi diff + permission matrix ครบ
- [ ] mock/HTTP parity วางแผน (ชื่อฟังก์ชันตรงกัน)
- [ ] sequence ครอบ happy + error paths
- [ ] non-goals เขียนไว้

### Tasks gate (ก่อน implement / ก่อนปิดงาน)

- [ ] tasks discrete + มี REQ trace + required/optional แยกชัด
- [ ] 4-place rule ครบทุก task ที่แตะ endpoint
- [ ] frontend rules เช็ก (screens, invalidateQueries, styling, alias, v9, tests query)
- [ ] `bun run build` ✅ **และ** `bun run test` ✅
