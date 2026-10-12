# 09 — Best Practices ฉบับ Huuk (กฎ + gotchas + troubleshooting)

> รวม Kiro best-practices FAQs (https://kiro.dev/docs/specs/best-practices/)
> ที่ปรับแล้ว + กฎเฉพาะ repo นี้จาก `AGENTS.md` — **อ่านก่อน implement ทุกครั้ง**

## 09.1 เลือก workflow อย่างไร (FAQ)

- **Requirements-First** — รู้ behavior ชัด / architecture ยืดหยุ่น /
  product-led / ไม่มี technical constraints → ดู 02
- **Design-First** — มี design อยู่แล้ว / non-functional เข้ม /
  prototype ด้วย stack เฉพาะ / สำรวจ feasibility ก่อน → ดู 03
- เริ่ม spec แล้ว**เปลี่ยน workflow ไม่ได้** — ต้องสร้าง spec ใหม่
  (copy เนื้อหาที่ใช้ได้มา) — ดังนั้นเลือกให้ถูกตั้งแต่ intent
- ใช้ **ทั้งสอง workflows ในโปรเจกต์เดียวได้** (คนละ specs):
  เช่น Design-First validate feasibility ก่อน แล้ว Requirements-First
  สำหรับ full feature development

## 09.2 Import ของเดิมเข้า spec อย่างไร (FAQ)

- **มี diagrams/docs อยู่แล้ว** → Design-First + upload PNG/JPG (draw.io,
  Lucidchart, ภาพ whiteboard) ใน intent ตั้งต้น → formalize ลง `design.md` →
  derive requirements
- **requirements อยู่ใน JIRA/Confluence/Word** → ถ้า tool มี MCP server
  ก็ connect ตรง; ไม่งั้น copy ไฟล์เข้า repo แล้วสั่ง
  `#foo-prfaq.md Generate a spec from it`
- **คุย vibe มาก่อน** → พิมพ์ `Generate spec` แล้วตอบ yes เพื่อเริ่ม spec session
  จาก context เดิม

## 09.3 Iterate / share specs อย่างไร (FAQ)

- **Iterate:** Requirements-First = แก้ requirements → สั่งอัปเดต design →
  Sync Files ใน tasks; Design-First = แก้ design → validate/regenerate
  requirements → Sync Files (ดู 06 §06.5)
- **Share กับทีม:** specs อยู่ใน repo เดียวกับโค้ด (version-controlled) —
  แนะนำแยก specs ต่อ feature (`.kiro/specs/user-auth/`, `product-catalog/`, …)
  ทำงานอิสระ ไม่ตีกัน, review โฟกัส, ทำขนานกันได้
- **อ้าง spec ในแชท:** `#spec:<name>` (Kiro ดึง requirements+design+tasks
  เข้า context) — เช่น `#spec:user-authentication implement task 2.3`
- **Tasks ที่เสร็จแล้ว (ใครทำไปก่อน):** Sync Files ใน `tasks.md` หรือสั่ง
  "check which tasks are already complete" ให้ scan codebase
- **Run tasks:** ทีละ task หรือทั้งหมด (Kiro จัด waves ขนานให้ — ดู 06 §06.2);
  รันเฉพาะ required ที่ยังไม่เสร็จ

## 09.4 กฎ Huuk ที่ spec ทุกฉบับต้องเคารพ

### Data flow (4-place rule)

Endpoint ใหม่/เปลี่ยน = 4 ที่ใน change เดียว:
`handlers.rs` → `http.ts` + `mock/api.ts` → `openapi.yaml` → tests 2 ฝั่ง
(รายละเอียดดู 00 §00.3)

### Frontend

- Screens ลง `core/screens.ts` ที่เดียว; มีแค่ `PlannerPage` เขียน posts;
  computed edits ผ่าน `ProtectedModal`; deck navigation (URL ตาม slide);
  import ผ่าน `@` alias
- Components vendored (`app/src/components/` + barrel `ui/index.ts`);
  ห้าม import จาก Pugu kit — port ไฟล์มาแทน
- Tailwind v4: สี `var(--*)`, type scale `text-2xs–text-5xl` คู่ `leading-*`,
  radii เฉพาะ `rounded-none`/`rounded-full`, border ต้องมีสี,
  ทุก `<button>` มี `w-full`/`w-auto`, input เปล่าไม่ต้องมี classes,
  ห้าม `<style>` ยกเว้น `@keyframes`/`@container`
- TanStack Table **v9**: `useTable` + `tableFeatures` + `create*RowModel`;
  column type `ColumnDef<typeof features, Row, unknown>[]`;
  row shapes เป็น `type` ไม่ใช่ `interface`; ส่ง `computed` ref ทั้งก้อน (ไม่ใช่ `.value`);
  numeric columns sort desc-first ยกเว้น `sortFn: 'basic'`
- Animation: animate `width`/`left` จริงบน pinned card ห้าม `scaleX`/`clip-path`;
  guard re-entry (`fullBusy` / `fullAnim?.stop()`); deck drag จับ pointer หลัง >8px,
  disable snap/smooth กลาง drag
- Mutation ใหม่ต้อง `invalidateQueries` ทุก affected key
- Tests query ด้วย role/text/tag — ห้าม query ด้วย styling classes
- Provider login pages (`PlatformLogin`) มี chrome ของตัวเอง ห้าม themed

### Backend

- JSON camelCase; errors `{ "error": "…" }`; validation ตาม `server/README.md`
- Locks จาก session เมื่อ auth on; create ไม่รับ `lockedBy`
- CMS `version` ทุก write; stale → 409; scheduled flip แบบ lazy
- Snapshots atomic; `data/` `backups/` `*.env.production` ห้าม commit
- `dev.sh` ต้องมี bun; port ผ่าน `PORT`/`BIND_ADDR`

## 09.5 Gotchas (ตารางกับดัก — เติมได้เรื่อยๆ)

| อาการ | สาเหตุ | แก้ |
|---|---|---|
| write สำเร็จแต่ UI ไม่ refresh | ลืม `invalidateQueries` key ที่เกี่ยว | เติมทุก affected key ใน `onSuccess` |
| UI ค้างหลังแก้ frontend (`:8080`) | ดู built SPA ของ deploy stack | `cd app && bun run build` + redeploy/restart |
| `.env.local` ไม่生效 | Vite อ่าน env ตอน boot | restart Vite |
| type error โผล่ตอน build ทั้งที่ test เขียว | vitest ไม่รัน `vue-tsc` | `cd app && bun run typecheck` / `bun run build` เสมอ |
| `useVueTable`/`getCoreRowModel` ไม่มี | จำ v8 มา — repo นี้คือ v9 | `useTable` + `tableFeatures` + `create*RowModel` |
| import component จาก `../../components/huuk` | นั่นของ Pugu ไม่ใช่ repo นี้ | port ไฟล์เข้า `app/src/components/` |
| 401/403 ทั้งที่ demo ใช้ได้ | production mode บังคับ token+permission | เช็ก `authRequired` + Bearer + role permissions |
| 409 แปลกๆ ตอนแก้ post | ติด lock ของคนอื่น / session holder ไม่ตรง | เช็ก holder + ใช้ session account |

## 09.6 Troubleshooting (spec-level)

- **Design ไม่ตอบ requirements** → requirements ชัดไหม → เติม acceptance →
  regenerate design (ดู 02 §02.5)
- **Requirements กำกวม** → เติม scenarios/edge/success criteria + เขียน EARS (ดู 04)
- **Requirements ไม่ตรง expectation (Design-First)** → กลับไป iterate arch (ดู 03 §03.5)
- **Tasks บางข้อเสร็จไปแล้ว** → Sync Files / scan codebase (ดู 06 §06.5 / 09 §09.3)
