# 06 — เขียน tasks.md + รัน tasks (Huuk edition)

> ขยายจาก https://kiro.dev/docs/specs/#three-phase-workflow (Tasks)
> และ https://kiro.dev/docs/specs/#task-execution (waves + parallel)

## 06.1 tasks.md ที่ดีหน้าตาอย่างไร

- **Discrete** — 1 task = 1 ผลลัพธ์ที่ verify ได้ (ไม่ใช่ "ทำ auth ให้เสร็จ")
- **Trackable** — checkbox + ID (`T-01…`) + อ้าง REQ-ID ที่มัน implement
- **Ordered by dependency** — task ที่ไม่มี dependency อยู่ wave แรก
- **Required vs optional** — แยกชัด (Kiro รันเฉพาะ required ที่ยังไม่เสร็จ)

รูปแบบต่อ task:

```markdown
- [ ] T-03 Implement POST /api/posts/{id}/lock handler (REQ-P02, REQ-P03)
      files: server/src/handlers.rs, server/openapi.yaml
      verify: cargo test --manifest-path server/Cargo.toml lock
```

## 06.2 Dependency waves (รันขนานอย่างปลอดภัย)

Kiro สร้าง **dependency graph** แล้วจัด tasks อิสระเป็น **waves**
(อ้างอิง https://kiro.dev/docs/specs/#running-tasks-in-parallel):

- **Wave 1** — tasks ที่ไม่มี dependencies → รันพร้อมกัน
- **Wave 2** — tasks ที่ dependencies ถูก Wave 1 เติมเต็ม → รันพร้อมกัน
- **Wave N** — จนครบ; waves วิ่งตามลำดับ, tasks ใน wave เดียวกันวิ่งขนาน

ตัวอย่าง Huuk (endpoint ใหม่ 1 ตัว):

```
Wave 1 (ขนานได้):  T-01 backend handler + openapi.yaml
                   T-02 mock/api.ts function
                   T-03 i18n keys + page skeleton
Wave 2 (รอ Wave 1): T-04 http.ts client (ต้องตรงกับ handler+mock)
                    T-05 queries.ts hooks + invalidateQueries
Wave 3 (รอ Wave 2): T-06 integration tests 2 ฝั่ง + typecheck/build
```

> ใน opencode workflow (ไม่มีปุ่ม Run all Tasks) ใช้ waves นี้จัดลำดับมือ:
> ทำงานใน wave เดียวกันสลับได้ แต่ห้ามข้าม wave

## 06.3 กฎแตก tasks สำหรับ Huuk (checklist ต่อ task)

ทุก task ที่แตะ endpoint ต้องเช็ก **4-place rule**:

- [ ] `server/src/handlers.rs` — handler + validation + `require_perm`
- [ ] `app/src/api/http.ts` **และ** `app/src/mock/api.ts` — ชื่อ/signature ตรงกัน
- [ ] `server/openapi.yaml` — path + schema + `x-permission` + error codes
- [ ] tests 2 ฝั่ง — `server/tests/api.rs` + `app/src/tests/*.test.ts`

ทุก task ที่แตะ frontend ต้องเช็ก:

- [ ] screens ลง `core/screens.ts` ที่เดียว (หรือ `meta.public`/`standalone` ถ้านอก deck)
- [ ] mutation ใหม่ `invalidateQueries` ทุก affected key
- [ ] styling ตามกฎ (09): rounded, border color, button width, no `<style>`,
      no hardcode hex, `text-*` คู่ `leading-*`
- [ ] import ผ่าน `@` alias; components ใหม่ vendored ใน `app/src/components/`
- [ ] TanStack Table v9 API (ถ้าแตะ tables)
- [ ] tests query ด้วย role/text/tag

## 06.4 Verification gates (บังคับทุก spec)

```sh
bun run build        # cargo build --release + vue-tsc -b && vite build
bun run test         # cargo test (server) + vitest run (app)
```

- Focused ระหว่างทาง:
  `cargo test --manifest-path server/Cargo.toml <filter>` /
  `cd app && bun run test -- <file-or-substring>` /
  `cd app && bun run typecheck`
- **type errors โผล่แค่ใน `vue-tsc -b`** (vitest ไม่รัน) — ห้ามข้าม `build`
- Task ถือว่าเสร็จเมื่อ gates ที่เกี่ยวข้องเขียว ไม่ใช่แค่ "เขียนโค้ดเสร็จ"

## 06.5 Sync Files (เมื่อ requirements/design เปลี่ยนกลางทาง)

(จาก https://kiro.dev/docs/specs/best-practices/#how-do-i-iterate-on-my-feature-specs)

**Requirements-First:**

1. แก้ `requirements.md` (ตรงๆ หรือสั่งในสเปกแชท)
2. สั่งอัปเดต `design.md` ให้ตรง requirements ใหม่
3. เปิด `tasks.md` → **Sync Files** → tasks ใหม่ map กับ requirements ใหม่

**Design-First:**

1. แก้ `design.md`
2. สั่ง validate/regenerate requirements ให้ feasible กับ arch ใหม่
3. **Sync Files** ใน `tasks.md`

**Tasks ที่ implement ไปแล้ว:** เปิด `tasks.md` → **Sync Files**
(หรือสั่ง "check which tasks are already complete" ให้ scan codebase)
แล้ว mark completed อัตโนมัติ — ดู 07/09
