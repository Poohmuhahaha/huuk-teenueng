# 08 — คู่มือใช้งาน Bugfix Specs (+ Property-based Testing)

> แก้บั๊กแบบกัน regression — เรียบเรียงจาก
> https://kiro.dev/docs/specs/bugfix-specs/ ,
> https://kiro.dev/docs/specs/correctness/ (property-based tests)
> และ best-practices หมวด Bugfix

## 08.1 มันคืออะไร (ต่างจาก Feature Spec ตรงไหน)

Bugfix Spec คือวิธีแก้บั๊กแบบนักพัฒนารุ่นใหญ่: **หา root cause →
เข้าใจว่าอะไรต้องเปลี่ยน → ระบุชัดว่าอะไรห้ามเปลี่ยน**
flow 3 phases เหมือน Feature Spec แต่ phase แรกเป็น **bug analysis**
(ไฟล์ `bugfix.md` แทน `requirements.md`) และ tasks มาพร้อม
**property-based tests** กัน regression

**ประโยชน์ 4 ข้อ:** scoped fixes (constraints กันแก้เกิน) ·
regression prevention (unchanged behavior ถูก document + test) ·
documentation (บันทึก bug+fix+reasoning) · reliability (กัน pitfalls ของ ad-hoc fix)

**ใช้เมื่อ (เช็ก 1 ใน 4):** บั๊กซับซ้อนต้อง root-cause · critical path
ที่ regression แพง · ต้องมีเอกสาร (compliance/team knowledge) ·
เคยแก้แล้วพังที่อื่นมาก่อน
*นอกนั้น (typo, one-liner ที่เข้าใจดี) quick fix ในแชทพอ*

## 08.2 วิธีกดใช้ (Getting Started)

ใช้ได้บน: **IDE ✓ / CLI ✓ / Web ✓** (Mobile ไม่มี)

1. จอ **"Let's build"** ตอนเริ่ม session → เลือก **Bug Fix**
   (กลาง session: คลิกชื่อ agent ใน bottom bar → เลือก **Bug Fix**)
2. **เล่าบั๊ก 3 ส่วน** (ยิ่งครบ Kiro ยิ่ง root-cause แม่น):
   - **When** — reproduction steps (เกิดเมื่อไหร่ ขั้นตอนเป๊ะๆ)
   - **What instead** — ควรเกิดอะไรแทน
   - **Constraints** — โค้ดส่วนไหนห้ามแตะ
3. เดิน 3 phases ข้างล่าง → implementation

## 08.3 Phase 1 — Bugfix Analysis (ไฟล์ `bugfix.md`)

Kiro สร้าง `bugfix.md` โครง 3 ส่วนนี้ (จำ pattern นี้ไว้ — ใช้เขียนเองก็ได้):

**Current Behavior (ของที่พัง):**

```
WHEN [condition] THEN the system [incorrect behavior]
```

**Expected Behavior (ของที่ถูก):**

```
WHEN [condition] THEN the system SHALL [correct behavior]
```

**Unchanged Behavior (ของที่ห้ามพัง — หัวใจกัน regression):**

```
WHEN [condition] THEN the system SHALL CONTINUE TO [existing behavior]
```

โครงนี้ทำให้ Kiro เข้าใจพร้อมกันว่า *อะไรพัง* และ *อะไรต้องยังใช้ได้*

## 08.4 Phase 2 — Design (root cause + fix approach)

Kiro อ่านโค้ดหา root cause แล้วร่าง `design.md` ประกอบด้วย:

- root cause analysis
- proposed fix approach
- **properties to test** 3 กลุ่ม:
  1. implementation ปัจจุบันผลิต incorrect behavior (ยืนยันว่าบั๊กมีจริง)
  2. implementation ที่แก้แล้วผลิต correct behavior (ยืนยันว่าแก้ได้)
  3. ส่วน unchanged ยังทำงานเหมือนเดิม (กัน regression)

## 08.5 Phase 3 — Tasks (มาพร้อม property-based tests)

tasks ที่ generate มาพร้อม **property-based tests (PBTs)** ที่ validate 3 เรื่อง:
บั๊ก reproduce ได้ · บั๊กถูกแก้ · ไม่มี regression

### Property-based testing คืออะไร (สรุปแบบใช้งาน)

- **Test แบบเดิม (example-based):** เขียนเคสทีละตัวอย่าง
  (เช่น "User A เพิ่ม Car #5 → Car #5 โผล่ใน list") — coverage เท่าที่คุณนึกออก
- **PBT:** ระบุ **property** (กฎสากลที่ต้องจริงเสมอ เช่น
  *"WHEN user adds ANY car THE SYSTEM SHALL display it in their list"*)
  แล้วเครื่อง **generate inputs สุ่มหลายร้อย–พันเคส** มาพยายามแหกกฎ
  (user ชื่อประหลาด, รถสถานะแปลกๆ, ค่าว่าง, boundary — เคสที่คุณไม่เขียนเอง)
- เจอ failure input ใหญ่ๆ เครื่องจะ **shrink** ให้เหลือ input เล็กสุดที่ยังพัง
  (เช่น username 200 ตัวอักษร → เหลือ string ว่าง) ทำให้เห็น root cause ทันที
- Kiro ชงต่อได้: อัปเดต implementation ให้ หรือเสนอให้เลือกว่าจะแก้
  spec / implementation / test

**ใน workflow:** design phase Kiro สกัด properties จาก EARS requirements
(hover ดูได้ว่า property นี้มาจาก requirement ข้อไหน + task ไหน) →
execution phase รัน PBT cases กับ implementation (PBT เป็น **optional by default**
โฟกัส core implementation ก่อนได้) → fail ปุ๊บ Kiro ชี้ failure scenario ให้
review → คุยกับ Kiro ว่าจะแก้ implementation / test / requirement

**ข้อควรรู้:** PBT ให้ *evidence* ของ correctness ไม่ใช่ *proof*
(ไม่ใช่ formal verification) — ผ่านแล้วมั่นใจขึ้นมากแต่ไม่รับประกันไร้บั๊ก;
property ที่อ่อน/ผิดจะเขียวทั้งที่ behavior ผิด; requirement ที่พึ่ง external
services/non-determinism อาจต้อง mock หรือใช้ example-based tests แทน

> PBT ใช้ได้บน **IDE เท่านั้น** (CLI/Web ไม่มี)

## 08.6 เขียน bug description ดีๆ (3 ส่วน + ตัวอย่าง)

1. **Reproduction steps** — เงื่อนไขเป๊ะๆ ที่ trigger
2. **Current vs Expected** — ตอนนี้ทำอะไร / ควรทำอะไร
3. **Constraints** — อะไรห้ามเปลี่ยน

ตัวอย่างที่ดี:

```
When a user submits a form with special characters in the name field
(e.g., O'Brien), the API returns a 500 error. It should accept the
input and save it correctly. The validation logic for email and
password fields should not be affected.
```

## 08.7 กฎกัน scope creep

- ถ้า root cause เผยว่าต้องมี functionality ใหม่ก้อนใหญ่ →
  **อย่าบวม bugfix** เปิด **Feature Spec แยก** สำหรับงานใหม่
  (ไม่มีปุ่ม convert — สร้างใหม่แล้วอ้าง bugfix นี้เป็น motivation)
- วิธีกัน regression ของ Huuk เพิ่มเติม: ทุก fix แตะ endpoint ต้องผ่าน
  **4-place rule + tests 2 ฝั่ง** (ดู 00/06) ไม่ใช่แค่ฝั่งที่พัง
