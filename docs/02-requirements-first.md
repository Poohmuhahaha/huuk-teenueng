# 02 — วิธีใช้ Requirements-First (Walkthrough ทีละจอ)

> workflow หลักของทีม (เริ่มจากพฤติกรรม) เรียบเรียงจาก
> https://kiro.dev/docs/specs/feature-specs/requirements-first/
> ลำดับ: สิ่งที่คุณเห็น → สิ่งที่คุณพิมพ์/กด → สิ่งที่ Kiro ทำให้

## ขั้น 0 — เตรียมประโยคตั้งต้น (1–2 ประโยค)

ก่อนกดเริ่ม เตรียมคำอธิบายฟีเจอร์ให้ตอบ 4 ข้อ: **ใครใช้ / อยากทำอะไร /
ทำไมต้องมี / สำเร็จคืออะไร** ตัวอย่างที่ดี:

```
Build a user authentication system. Users should be able to register with
email/password, login securely, reset forgotten passwords, and logout.
The system must prevent brute force attacks and validate email formats.
```

## ขั้น 1 — สร้าง Feature Spec (กด 3 คลิก)

1. เปิด Kiro panel → หัวข้อ **Specs** → กดปุ่ม **`+`**
   (หรือ chat pane → เลือก **Spec**)
2. Kiro ถาม Feature หรือ Bug → เลือก **Feature**
3. Kiro ถาม workflow → เลือก **Requirements-First**
4. วางประโยคตั้งต้นจากขั้น 0 แล้ว Enter

## ขั้น 2 — Requirements Phase (จอแรกที่คุณต้องตรวจ)

**สิ่งที่ Kiro ทำให้:** สร้างไฟล์ `requirements.md` ประกอบด้วย

- user stories + acceptance criteria
- system behaviors ด้วย **EARS** (`WHEN… THE SYSTEM SHALL…` — อ่านวิธีเขียนที่ 04)
- functional requirements + edge cases + error handling

ตัวอย่างที่ Kiro จะร่างให้:

```markdown
## User Registration
WHEN a user submits valid registration data
THE SYSTEM SHALL create a new user account
WHEN a user submits an email that already exists
THE SYSTEM SHALL display "Email already registered" error
```

**สิ่งที่คุณต้องทำ (อย่ากดผ่าน):**

1. อ่านทุก user story — behavior ตรงกับที่ต้องการไหม
2. พิมพ์สั่งแก้ตรงๆ ได้เลย เช่น *"add a requirement for password reset expiry"*
   หรือแก้ไฟล์ `requirements.md` เอง
3. เติม scenarios/edge cases ที่หาย (เช่น brute-force lockout, email format)
4. (แนะนำ) กด **Analyze Requirements** ให้ Kiro ตรวจข้อขัดแย้งก่อนไปต่อ — วิธีดู 07
5. พอใจแล้ว → **ยืนยัน (Continue / Proceed to Design)** เพื่อไปขั้น 3

> เคล็ดลับ: พลังของ Requirements-First คือเอา **"what" ให้ถูกก่อน commit "how"**
> ใช้เวลาตรงนี้เยอะๆ เอา stakeholder มาอ่าน requirements ด้วย (ไม่ใช่มาอ่านโค้ด)

## ขั้น 3 — Design Phase (จอที่สอง)

**สิ่งที่ Kiro ทำให้:** สร้างไฟล์ `design.md` ประกอบด้วย

- system architecture + components
- sequence diagrams (ปฏิสัมพันธ์ระหว่าง components)
- data models + interfaces, technology stack, error handling, testing strategy

**สิ่งที่คุณต้องทำ:**

1. อ่าน technical approach — ตอบทุก requirement ไหม
2. ติ architectural decisions / technology choices (สั่งในแชทได้:
   *"use Redis instead of in-memory for sessions because…"*)
3. ตรวจ non-functional: performance, maintainability
4. พอใจแล้ว → **ยืนยัน** เพื่อไปขั้น 4

## ขั้น 4 — Tasks Phase (จอที่สาม)

**สิ่งที่ Kiro ทำให้:** สร้างไฟล์ `tasks.md` — งานย่อย discrete พร้อม
คำอธิบาย + ผลลัพธ์ที่คาด + dependencies + แยก required/optional

**สิ่งที่คุณต้องทำ:**

1. อ่าน task breakdown — ย่อยพอรันทีละข้อไหม
2. ปรับ priority / ติ๊กข้อที่ไม่เอาเป็น optional
3. เริ่ม implement (ทีละ task หรือ **Run all Tasks** — วิธีดู 01 §01.6)

## ขั้น 5 — Implementation

- รันทีละ task ดูสถานะ real-time ใน `tasks.md` หรือกด **Run all Tasks**
  ให้ waves ขนานกัน
- แก้ spec กลางทางได้: เปิดสเปกแชทสั่ง เช่น
  *"#spec:user-authentication update the design to include password reset"*
  แล้วกด **Sync Files** ใน `tasks.md` ให้ tasks ตามทัน

## ปัญหาที่เจอบ่อย + วิธีกดแก้

| อาการ | วิธีกดแก้ |
|---|---|
| Design ไม่ตอบ requirements | กลับไปเติม acceptance criteria ใน `requirements.md` ให้เฉพาะเจาะจง → สั่ง regenerate design |
| Requirements กำกวม | เติม scenarios/edge cases ใน prompt → เขียนด้วย EARS (ดู 04) |
| ต้องเปลี่ยน requirements หลังมี design แล้ว | แก้ `requirements.md` → เปิดสเปกแชทสั่ง *"update the design to match"* → Kiro อัปเดต design + tasks ให้ |

## รูปแบบงานที่ใช้ workflow นี้บ่อย (3 patterns)

1. **Product Feature** — เอา user stories ของ PM วางเป็นประโยคตั้งต้น →
   Kiro formalize เป็น EARS → iterate กับ PM → design → validate กับ eng → implement
2. **Customer-Driven** — เล่าความต้องการลูกค้า → requirements จับ intent →
   เอากลับไป validate กับลูกค้า → design → implement + deliver
3. **Greenfield** — นิยาม core journeys → requirements ชุดใหญ่ →
   review design เรื่อง scalability → tasks สร้าง MVP

> ตัวอย่าง spec ฉบับ Huuk (landing page `/landing` ที่สร้างจริงใน repo นี้):
> [examples/landing-page/requirements.md](examples/landing-page/requirements) — เทียบ
> `requirements.md` กับหน้าจอจริงได้เลย
