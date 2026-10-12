# 01 — คู่มือใช้งาน Feature Specs (User Manual)

> สอนกดใช้งานจริงทีละขั้น แปล + เรียบเรียงจาก
> https://kiro.dev/docs/specs/ และ https://kiro.dev/docs/specs/feature-specs/
> ชื่อปุ่ม/หน้าจอคงภาษาอังกฤษตามที่ Kiro แสดงจริง คำอธิบายเป็นภาษาไทย

## 01.1 Feature Specs ใช้ทำอะไร (What — 30 วินาที)

Feature Specs คือตัวช่วยสร้างฟีเจอร์ใหม่แบบเป็นขั้นเป็นตอนใน Kiro:
คุณเล่าไอเดีย → Kiro ร่าง **requirements** (จะสร้างอะไร) →
คุณกดยืนยัน → Kiro ร่าง **design** (จะสร้างอย่างไร) →
คุณกดยืนยัน → Kiro แตก **tasks** (งานย่อยที่กดรันได้ทีละข้อ)
ทุกขั้นมีจุดให้คุณตรวจก่อนไปต่อ (approval gate) — ไม่ใช่สั่งแล้วได้โค้ดเลย

**ใช้ Kiro บนพื้นไหนได้บ้าง:**

| ความสามารถ | IDE | CLI | Web | Mobile |
|---|---|---|---|---|
| Feature Specs | ✓ | ✓ | ✓ | — |
| Bugfix Specs | ✓ | ✓ | ✓ | — |
| Quick Spec | ✓ | ✓ | ✓ | — |
| รัน tasks ขนาน (parallel) | ✓ | ✓ | ✓ | — |
| Analyze Requirements | ✓ | ✓ | — | — |
| Correctness (property-based testing) | ✓ | — | — | — |

## 01.2 เริ่มใช้งานครั้งแรก (Getting Started)

มี 3 ทางเข้า เลือกทางที่ถนัด — ทุกทางพาไป flow เดียวกัน:

**ทางที่ 1 — Kiro pane (IDE, แนะนำสำหรับมือใหม่):**

1. เปิด **Kiro pane** (แถบข้าง)
2. หาหัวข้อ **Specs** แล้วกดปุ่ม **`+`**
   (อีกทาง: เปิด chat pane แล้วเลือก **Spec** จากตัวเลือก)

**ทางที่ 2 — หน้า "Let's build" (IDE / CLI / Web):**

1. เริ่ม session ใหม่จนเจอจอ **"Let's build"**
2. Kiro จะถามว่า Developing a **Feature** หรือ fixing a **Bug**
3. เลือก **Feature** → พิมพ์อธิบายฟีเจอร์สั้นๆ → เลือก workflow
   (**Requirements-First** หรือ **Design-First**, ดู §01.4 ว่าต่างกันอย่างไร)

**ทางที่ 3 — เริ่มจากบทสนทนา vibe (ใช้ได้ทุกพื้น):**

1. คุยกับ Kiro ตามปกติจนไอเดียชัด
2. พิมพ์ว่า **`Generate spec`**
3. Kiro จะถามว่าต้องการเริ่ม spec session ไหม → ตอบ **yes**
   Kiro จะเอา context บทสนทนามาสร้าง requirements ให้เลย

> **ตั้งค่า default workflow:** ใน Kiro settings ตั้ง workflow เริ่มต้นได้
> (Requirements-First / Design-First / Quick Spec) — ตั้งแล้วจะข้ามขั้นเลือก workflow
> ไปเลย ทีมควรตกลงกันครั้งเดียวว่าจะใช้ค่าอะไร

## 01.3 ของที่ได้: ไฟล์ 3 ไฟล์ (Core Structure)

ทุก spec จะสร้างไฟล์ 3 ไฟล์นี้ (อยู่ใน `.kiro/specs/<ชื่อฟีเจอร์>/`):

| ไฟล์ | ข้างในคืออะไร | เปรียบเหมือน |
|---|---|---|
| `requirements.md` | user stories + acceptance criteria (เขียนด้วย EARS — ดู 04) | ใบสั่งงานจากลูกค้า |
| `design.md` | architecture, sequence diagrams, วิธี implement | แบบแปลนจากสถาปนิก |
| `tasks.md` | งานย่อยทีละข้อ พร้อม checkbox ติดตามสถานะ | เช็กลิสต์ช่างหน้างาน |

(Bugfix Spec ใช้ `bugfix.md` แทน `requirements.md` — ดู 08)

## 01.4 เลือก workflow ไหน (2 แบบ + Quick Spec)

หลังเลือก **Feature** Kiro จะให้เลือก workflow — ตัดสินใจด้วยคำถามเดียว:

> **"ตอนนี้ในหัวคุณชัดกว่ากัน: พฤติกรรมที่อยากได้ หรือ วิธีสร้าง?"**

- ชัดเรื่อง**พฤติกรรม** (อยากให้ระบบทำอะไร) → **Requirements-First**
  เริ่มจาก behavior → ค่อยออกแบบ (เหมาะกับงาน product-led, greenfield)
  วิธีใช้ละเอียด: [02-requirements-first.md](02-requirements-first.md)
- ชัดเรื่อง**วิธีสร้าง** (มี architecture/tech ในใจ) → **Design-First**
  เริ่มจาก design → ค่อย derive requirements ที่ทำได้จริง
  (เหมาะกับงานที่มี constraints เข้ม: latency, throughput, compliance)
  วิธีใช้ละเอียด: [03-design-first.md](03-design-first.md)

| | Requirements-First | Design-First |
|---|---|---|
| **เริ่มจาก** | system behavior (requirements) | technical design / pseudocode |
| **Kiro สร้างต่อให้** | design จาก requirements | requirements จาก design |
| **เหมาะกับ** | product-driven development | technically-constrained projects |
| **รับประกัน** | behavior ที่ต้องการถูก specify | technical feasibility |

> **ทางลัด:** ฟีเจอร์ที่เข้าใจดี + ไว้ใจ output ของ Kiro →
> เลือก **[Quick Spec](07-quick-plan-analyze.md)** วิ่ง 3 phases รวดเดียว
> ไม่มี approval gates มาตอบ clarifying questions ตอนต้นครั้งเดียวจบ

## 01.5 วิธีเดิน flow มาตรฐาน (Three-Phase Workflow)

```
Phase 1: Requirements (หรือ Bug Analysis)  →  คุณกดยืนยัน  ─┐
Phase 2: Design                            →  คุณกดยืนยัน  ─┤ approval gates
Phase 3: Tasks (งานย่อยที่รันได้)          →  เริ่ม implement ┘
```

**Phase 1 — Requirements:** Kiro ร่าง `requirements.md` จากคำอธิบายของคุณ
(user stories + acceptance criteria ด้วย EARS) → **คุณอ่าน แก้ไข เพิ่ม edge cases
จนพอใจ แล้วกดยืนยัน** → ถ้ายังไม่แน่ใจ กด **Analyze Requirements** ให้ Kiro
ตรวจข้อขัดแย้ง/กำกวมก่อน (ดู 07)

**Phase 2 — Design:** Kiro ร่าง `design.md` (architecture + sequence diagrams +
testing strategy) → **คุณอ่าน ตรวจ technology choices ติ architectural decisions
แล้วกดยืนยัน**

**Phase 3 — Tasks:** Kiro แตก `tasks.md` (งานย่อย discrete + dependencies +
required/optional) → **คุณอ่าน ปรับ priority แล้วเริ่ม implement**

## 01.6 วิธีรัน tasks (Task Execution UI)

ไฟล์ `tasks.md` ใน Kiro มีหน้าจอรันงานในตัว:

- สถานะอัปเดต **real-time** (in-progress / completed) ขณะ implement
- รัน**ทีละ task** หรือกด **Run all Tasks** รันทั้งหมดในครั้งเดียว
- กด **Run all Tasks**: Kiro สร้าง **dependency graph** แล้วจัด tasks อิสระเป็น
  **waves** รันขนานกัน (ไม่ต้องตั้งค่าอะไร):
  - **Wave 1** — tasks ที่ไม่มี dependencies → รันพร้อมกัน
  - **Wave 2** — tasks ที่รอ Wave 1 เสร็จ → รันพร้อมกัน
  - **Wave N** — จนครบ (waves วิ่งตามลำดับ, tasks ใน wave เดียวกันขนานกัน)
- รันเฉพาะ tasks ที่ยังไม่เสร็จและ marked เป็น required
- งานไหนเพื่อน/คุณทำไปแล้ว: เปิด `tasks.md` → กด **Sync Files**
  (หรือสั่งในสเปกแชท: *"check which tasks are already complete"*
  ให้ Kiro scan codebase แล้วติ๊ก tasks ที่เสร็จให้อัตโนมัติ)

## 01.7 วิธีอ้าง spec ในแชท (#spec)

พิมพ์ `#spec` + Enter ในช่องแชท → เลือก spec จากรายการ →
Kiro ดึงทั้ง 3 ไฟล์เข้า context บทสนทนา ใช้เมื่อ:

- **สั่ง implement:** `#spec:user-authentication implement task 2.3`
  (โค้ดที่ได้จะตรง design decisions + acceptance criteria)
- **ขอแก้ spec:** `#spec:user-authentication update the design file to include password reset flow`
- **ตรวจงาน:** `#spec:user-authentication does my current implementation meet the acceptance criteria for task 7.1?`
- **ถามเหตุผล:** `#spec:user-authentication why did we choose JWT over session-based authentication?`

## 01.8 จัดหลาย specs ใน repo เดียว

มีได้ไม่จำกัด — **แนะนำแยก spec ต่อฟีเจอร์** อย่ารวมทั้ง codebase ใน spec เดียว:

```
.kiro/specs/
├── user-authentication/   # login, signup, password reset
├── product-catalog/       # listing, search, filtering
├── shopping-cart/         # add to cart, checkout
└── admin-dashboard/       # product management, analytics
```

ข้อดี: ทำอิสระไม่ตีกัน, เอกสารโฟกัส, iterate เฉพาะจุด, ทำขนานกันได้หลายคน
(แชร์ข้ามทีมด้วย Git submodules / package references / central specs repo — ดู 09)

## 01.9 เริ่มจากตรงไหนต่อ (Next steps)

- ไป [02-requirements-first.md](02-requirements-first.md) — เดิน flow หลักทีละจอ
- ไป [11-hands-on-walkthrough.md](11-hands-on-walkthrough.md) — ตัวอย่าง session
  จริงตั้งแต่พิมพ์ประโยคแรกจนรัน tasks
- กฎเฉพาะ repo Huuk (4-place rule, deck, verification gates) อยู่ที่
  [00-huuk-overview.md](00-huuk-overview.md) และ [09-best-practices-huuk.md](09-best-practices-huuk.md)
