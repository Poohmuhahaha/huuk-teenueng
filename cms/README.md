# Huuk CMS — Strapi HeadlessCMS for Docs

Strapi 5 + Postgres ถือ content ของ Docs (`Doc Page`: title/slug/section/order/summary/body markdown)
เว็บ Docs (`../docs-web/`) อ่านจาก API สด — แก้ content ใน admin แล้วเว็บเปลี่ยนทันที ไม่ต้อง build ใหม่

## Run (local)

```sh
cd cms
docker compose up --build    # Strapi :1337 + Postgres :5432 (seed 20 หน้ารอบแรกอัตโนมัติ)
```

1. เปิด http://localhost:1337/admin → สร้าง admin account (ครั้งแรกครั้งเดียว)
2. เมนู **Content Manager → Doc Page** = เนื้อหา Docs ทั้งหมด (แก้/เพิ่ม/ลบ + Publish)
3. ตรวจ API: `curl http://localhost:1337/api/doc-pages | head -c 300`
4. เปิดเว็บ Docs: `cd ../docs-web && bun install && bun run dev` → http://localhost:5175/

## Content model (Doc Page)

| Field | Type | หมายเหตุ |
|---|---|---|
| `title` | string (required) | ชื่อหน้า |
| `slug` | uid → title (required, unique) | ใช้เป็น URL (`/:slug`), หน้าแรก = `index` |
| `section` | enum: start/workflows/authoring/reference/templates/examples | กลุ่ม sidebar |
| `sectionLabel` | string | ป้ายกลุ่มภาษาไทย |
| `order` | integer | ลำดับใน sidebar |
| `summary` | text | คำโปรย (การ์ด + search) |
| `body` | text | markdown เต็ม — docs-web render ด้วย `marked` |

`draftAndPublish: true` — API ส่งเฉพาะหน้าที่ **Published** (ร่างไม่หลุดไปหน้าเว็บ)

## Seed (markdown → Strapi)

- `docs/*.md` = seed source (ยังอยู่ใน git) → `node scripts/export-docs.mjs` → `seed/docs.json` (20 หน้า)
- bootstrap (`src/index.ts`) **ensure ทุก bootแบบ idempotent**: slug ไหน admin
  เป็นเจ้าของแล้ว (draft/published) จะไม่ถูก overwrite; draft seed ที่ค้างจะถูก publish
- v5 notes (กันงงรอบหน้า): publishing = top-level `status: 'published'`
  (ไม่ใช่ field `publishedAt`); ไม่มีคอลัมน์ `enabled` ใน permissions —
  เปิด public `find`/`findOne` ผ่าน `users-permissions` role service
  (`findOne` + `updateRole`, กลไกเดียวกับปุ่ม save ใน admin)

## Env & secrets

- `docker-compose.yml` มี defaults สำหรับ **local dev เท่านั้น**
- staging/production: copy `.env.example` → `.env` ใส่ secrets จริง
  (`openssl rand -base64 32`, APP_KEYS 4 ค่า comma-separated)
- Postgres data อยู่ใน volume `pgdata` (`docker compose down -v` = ล้างหมด + seed ใหม่รอบหน้า)

## Files

```
cms/
├── src/api/doc-page/   # content-type + controller/service/routes (CRUD)
├── src/index.ts        # bootstrap: seed + public permissions
├── config/database.ts  # Postgres via env (default scaffold, ไม่ได้แก้)
├── seed/docs.json      # seed 20 หน้า (generate จาก docs/*.md)
├── scripts/export-docs.mjs
├── Dockerfile / docker-compose.yml / .env.example
```
