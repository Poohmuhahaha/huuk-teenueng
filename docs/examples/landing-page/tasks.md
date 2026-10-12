# tasks.md — EXAMPLE: Landing Page (`/landing`)

> implement จริงแล้วทั้งหมด ✅ — เก็บไว้เป็น reference ว่า waves เล็กหน้าตาอย่างไร

## Wave 1 (no dependencies — ขนานได้)

- [x] T-01 `LandingPage.vue` static sections (REQ-L02…L05)
      files: `app/src/pages/LandingPage.vue`
- [x] T-02 `landing.*` i18n keys ~30 (REQ-L07)
      files: `app/src/core/i18n.ts`
- [x] T-03 route `/landing` meta.public (REQ-L01)
      files: `app/src/app/router/index.ts`

## Wave 2 (รอ Wave 1)

- [x] T-04 signed-in header branch (REQ-L06) — รวมในไฟล์เดียวกับ T-01
- [x] T-05 `landing.test.ts` 4 tests (REQ-L08)
      files: `app/src/tests/landing.test.ts`
      verify: `bun run test -- landing` → 4 passed ✅

## Wave 3 (gates)

- [x] T-06 typecheck + full gates
      verify: `bun run typecheck` ✅ → ต่อด้วย `bun run build` + `bun run test` ก่อนปิดงาน

## Notes

- งานนี้ไม่มี backend/openapi → 4-place rule ไม่เกี่ยว (บันทึกไว้ใน requirements
  non-goals ตั้งแต่ต้น — ตัวอย่างที่ดีของการ scoping)
- ถ้าจะต่อยอด (เช่น แปลไทย, เพิ่ม screenshots, analytics): เปิด spec ใหม่
  อ้าง spec นี้ อย่าบวม tasks นี้
