# 11 — Hands-on Walkthrough (ตัวอย่าง session จริงตั้งแต่ต้นจนจบ)

> อ่าน 01–02 มาก่อน ไฟล์นี้คือภาพจำลอง 1 session เต็ม:
> **คุณพิมพ์อะไร → Kiro ตอบ/แสดงอะไร → คุณกดอะไรต่อ**
> ตัวอย่าง: ฟีเจอร์ "scheduled publishing" ของ Content Studio
> (ใช้ Requirements-First บน Kiro IDE)

## ฉากที่ 1 — เริ่ม session (คุณพิมพ์ 2 บรรทัด)

คุณกด **Specs → `+`** → เลือก **Feature** → เลือก **Requirements-First**
แล้วพิมพ์:

```
I want scheduled publishing for Content Studio articles. Authors pick a
future date/time, the item auto-publishes when time comes, and clients
see only published items in the reader.
```

Kiro ตอบกลับด้วย `requirements.md` ฉบับร่าง (user stories + EARS + edge cases)
เช่น:

```markdown
WHEN an author schedules an article with a future scheduledFor
THE SYSTEM SHALL keep it invisible in the public reader until that time
WHEN any content read happens after an item's scheduledFor
THE SYSTEM SHALL flip it to published (lazy, no cron required)
```

## ฉากที่ 2 — คุณตรวจ requirements (อย่ากดผ่าน)

คุณอ่านแล้วเจอว่าขาดเคส "ยกเลิก schedule" → พิมพ์สั่งตรงๆ:

```
Add a requirement: authors can unschedule back to draft, and clients
must never see scheduled (unpublished) items even with direct slug URL.
```

Kiro แก้ `requirements.md` ให้ → คุณเห็นว่าข้อนี้ตรงกับ behavior จริงของ Huuk
(scheduled flip แบบ lazy + public delivery ให้เฉพาะ published — ดู 00 §00.6)
→ คุณกด **Analyze Requirements** (ในแชท หรือ Continue dropdown)

Kiro ใช้เวลาหลายนาทีแล้ว stream คำถามมา เช่น:

> *REQ-2 vs REQ-5 ขัดกันไหม: "flip on any content read" รวม public read
> ด้วยหรือไม่? — [a] รวม (public read ก็ trigger ได้) / [b] เฉพาะ authed reads
> / พิมพ์เอง / dismiss*

คุณกดเลือก → Kiro อัปเดต `requirements.md` ทันที → คุณกด **Proceed to Design**

## ฉากที่ 3 — คุณตรวจ design

Kiro ร่าง `design.md` (components + sequence + `version` concurrency + tests)
คุณติเรื่องเดียว:

```
The schedule write must also send the version we last read, and a stale
version must return 409 like other content writes.
```

Kiro อัปเดต design → คุณกดยืนยัน → Kiro แตก `tasks.md` (waves 1–3)

## ฉากที่ 4 — คุณรัน tasks

- กด **Run all Tasks** → Wave 1 (handler + mock + page skeleton) วิ่งขนาน →
  Wave 2 (http client + queries hooks + invalidateQueries) → Wave 3 (tests 2 ฝั่ง)
- สถานะอัปเดต real-time ใน `tasks.md`
- มี task นึงเพื่อนทำไปแล้ว → กด **Sync Files** Kiro ติ๊กให้อัตโนมัติ

## ฉากที่ 5 — คุณใช้ spec ต่อหลัง implement

```text
#spec:scheduled-publishing does my current implementation meet the
acceptance criteria for task 3.2?
```

Kiro อ่าน 3 ไฟล์ใน context แล้วตอบพร้อมชี้จุดที่ขาด → คุณสั่งแก้ →
ปิดงานด้วย verification gates ของ Huuk (`bun run build` + `bun run test` — ดู 06)

## สิ่งที่ walkthrough นี้สอน (5 บทเรียน)

1. ประโยคตั้งต้นดี → requirements ดี (ตอบ who/what/why/success ตั้งแต่ขั้น 0)
2. อย่ากดผ่าน requirements — สั่งแก้ด้วยภาษาคนได้เลย
3. Analyze จับข้อขัดแย้งที่คุณอ่านผ่านๆ มองไม่เห็น (เผื่อเวลาหลายนาที)
4. Design ติได้ทีละจุด (เช่น concurrency `version` + 409)
5. `#spec` ทำให้ spec มีชีวิตหลัง implement (ตรวจงาน/ถามเหตุผล/สั่งแก้)
