# Huuk Docs — สารบัญ (Index)

> ภาษาไทยเป็นหลัก + ศัพท์เทคนิคภาษาอังกฤษ (ตาม Kiro docs ต้นฉบับ —
> ปรับให้เข้ากับ Huuk โดยเฉพาะ)

## เปิดอ่านแบบเว็บ (เหมือน kiro.dev)

```sh
cd docs && bun install   # ครั้งแรกครั้งเดียว
bun run docs:dev          # → http://localhost:5174/ (sidebar + search + หน้า home)
bun run docs:build        # build static site (output: docs/.vitepress/dist/)
```

(อ่านเป็นไฟล์ markdown ตรงๆ ก็ได้ — ตารางข้างล่างคือสารบัญเดียวกัน)

> **Content อยู่ไหน?** ไฟล์ `.md` ที่นี่คือ seed source — ตัวจริงถูก serve สด
> จาก Strapi HeadlessCMS (`../cms/`, ดู `../cms/README.md`) เว็บอ่านสดคือ
> `../docs-web/` (http://localhost:5175/) แก้ content ใน Strapi admin
> แล้วเว็บเปลี่ยนทันทีโดยไม่ต้อง build ใหม่

Docs ชุดนี้มี 2 หน้าที่ซ้อนกัน:

1. **คู่มือ Feature Specs แบบ Kiro** — วิธีแตกฟีเจอร์ใหม่เป็น
   `requirements.md` → `design.md` → `tasks.md` อย่างเป็นระบบ
   (รวม EARS notation, Design-First vs Requirements-First, Quick Spec,
   Analyze Requirements, Bugfix Specs)
2. **บริบทของโปรเจกต์ Huuk** — กฎเฉพาะของ repo นี้ที่ spec ทุกฉบับต้องเคารพ
   (data-source swap, 4-place rule, deck navigation, Tailwind v4, TanStack Table v9,
   verification gates)

## แผนที่อ่าน (Reading map)

| # | ไฟล์ | อ่านเมื่อ… |
|---|------|-----------|
| 00 | [00-huuk-overview.md](00-huuk-overview.md) | ต้องการภาพรวม stack / สถาปัตยกรรม / data flow ของ Huuk ก่อนเขียน spec |
| 01 | [01-feature-specs.md](01-feature-specs.md) | ต้องการคู่มือกดใช้งาน: เริ่ม spec ยังไง, เลือก workflow ไหน, gates คืออะไร, รัน tasks + `#spec` อย่างไร |
| 02 | [02-requirements-first.md](02-requirements-first.md) | เริ่มจาก user behavior / product-led (walkthrough ทีละจอ) |
| 03 | [03-design-first.md](03-design-first.md) | เริ่มจาก architecture / tech constraints / เอา design เก่ามา port (walkthrough ทีละจอ) |
| 04 | [04-ears.md](04-ears.md) | ต้องการเขียน requirements ให้ testable ด้วย EARS + พจนานุกรมตัวอย่าง Huuk |
| 05 | [05-design-doc.md](05-design-doc.md) | ต้องการเขียน `design.md` สำหรับ Huuk (components, sequence, API contract, openapi.yaml) |
| 06 | [06-tasks-doc.md](06-tasks-doc.md) | ต้องการแตก `tasks.md` + dependency waves + verification gates (`build` + `test`) |
| 07 | [07-quick-plan-analyze.md](07-quick-plan-analyze.md) | วิธีกดใช้ Quick Spec (Let's build + agent picker), Plan Mode (Q&A + handoff), Analyze Requirements (ปุ่มอยู่ตรงไหน) |
| 08 | [08-bugfix-specs.md](08-bugfix-specs.md) | วิธีกดใช้ Bugfix Spec (Bug Fix + เล่าบั๊ก 3 ส่วน) + property-based testing คืออะไร |
| 09 | [09-best-practices-huuk.md](09-best-practices-huuk.md) | ก่อนเริ่ม implement จริง: กฎ Huuk + gotchas + troubleshooting |
| 10 | [10-glossary-checklists.md](10-glossary-checklists.md) | ต้องการ decision tree, checklists, glossary ไว้เปิดข้างๆ ตอนทำงาน |
| 11 | [11-hands-on-walkthrough.md](11-hands-on-walkthrough.md) | อยากเห็น session จริง 1 รอบ: พิมพ์อะไร → Kiro แสดงอะไร → กดอะไรต่อ |
| — | [templates/](templates/) | เทมเพลตเปล่าพร้อม copy: requirements / design / tasks / bugfix |
| — | [examples/landing-page/](examples/landing-page/) | ตัวอย่าง spec ครบ 3 ไฟล์ของฟีเจอร์จริง (`/landing` — เพิ่งสร้างใน repo นี้) |

## เส้นทางลัดตามบทบาท

- **เพิ่งเข้าทีม / ไม่เคยทำ Kiro specs:** 00 → 01 → 11 (ดูตัวอย่าง session จริง) → 04 → 02 → 06 → 09
- **จะขึ้นฟีเจอร์ใหม่ที่มี tech constraints ชัด:** 00 → 01 → 03 → 05 → 06
- **แก้บั๊กด่วนใน critical path:** 08 → 06 (เฉพาะ tasks ที่เกี่ยวกับ regression tests)
- **ฟีเจอร์เล็กที่มั่นใจแล้ว:** 07 (Quick Spec) → 06 → 09 § verification

## อนุสัญญาของ docs ชุดนี้

- `requirements.md` / `design.md` / `tasks.md` / `bugfix.md` = ชื่อไฟล์ artifact ของ Kiro specs
- **EARS** = Easy Approach to Requirements Syntax — รูปแบบ `WHEN … THE SYSTEM SHALL …`
- **4-place rule** = กฎ Huuk: endpoint ใหม่/เปลี่ยน = แก้ 4 ที่พร้อมกัน
  (`server/src/handlers.rs` → `app/src/api/http.ts` + `app/src/mock/api.ts` →
  `server/openapi.yaml` → tests 2 ฝั่ง)
- คำว่า **demo mode** = `DEMO_MODE=1` (seed accounts, auth optional);
  **production mode** = ไม่มี `DEMO_MODE` (auth required, demo accounts หาย, registration ปิด)
- ตัวอย่างโค้ดอ้าง path จริงใน repo นี้ (เช่น `app/src/core/screens.ts:27`)
  ตรวจสอบ ณ วันที่เขียน — ถ้าไฟล์ย้าย ให้ค้นชื่อไฟล์/ฟังก์ชันแทน

## ที่มาของเนื้อหา

โครงและแนวคิดหลัก (workflows, EARS, 3-phase, task waves, best-practice FAQs)
อ้างอิง Kiro docs หมวด Specs:

- https://kiro.dev/docs/specs/ (ภาพรวม + getting started + task execution + waves)
- https://kiro.dev/docs/specs/feature-specs/ (benefits, variants, EARS, design doc)
- https://kiro.dev/docs/specs/feature-specs/requirements-first/
- https://kiro.dev/docs/specs/feature-specs/tech-design-first/
- https://kiro.dev/docs/specs/quick-spec/ (วิธีกด + FAQ)
- https://kiro.dev/docs/specs/plan/ (วิธีเรียก + workflow + read-only + handoff)
- https://kiro.dev/docs/specs/analyze-requirements/ (วิธีกด + สิ่งที่เกิด)
- https://kiro.dev/docs/specs/bugfix-specs/ (วิธีกด + 3 phases)
- https://kiro.dev/docs/specs/correctness/ (property-based testing)
- https://kiro.dev/docs/specs/best-practices/

ส่วนที่เป็น **Huuk-specific** (screens registry, swap point, permissions,
validation envelopes, Tailwind/TanStack rules, verification gates) สกัดจาก
`AGENTS.md`, `README.md`, `app/README.md`, `server/README.md`,
`server/openapi.yaml` และโค้ดจริง — ไม่ได้มาจาก Kiro
