# STATUS — content-planner-huuk (Huuk by teenueng)

- Category: Webapp | Priority: **P0-FLAGSHIP** (~90% แล้ว)
- Layout: `app/` = Vue 3 frontend, `server/` = Rust (Axum) backend (ย้ายจาก `apps/production`, `apps/server` 2026-09-27)
- Run: `bun run dev` (รูท) → backend `:8787` + frontend `:5173`
- Docs: `Docs/01_Dev` → `02_Design` → `03_Component` → `04_API` → `05_Tech-Stack` → `06_Business` (อ่านโค้ดจริง 2026-09-27)

## Next 3
1. `./deploy.sh init + up + smoke` ซ้อม staging แล้ว rotate รหัส bootstrap
2. ต่อ OAuth จริง 1 เจ้า (Meta/Google/TikTok) end-to-end + ตรวจ `/#/profile?oauth`
3. เปิด backup systemd รายวัน + ทดสอบ restore นอกเครื่อง

## DoD
Client ล็อกอิน + สร้าง/ส่งบทความผ่าน `/#/studio` ได้บน `huuk.teenueng.com`
