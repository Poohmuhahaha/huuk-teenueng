# Huuk — Figma Plugin (wireframe generator) — complete

สร้าง **ทุกอย่าง** ให้อัตโนมัติใน Figma: design system + ทุกหน้าจอ + ทุก overlay + state matrix

## วิธีใช้
1. เปิด **Figma Desktop**
2. เปิดไฟล์ design ใหม่
3. **Plugins → Development → Import plugin from manifest…** → เลือก `huuk/figma-plugin/manifest.json`
4. **Plugins → Development → Huuk — Wireframe Generator → Run**

## ได้ทั้งหมด 4 pages

### 1) Huuk — Design System
- **Color tokens** 9 ตัว (ink · paper · muted · faint · line · entry · step · success · wash)
- **Type scale** 6 ระดับ (Display → Eyebrow)
- **Components 364 ตัว** (Figma components จริง) — ครบทุกอย่าง:
  - **Semantic (57):** Buttons · Forms & inputs · Navigation · Data display · Labels & status · Feedback & states · Overlays · Brand kit
  - **CSS classes (194)** จาก `app/src/app/style.css` ทั้งหมด — Ads · Buttons · Cards & brand · Chips & badges · Forms & inputs · Navigation & layout · Overlays & menus · Plans & campaigns · States & feedback · Tables & charts · Tabs & steps · Elements
  - **Vue components (30)** — shared components ทั้งหมด
  - **Vue pages (25)** — ทุกหน้า

### 2) Huuk — Screens (21 frames — ทุกหน้า + ทุก tab)
สไตล์ **minimal card** (มือถือ 430×932): หัวเรื่อง **serif ตัวพิมพ์ใหญ่** กลางบน · label + เส้นใต้ · ปุ่มแคปซูลดำล่างสุด
Login · Register · Plans · Welcome ·
Home ·
**Plan** (Monthly · Calendar · Ideas · Hashtags) ·
**Content** (Studio · Feed preview) ·
**Promote** (Campaigns · Meta Ads) ·
**Analyze** (Performance · Finance) ·
**Settings** (Brand · Workspace · Members · Connections) ·
Read (index · article) · 404

### 3) Huuk — Overlays (14)
สไตล์ minimal card เดียวกัน (หัว serif + ปุ่ม CONFIRM)
Account menu · Settings sheet · Login modal · Change password · Protected cell · Platform connect · OAuth page picker · Guide · Date picker · Ads (Edit budget · opt-in · Boost) · Publish menu · Revisions drawer

### 4) Huuk — States
ตาราง state ต่อหน้าจอ (loading · data · empty · error · no-perm · busy · validation · conflict 409 · locked-by-other · reconnect · not-found …)

## ปรับแก้
- เพิ่ม/แก้หน้าจอ: array `SCREENS` ใน `code.js`
- เพิ่ม/แก้ overlay: array `OVERS`
- เพิ่ม/แก้ state: array `STATES`
- เพิ่ม component: array `ALL_COMPONENTS` (+ วาดใน `drawComponentBody`)

## หมายเหตุ
- ฟอนต์ใช้ **Inter** (ปลอดภัยสุด) — ถ้ามี **Tinos** เปลี่ยน `family: 'Inter'` → `'Tinos'` เพื่อ serif display
- Figma REST API เป็น read-only → ต้องใช้ plugin นี้ในการ generate
