# 03 — วิธีใช้ Design-First (Walkthrough ทีละจอ)

> เริ่มจากเทคนิค/สถาปัตยกรรม เรียบเรียงจาก
> https://kiro.dev/docs/specs/feature-specs/tech-design-first/
> ใช้เมื่อในหัวคุณชัดเรื่อง "วิธีสร้าง" มากกว่า "พฤติกรรมที่อยากได้"

## ใช้เมื่อไหร่ (เช็ก 1 ใน 5 ข้อนี้)

- รู้ tech stack แล้ว อยาก **prototype เร็วๆ** ดูว่าทำอะไรได้บ้าง
- ระบบมี **constraints เข้ม** (latency, throughput, uptime, compliance)
- มี **design อยู่แล้ว** (diagrams/docs จากเครื่องมืออื่น) จะ port เข้า Kiro
- อยากสำรวจ **feasibility** ก่อน commit scope
- มี **architectural opinions** ชัดเจน (อยากได้อย่างนี้เท่านั้น)

## ขั้น 1 — สร้าง Feature Spec (เหมือน 02 แต่เลือกอีกอัน)

1. Kiro panel → **Specs** → **`+`** (หรือ chat → **Spec**)
2. เลือก **Feature**
3. เลือก **Design-First** ← ต่างจาก 02 แค่ขั้นนี้
4. พิมพ์ constraints ทางเทคนิคให้ชัดตั้งแต่ประโยคแรก ตัวอย่างที่ดี:

```
Create an MCP server for querying our Redshift data lake. Must use FastAPI
and mcp-python. Target: <100ms query latency for 95th percentile.
No UI components needed - pure API server.
```

ยิ่งระบุเลข/ชื่อเฉพาะ (technologies, latency, services, integration points)
design ที่ได้ยิ่งไม่ generic

## ขั้น 2 — เลือกระดับ detail (จอที่ Kiro ถามต่อ)

Kiro จะให้เลือก 1 ใน 2 ระดับ:

**High Level Design** — สถาปัตยกรรมเต็ม:

- architecture diagram + คำอธิบาย components + interactions
- technical approach + patterns + non-functional properties
- เหมาะกับ: ระบบซับซ้อน, งานทีม (เอกสาร + shared understanding)

**Low Level Design** — ลง implementation:

- pseudocode ของ algorithm + interface definitions/contracts
- key data structures + non-functional properties
- เหมาะกับ: prototype เร็ว, feasibility check, solo dev

> งานหลายคน implement → High Level / เช็คไอเดียเร็วๆ → Low Level

## ขั้น 3 — Design Phase (จอแรกที่คุณต้องตรวจ)

**สิ่งที่ Kiro ทำให้:** สร้างไฟล์ `design.md` (components + responsibilities,
data flow, tech stack, API contracts, non-functional considerations)
*Tip: design เปิดเป็น preview mode อ่านง่าย — กด **Edit** เมื่อจะแก้*

**สิ่งที่คุณต้องทำ:**

1. อ่าน architecture — ถูกต้องไหม ลองสั่ง *"explore alternative with …"*
   ให้ Kiro เสนอทางเลือก + trade-offs (latency vs cost, complexity vs maintainability)
2. ชั่ง non-functional ว่าทำได้จริงไหม
3. ถูกต้องแล้ว → **ยืนยัน** เพื่อไปขั้น 4

**เอาของเดิมเข้า:** มี diagrams จาก draw.io/Lucidchart/ภาพ whiteboard →
export PNG / ถ่ายรูปแนบใน prompt ตั้งต้น Kiro จะรวมเข้า architecture ให้

## ขั้น 4 — Requirements Phase (Kiro derive ให้ — รับประกัน feasible)

**สิ่งที่ Kiro ทำให้:** derive requirements จาก architecture ที่คุณยืนยันแล้ว
(user stories + EARS + edge cases) — **ข้อดี: requirements ชุดนี้ feasible
แน่นอนเพราะเกิดจาก validated design** ไม่ต้องมา reconcile ทีหลัง

**สิ่งที่คุณต้องทำ:**

1. อ่าน — ครบพฤติกรรมฝั่ง user ไหม
2. เติม user-facing behaviors ที่หายไป (สั่งในแชทหรือแก้ไฟล์ตรง)
3. พอใจแล้ว → **ยืนยัน** ไป tasks

## ขั้น 5–6 — Tasks + Implementation

เหมือน Requirements-First ทุกประการ (ดู 02 ขั้น 4–5 + วิธีรันที่ 01 §01.6)

## ปัญหาที่เจอบ่อย + วิธีกดแก้

| อาการ | วิธีกดแก้ |
|---|---|
| Requirements ไม่ตรงที่คาด | กลับไป iterate architecture (เติม intended capabilities + non-functional) → regenerate requirements |
| Architecture generic เกิน | เติม technical details (frameworks/libs/services ตัวจริง + ตัวเลข) หรือสลับเป็น High Level |
| ต้องเปลี่ยน design หลังมี requirements | แก้ `design.md` → สั่ง *"validate requirements against updated architecture"* → regenerate ถ้าจำเป็น |

## ตัวอย่าง prompt พร้อมใช้ (2 ระดับ)

**High Level:**

```
Design a real-time notification system using AWS services. Must support:
- 100k concurrent WebSocket connections
- <50ms message delivery latency
- Message persistence for offline users
- Integration with existing user authentication service
Use API Gateway WebSocket, Lambda, DynamoDB, and SQS. Include
architecture diagram showing component interactions and data flow.
```

**Low Level:**

```
Build a rate limiter middleware for our Express API. Requirements:
- Token bucket algorithm
- Per-user and per-IP limits
- Redis for distributed state
- Configurable limits per endpoint
Show pseudocode for the core algorithm and interface definitions.
```

## ทำไมต้อง Design-First (5 ข้อ)

1. **Feasible requirements** — เกิดจาก validated design
2. **ไม่ต้อง reconcile** — ไม่แก้ requirements ตอนสำรวจ designs
3. **ประหยัดเวลา/credits** — iterations น้อยลง
4. **สำรวจ technical options** ก่อน commit
5. **Validate constraints เร็ว** — non-functional ชัวร์ตั้งแต่ต้น
