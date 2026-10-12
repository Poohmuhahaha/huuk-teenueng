# 00 — ภาพรวม Huuk (Project Overview)

> อ่านไฟล์นี้ก่อนเขียน spec ใดๆ — มันคือ "สนาม" ที่ทุก requirements/design/tasks
> ต้องลงไปวิ่งจริง อ้างอิง `AGENTS.md`, `README.md`, `app/README.md`, `server/README.md`

## 00.1 Huuk คืออะไร (What it is)

**Huuk by teenueng** = content planner สำหรับ studio + client:

- วางแผนคอนเทนต์รายเดือน (topics, calendar, ideas backlog, hashtags)
- เขียน/รีวิวคอนเทนต์ใน **Content Studio** (CMS: articles/pages/notes + Markdown +
  revision history + scheduled publishing + public reader `/#/read/{slug}`)
- โปรโมท (campaigns, Meta Ads mirror + management) และวัดผล
  (performance metrics, finance)
- Onboard client ด้วย role **Client** + สร้าง sign-in account ให้จาก Settings
  (temporary password แสดงครั้งเดียว)

สถานะ (ณ วันที่เขียน docs): user stories US-001…US-014 ครบ, Rust tests 94 +
frontend tests 188+ เขียว

## 00.2 Stack (The two sides)

| ฝั่ง | เทคโนโลยี | โฟลเดอร์ | entry |
|-----|-----------|----------|-------|
| Frontend | Vue 3 + TanStack Query + Tailwind v4 + Vite | `app/` | `app/src/app/main.ts`, `App.vue`, `router/index.ts` |
| Backend | Rust + Axum 0.8 + Tokio, in-memory `Store` + atomic JSON snapshots | `server/` | `server/src/main.rs` (config/loop/shutdown), `lib.rs` (router + `AppState`), `store.rs`, `model.rs`, `handlers.rs`, `error.rs`, `oauth.rs`, `live.rs`, `ads.rs`, `secrets.rs` |
| Ops | compose env, Caddy, systemd, scripts | `ops/` | `ops/scripts/{dev,deploy,install,backup,restore,smoke}.sh`, `Makefile` (thin front door) |

ไม่มี CI / eslint / prettier — **verification gates** คือ `vue-tsc` (ผ่าน build)
กับ test 2 suites (ดู §00.7)

## 00.3 Data-source swap (จุดสลับ mock ↔ backend)

หัวใจของ data flow อยู่ใน `app/src/api/index.ts:1-21`:

```
VITE_API_URL unset (+ DEV)  →  @/mock/api  (in-memory, รีเซ็ตเมื่อ reload)
VITE_API_URL set            →  @/api/http   →  Rust API
```

กฎเหล็ก (spec ใดแตะ data flow ต้องเขียนข้อนี้ลง requirements):

1. Pages/components **ห้าม import `@/mock/` ตรงๆ** — ผ่าน `@/api` เท่านั้น
2. Endpoint ใหม่/เปลี่ยน = **4 ที่ใน change เดียว** (4-place rule):
   `server/src/handlers.rs` → `app/src/api/http.ts` **และ** `app/src/mock/api.ts`
   → `server/openapi.yaml` (source of truth, serve ที่ `/api/openapi.yaml` +
   Swagger UI ที่ `/api/docs`, `tests/api.rs` assert ว่า serve จริง) → tests 2 ฝั่ง
3. JSON เป็น **camelCase**; errors เป็น `{ "error": "…" }` เสมอ
4. เติม/เปลี่ยน `.env.local` (`VITE_API_URL=http://localhost:8787`) แล้วต้อง **restart Vite**
5. UI ค้างบน `:8080` คือเรื่องปกติ — นั่นคือ **built SPA** จาก deploy stack
   ไม่ใช่ dev server; หลังแก้ frontend ต้อง `cd app && bun run build` แล้ว redeploy/restart

## 00.4 Navigation = slide deck (ไม่ใช่ page swaps)

- Screens ลงทะเบียน **ที่เดียว** ใน `app/src/core/screens.ts:27` (ลำดับใน array = ลำดับ deck)
  ปัจจุบันมี 6 hubs: `/settings` (Brand Identity — การ์ดหน้าสุด + landing ที่ `/`),
  `/dashboard` (Home), `/plan`, `/content`, `/promote`, `/analyze`
- ฟีเจอร์ย่อยเป็น **tabs ใน hub** (metadata ที่ `@/core/subnav.ts` — `HUB_TABS`)
  ไม่ใช่การ์ดแยก; legacy paths เก่า redirect เข้า hub+tab (ดู `router/index.ts:16-47`)
- **URL ตาม slide** — spec ที่เพิ่มหน้าย่อยต้องระบุว่าไปอยู่ hub ไหน tab ไหน
  (ข้อยกเว้น: หน้า public/standalone นอก deck — ดู §00.5)
- Queries/mutations ใหม่ใน `core/queries.ts` ต้อง `invalidateQueries` ทุก affected key
  ไม่งั้น write สำเร็จแต่ UI ไม่ refresh

## 00.5 หน้าประเภทพิเศษ (Public / standalone)

`app/src/app/App.vue:46-60` แยก 3 โหมด:

| ประเภท | ตัวอย่าง | meta | render |
|--------|---------|------|--------|
| Public | `/login`, `/register`, `/plans`, **`/landing`** (ใหม่) | `meta.public` | `RouterView` ตรง (ไม่มี AppShell) + login users โดนกันออก (`router/index.ts:73-76`) |
| Standalone | `/read/:slug`, `/welcome`, `/components` | `meta.standalone` | ใน AppShell แต่ข้าม `ScreensDeck` |
| Deck | 6 hubs | — | `ScreensDeck` |

**Landing Page** (`app/src/pages/LandingPage.vue`, route `/landing`, `meta.public`)
เป็นตัวอย่างล่าสุดของ public page: static (ไม่มี queries), CTA เข้า
`/register` `/login` `/plans` `/`, sections Hero + 6 hubs + Pricing + Demo
ดู spec เต็มของมันที่ [examples/landing-page/requirements.md](examples/landing-page/requirements)

## 00.6 Backend notes (ที่ spec ต้องรู้)

- State: `Arc<RwLock<Store>>` (`AppState`) — read lock สำหรับ queries,
  write lock สำหรับ mutations; single instance
- Snapshots ลง `DATA_FILE` แบบ atomic ทุก 5 วิเมื่อ dirty + ตอน SIGTERM/SIGINT;
  `data/`, `backups/`, `ops/deploy/.env.production`, `*.prompt.md`, `*.pdf`/`*.typ`
  เป็น runtime/secrets (git-ignored) — **ห้าม commit**
- Auth: Argon2id, session token 64-hex (32 bytes `rand`), TTL 30 วัน, in-memory
  (restart แล้วหาย); demo logins `owner@studio.local` / `editor@studio.local` /
  `client@studio.local` รหัส `demo1234`; token อยู่ใน `localStorage` (`cp.token`)
- `authRequired` on → post locks ดึง holder จาก **session** (client-sent
  `user`/`lockedBy` ถูก ignore; create ไม่รับ `lockedBy`)
- Validation สำคัญ: `400` (payload ผิด), `401` (token), `403` (ขาด permission —
  13 permissions ดู `server/README.md` § Permissions), `409` (locked by another /
  email taken / name taken / stale CMS `version`), `404` (unknown id)
- CMS concurrency: ทุก update/publish/schedule/restore ส่ง `version` ที่อ่านมา;
  stale → `409`; scheduled items flip เป็น published แบบ lazy เมื่อมี content read
  หลัง `scheduledFor` (ไม่ต้องมี cron)
- OAuth 2.0 แบบ credential-optional (Meta/Google/TikTok): ไม่มี credentials →
  `mock` mode (simulation ยังใช้ได้); ใส่ credentials → `redirect` mode
  (provider domain จริง, `state` single-use 10 นาที)
- Ports: backend `:8787` (`PORT`/`BIND_ADDR` override); frontend Vite default 5173
- Binary serve SPA เองได้ (`STATIC_DIR=app/dist`); `dev.sh` **ต้องมี bun**
  (`command -v bun` ไม่งั้น exit) และ reuse backend ที่รันอยู่แล้วบน port นั้น

## 00.7 Run / verify (คำสั่งที่ spec ต้องอ้าง)

```sh
bun run dev          # backend :8787 (DEMO_MODE=1) + Vite (VITE_API_URL wired); Ctrl+C หยุดทั้งคู่
bun run dev:mock     # frontend อย่างเดียว (in-memory mock, ไม่ต้องมี Rust)
bun run dev-server   # backend อย่างเดียว
bun run test         # cargo test (server) + vitest run (app)
bun run build        # cargo build --release + vue-tsc -b && vite build
```

- Focused: `cargo test --manifest-path server/Cargo.toml <filter>` (substring;
  suite `server/tests/api.rs` boot router จริงผ่าน `tower::oneshot`, ไม่ใช้ network);
  `cd app && bun run test -- <file-or-substring>` (vitest filter);
  typecheck อย่างเดียว: `cd app && bun run typecheck`
- **ก่อนเรียกงานเสร็จ: `bun run build` และ `bun run test` ทั้งคู่**
  (type errors โผล่แค่ใน `vue-tsc -b` ซึ่ง vitest ไม่รัน)

## 00.8 Frontend rules (ฉบับย่อ — ฉบับเต็มดู 09)

Non-negotiables จาก `app/README.md`:

1. One sheet, one screen (ลงทะเบียนใน `screens.ts` เท่านั้น)
2. มีแค่ `PlannerPage` ที่เขียน posts — view อื่นเป็น read projection
3. แก้ computed field ผ่าน `ProtectedModal` (Cancel เป็น default)
4. Navigation เป็น slide deck, URL ตาม slide
5. Import ผ่าน `@` alias (`@/*` → `src/*`) ห้าม `../../` ยาวๆ
6. Components vendored: `app/src/components/` คือ source of truth
   (barrel `components/ui/index.ts`); ห้าม import จาก sibling Pugu kit
   (`../../components/huuk`) — ให้ port ไฟล์มาแทน
7. Styling Tailwind v4: สีเป็น `var(--*)` aliases (runtime brand theming ผ่าน
   `core/theme.ts` → `:root` ยังทำงาน), type scale `text-2xs(11)–text-5xl(26)`
   คู่ `leading-*`, radii เฉพาะ `rounded-none`/`rounded-full`, border ต้องมีสี
   (`border border-line`), ทุก `<button>` มี `w-full`/`w-auto` ชัดเจน,
   input/select/textarea เปล่าไม่ต้องมี classes (มี base skin แล้ว),
   ห้าม `<style>` ใน components ยกเว้น `@keyframes`/`@container`
8. TanStack Table **v9** (`useTable` + `tableFeatures` + `create*RowModel`;
   `useVueTable`/`getCoreRowModel` ไม่มีอยู่จริง)
9. Animation: animate `width`/`left` จริงบน pinned card (`position:fixed`)
   ห้าม `scaleX`/`clip-path`; guard re-entry; deck drag จับ pointer หลังขยับ >8px
10. Tests query ด้วย role/text/tag — ห้าม query ด้วย styling classes

→ รายละเอียด + ตัวอย่างผิด/ถูก ดู [09-best-practices-huuk.md](09-best-practices-huuk.md)
