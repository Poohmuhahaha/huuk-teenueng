# Huuk → Figma Handoff Kit

Everything here is generated from the repo (code + `portfolio/design.md`). Three tracks:

| Track | Folder | นำเข้า Figma ด้วย |
|---|---|---|
| **A. Architecture diagrams** | `diagrams/` + `diagrams/svg/` | ลาก SVG เข้า FigJam/Figma, หรือ plugin "Mermaid to FigJam", หรือ Figma MCP `generate_diagram` |
| **B. Design tokens** | `tokens/` | Plugin **Tokens Studio for Figma** → Import → สร้าง Variables/Styles อัตโนมัติ |
| **C. UI screens** | `ui-screens/build-spec.md` | สร้างด้วยมือตาม spec, หรือ Figma MCP (ต้องใช้ token ที่เขียนได้) |

> **ข้อจำกัดที่ต้องรู้:** Figma connection ที่ต่ออยู่ตอนนี้เป็น **read-only** (อ่านไฟล์ + คอมเมนต์/dev resources เท่านั้น) จึงสร้าง node/วาดอัตโนมัติให้ไม่ได้ ทุก track จึงส่งออกเป็นไฟล์ที่ **คุณลาก/อิมพอร์ตเอง** หรือใช้ Figma MCP ที่เขียนได้

---

## Track A — Architecture diagrams → FigJam/Figma

โฟลเดอร์ `diagrams/` มี Mermaid ต้นฉบับ 13 ไฟล์ (`.mmd`) และ SVG ที่เรนเดอร์แล้วใน `diagrams/svg/`

```
diagrams/
├── high-level-data-flow.mmd      ├── svg/high-level-data-flow.svg
├── deployment-topologies.mmd     ├── svg/deployment-topologies.svg
├── data-model-er.mmd             ├── svg/data-model-er.svg
├── auth-session-flow.mmd         ├── svg/auth-session-flow.svg
├── permission-resolution.mmd     ├── svg/permission-resolution.svg
├── content-planning-crud.mmd     ├── svg/content-planning-crud.svg
├── cms-content-lifecycle.mmd     ├── svg/cms-content-lifecycle.svg
├── cms-autosave-conflict.mmd     ├── svg/cms-autosave-conflict.svg
├── live-oauth-flow.mmd           ├── svg/live-oauth-flow.svg
├── ads-management-gate.mmd       ├── svg/ads-management-gate.svg
├── workspace-lifecycle.mmd       ├── svg/workspace-lifecycle.svg
├── app-render-gate.mmd           ├── svg/app-render-gate.svg
└── remediation-roadmap.mmd       └── svg/remediation-roadmap.svg
```

### วิธีที่ 1 — ลาก SVG (ง่ายสุด, ได้ vector แก้ไขได้)
1. เปิด Figma / FigJam
2. ลากไฟล์ `diagrams/svg/*.svg` ทั้งหมดเข้า canvas
3. FigJam: เลือกทั้งหมด → **Bulk edit → Ungroup** เพื่อแก้ node ได้; Figma Design: ได้ vector layer แก้ไขได้ทันที

### วิธีที่ 2 — Plugin (ได้ FigJam diagram ที่แก้ไขได้เหมือนวาดเอง)
1. เปิด FigJam board
2. **Resources → Plugins → ค้นหา “Mermaid to FigJam”** (หรือ “FigJam ↔ Mermaid Converter”) → Run
3. วางเนื้อหา `.mmd` ทีละไฟล์ → **Convert**
4. Diagram กลายเป็น FigJam shapes/connectors ที่แก้และ comment ได้

> `sequenceDiagram` / `stateDiagram` รองรับดีใน Creative นี้; ถ้า plugin ตัวไหนไม่รองรับ ให้ใช้ SVG

### วิธีที่ 3 — Figma MCP `generate_diagram` (อัตโนมัติ, สร้าง FigJam)
ถ้าเพิ่ม **Figma MCP server** ใน MCP client (ไม่ใช่ connection ผ่าน Composio นี้) จะมี tool `generate_diagram` รับ Mermaid แล้วสร้าง FigJam editable diagram ให้:
- ใส่ `mermaidSyntax` = เนื้อหาใน `.mmd`
- ระบุ `fileKey` ของ board เดิมถ้าต้องการรวมเข้าไฟล์เดียว
- ต้องโหลด skill `figma-generate-diagram` ก่อนเรียก (ตามข้อกำหนดของ Figma MCP)

---

## Track B — Design tokens → Figma Variables & Styles

จาก `portfolio/design.md` (ระบบ monochrome: ink บน paper, serif/sans duet, ทุกมุม 0px, hard-offset shadow)

| ไฟล์ | รูปแบบ | ใช้กับ |
|---|---|---|
| `tokens/tokens-studio.json` | Tokens Studio for Figma | **แนะนำ** — import แล้วได้ Variables + Text/Color/Effect styles ครบ |
| `tokens/tokens.dtcg.json` | W3C DTCG | เครื่องมืออื่น หรือ Tokens Studio (รองรับ DTCG) |

### ขั้นตอน (Tokens Studio)
1. Figma → **Plugins → Tokens Studio for Figma** (ฟรี) → Run
2. เมนู **Tools → Load from file / Import** → เลือก `tokens/tokens-studio.json`
3. ได้ token sets: `color`, `fontFamilies`, `fontSizes`, `spacing`, `borderRadius`, `boxShadow`, `motion`, `duration`, `other` (breakpoints/header height)
4. กด **Create variables** (หรือ **Apply to selection**) เพื่อให้ Tokens Studio สร้าง:
   - **Variables** สี/ตัวเลข
   - **Text styles** ตาม typography (display-1 … tag)
   - **Effect styles** สำหรับ `hard-lift`
5. (ทางเลือก) import `tokens.dtcg.json` ด้วย DTCG preset ถ้าทีมใช้ format กลาง

**หมายเหตุการแปลง**
- ค่า `clamp(...)` เก็บเป็น **ค่าสูงสุด** (เช่น display-1 = 92, heading-1 = 44) เพราะ Figma ยังไม่รองรับ clamp ในตัวแปร — ทำเป็น responsive mode แยกได้ถ้าต้องการ
- `paper-frosted` = ขาว 88% (rgba) — Figma variable รองรับ alpha
- Shadow `hard-lift` ต้องคู่กับ `translate(-3px,-3px)` จึงจะเห็นผลแบบในเว็บ
- ไม่มี accent hue ใด ๆ — ink (#000000) ทำหน้าที่เป็น accent เดียว

---

## Track C — UI screens → Figma

ไฟล์ `ui-screens/build-spec.md` มี:
- Frame size ต่อหน้า (Desktop 1440 / Tablet 900 / Mobile 390)
- รายการ frame + component ที่ต้องสร้าง (Header, Hero, Work card, Team card, Capability row, Conditions band, Footer ฯลฯ)
- การแมปไปยัง token (สี/ฟอนต์/spacing/radius/shadow) จาก Track B
- สถานะ (rest/hover/focus) และ motion spec

### วิธีสร้าง
1. **ด้วยมือ:** import tokens (Track B) → สร้าง component ตาม `build-spec.md` → ผูก style กับ variables
2. **Figma MCP (อัตโนมัติ):** ถ้าตั้ง Figma MCP แบบเขียนได้ → สั่งสร้างทีละ frame จาก spec
3. **Plugin/Dev Mode:** เปิด Design file แล้วใช้ Dev Mode อ้างอิงกับแอปจริง (`app/`) เพื่อ parity

> ถ้าต้องการให้ผมวาดลง Figma ให้อัตโนมัติ ต้องมี **Figma token แบบเขียนได้** (หรือ Figma MCP `use_figma`/plugin) — token ปัจจุบันอ่านอย่างเดียว

---

## โครงสร้างโฟลเดอร์

```
docs/figma/
├── README.md                 ← ไฟล์นี้
├── diagrams/
│   ├── *.mmd                 ← Mermaid ต้นฉบับ (13)
│   └── svg/*.svg             ← SVG พร้อมลากเข้า Figma
├── tokens/
│   ├── tokens-studio.json    ← import เข้า Tokens Studio
│   └── tokens.dtcg.json      ← W3C DTCG
└── ui-screens/
    └── build-spec.md         ← สเปกสร้างหน้าจอ
```

## แหล่งที่มา (provenance)
- แผนผังทั้งหมด: reverse-engineered จาก `server/src/` + `app/src/` (ดู `docs/ARCHITECTURE-E2E-FLOW.md`)
- Tokens: ดึงจาก `portfolio/design.md` (monochrome system)
- เรนเดอร์ SVG ผ่าน `mermaid.ink` (ไม่ต้องลง Chromium): `GET https://mermaid.ink/svg/<base64url>`
