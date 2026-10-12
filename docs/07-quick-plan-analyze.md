# 07 — คู่มือใช้งาน Quick Spec, Plan Mode, Analyze Requirements

> 3 เครื่องมือรอบๆ Feature Specs หลัก — เรียบเรียงจาก
> https://kiro.dev/docs/specs/quick-spec/ ,
> https://kiro.dev/docs/specs/plan/ ,
> https://kiro.dev/docs/specs/analyze-requirements/
> เน้นวิธีกด + สิ่งที่เห็นบนจอ

---

## PART A — วิธีใช้ Quick Spec (วิ่งรวดเดียว ไม่มี gates)

### A.1 มันคืออะไร (ต่างจาก standard ยังไง)

Quick Spec สร้าง `requirements.md` + `design.md` + `tasks.md` **ใน pass เดียว**
คุณตอบ clarifying questions ตอนต้นครั้งเดียว แล้วลง task list เลย
artifacts ที่ได้เหมือน standard ทุกประการ (อยู่ใน `.kiro/specs/` เหมือนกัน)
แก้ทีหลังได้เหมือนกัน — ต่างแค่ว่า**ไม่มี approval gates คั่น**

เทียบกับ Vibe mode: Vibe คุยปากเปล่าไม่มี artifacts เซฟลง `.kiro/specs/`
แต่ Quick Spec มีครบ 3 ไฟล์

### A.2 วิธีกดใช้ (2 ทาง)

**ทางที่ 1 — ตอนเริ่ม session:**

1. เปิดจอ **"Let's build"**
2. ใน workflow options เลือก **Quick Spec**
3. ตอบ clarifying questions ของ Kiro (scope, constraints, edge cases —
   จุดนี้คือจุด input หลักของคุณ ตอบดี specs ดี)
4. Kiro generate 3 ไฟล์ตามลำดับโดยไม่หยุดถาม → คุณลงที่ task list พร้อม implement

**ทางที่ 2 — สลับกลาง session:**

1. คลิก**ชื่อ agent** ใน bottom bar ของช่อง chat input
2. เลือก **Quick Spec** จาก agent picker

### A.3 ใช้เมื่อไหร่ (เช็กลิสต์ตัดสินใจ)

**ใช้ Quick Spec เมื่อ:**

- ฟีเจอร์ที่เข้าใจดี + ไว้ใจ output ของ Kiro (ไม่ต้อง iterate requirements/design)
- rapid prototyping (speed > review)
- ปกติใช้ standard flow แล้วแทบไม่แก้ requirements/design อยู่แล้ว

**ใช้ standard Feature Specs (02/03) เมื่อ:**

- บุกเบิก territory ใหม่ / requirements หรือ design ต้อง iterate
- compliance-sensitive (review gates มีค่าจริง)
- ทีมได้ประโยชน์จาก explicit approval checkpoints

> ถ้า auto-generated requirements ตกหล่น: แก้ `requirements.md` ตรงๆ
> หรือสั่งในสเปกแชทให้เติม แล้วกด **Sync Files** regenerate tasks
> (artifacts ฟอร์แมตเดียวกับ standard เลยทำงานต่อด้วย workflow ไหนก็ได้ —
> ไม่มีปุ่ม switch โหมดอย่างเป็นทางการ แต่ไม่ต้องมีเพราะไฟล์เหมือนกัน)

---

## PART B — วิธีใช้ Plan Mode (คิดก่อนสั่งทำ)

### B.1 มันคืออะไร (ต่างจาก Specs ยังไง)

Plan mode = โหมด "คิดก่อนทำ" — ถามคำถาม + อ่านโค้ดแบบ read-only +
ออก implementation plan แล้วส่งต่อให้ execution agent ทำต่อ
**ไม่สร้างไฟล์ requirements/design/tasks** (ต่างจาก Specs ตรงนี้)

| | Plan mode | Feature Specs |
|---|---|---|
| **Output** | แผนในบทสนทนา + task breakdown | ไฟล์ `requirements.md`/`design.md`/`tasks.md` |
| **Process** | Q&A → plan → handoff | requirements → design → tasks + approval gates |
| **ความเร็ว** | นาที (Minutes) | นานกว่า (หลาย phases + reviews) |
| **เหมาะกับ** | งานกลางๆ แบบ explore-then-build | งานใหญ่, งานทีม, ต้องการ audit trail |
| **Artifacts** | อยู่ใน conversation context | เซฟใน `.kiro/specs/` |

ใช้ได้บน: **IDE ✓ / CLI ✓** (Web/Mobile ไม่มี)

### B.2 วิธีเรียกใช้ (2 ทาง)

1. จอ **"Let's build"** ตอนเริ่ม session → เลือก **Plan**
2. กลาง session → คลิก**ชื่อ agent** ใน bottom bar → เลือก **Plan**
   (ชื่อ agent ในช่อง input จะบอกว่า active ตัวไหนอยู่)

### B.3 สิ่งที่เกิดทีละขั้น (4 ขั้น)

**ขั้น 1 — Requirements gathering (ตอบคำถาม):**
planner ถาม structured questions เช่น

```
[plan] > I want to build a todo app
I understand you want to build a todo app. Let me help you plan this.
[1]: What platform should this target?
    a. Web Application  b. Mobile App  c. CLI Tool  d. Other
[2]: What's the primary use case?
    a. Personal Task Management  b. Team Collaboration ...
(Answer with "1=a, 2=b" or provide your own answers)
```

ตอบทีละข้อ / ตอบฟรีฟอร์ม / ข้ามข้อก็ได้

**ขั้น 2 — Research and analysis (Kiro อ่านโค้ด — คุณรอ):**
planner สำรวจ codebase แบบ **read-only** (อ่านไฟล์, code intelligence,
grep/glob, web search) แล้วปรับแผนให้เข้ากับ conventions ของคุณ

**ขั้น 3 — Implementation plan (อ่านแผน):**
ได้ step-by-step plan (problem statement + requirements + task breakdown
พร้อม demo criteria ต่อ task) → **อ่านแล้ว iterate จนตรงใจก่อน approve**

**ขั้น 4 — Handoff to execution (กด approve แล้วทำเลย):**

1. Plan mode จบ → agent สลับเป็น execution mode
2. แผนทั้งก้อนส่งเป็น context ให้ execution agent
3. เริ่ม implement ทันทีตามแผนที่ approve

> หมายเหตุ CLI: auto-execution ต้องใช้ **v3 engine** — classic mode
> จะถาม confirm handoff ก่อน ถ้าแผนไม่ส่งต่อให้ตรวจว่า approve ชัดเจนไหม

### B.4 สิ่งที่ Plan mode ทำได้/ไม่ได้ (read-only)

| Operation | ได้ | ไม่ได้ |
|---|---|---|
| อ่านไฟล์ / code intelligence | ✓ | |
| Search (grep, glob) / web search | ✓ | |
| เขียนไฟล์ / รันคำสั่ง / MCP tools | | — (ตั้งใจให้คิดอย่างเดียว) |

### B.5 Tips

- เหมาะกับงานที่ implement 15–60 นาที (นานพอให้ planning คุ้ม,
  สั้นเกินกว่าจะตั้ง formal specs)
- ตอบ structured questions ดีๆ — input ดีแผนดี
- ปล่อยให้ planner สำรวจ codebase (มันเจอ patterns ที่คุณมองข้าม)
- **อ่านแผนก่อน approve handoff ทุกครั้ง**

---

## PART C — วิธีใช้ Analyze Requirements (ตรวจ requirements ก่อนขึ้น design)

### C.1 มันคืออะไร (จับอะไรได้บ้าง)

สั่ง Kiro วิเคราะห์ requirements ทั้งชุด (reason ข้ามข้อ ไม่ใช่ทีละข้อ)
หาสิ่งที่อ่านผ่านๆ มองไม่เห็น:

- **Logical inconsistencies** — แต่ละข้อมีเหตุผล แต่รวมกันเป็นไปไม่ได้
- **Ambiguities** — คำกำกวม ("ไฟล์ใหญ่", "ตอบเร็ว") ที่ implement ได้หลายแบบ
- **Conflicting constraints** — functional vs non-functional ที่ทำพร้อมกันไม่ได้
- **Unstated assumptions** — อ้าง concept ที่ไม่ได้นิยาม
- **Missing edge cases** — failure modes, boundary, concurrent access ที่ happy path ไม่ครอบ

ใช้คุ้มเมื่อ: ฟีเจอร์ซับซ้อน requirements เยอะ, domain-sensitive
(financial/healthcare/compliance), Quick Spec ที่ requirements ถูก auto-generate
แบบไม่มีคน review — งานเล็ก/เข้าใจดีแล้วข้ามไป design ได้เลย

### C.2 วิธีกดเรียก (2 ที่ — หลัง requirements ถูก generate)

1. ใน **chat** — ตัวเลือก **Analyze Requirements** ข้างๆ ทางไปต่ออื่นๆ, หรือ
2. ใน **editor** — dropdown **Continue** → **Analyze Requirements**
   (อยู่ข้างๆ **Proceed to Design**)

ใช้ได้บน: **IDE ✓ / CLI ✓** (Web ไม่มี)

### C.3 สิ่งที่เกิด (เผื่อเวลา: หน่วยนาที ไม่ใช่วินาที)

- analysis หนักกว่าปกติ (cross-requirement reasoning) — **ใช้เวลาหลายนาที**
- findings พร้อมแล้ว clarifying questions จะ **stream เข้าแชททีละข้อ**
- แต่ละข้อมี: requirements ที่เกี่ยว + อธิบายปัญหาภาษาคน + **suggested fixes ให้กดเลือก**
- คุณทำได้ 3 อย่างต่อข้อ: **กดเลือก fix ที่เสนอ / พิมพ์คำตอบเอง / dismiss**
  (ถ้าความกำกวมนั้นตั้งใจ)
- ทุกครั้งที่ resolve ข้อหนึ่ง Kiro **อัปเดต `requirements.md` ใน editor ให้ทันที**
- แก้ requirements เองทีหลังได้ — **re-run analysis** ได้ Kiro จะวิเคราะห์
  requirements ที่กระทบ + interactions ใหม่
