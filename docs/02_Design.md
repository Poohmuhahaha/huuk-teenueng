# Design — content-planner-huuk

- Tokens หลัก: `app/src/app/style.css` (~1010 บรรทัด) — `:root { --surface:#fff; --surface-2:#f8fafc; --ink:#0f172a; --accent:#4f46e5; --accent-hover:#4338ca; --radius:12px; --font:"Noto Sans Thai","Inter",system-ui }`
- Brand CI แบบลูกค้าตั้งเอง: `app/src/core/theme.ts` — รับ palette/fonts ของ Brand → เขียน CSS vars (`--brand-1..4`, shadow/radius tiers), validate `HEX`/`FONT_NAME`
- Assets: `app/public/huuk-logo.svg`, `favicon.svg`, `default-identity.svg`, `icons.svg`, `system-map.png`
- Spec: `business/features/theming/theming.md`
- งานดีไซน์ค้าง: ตรวจ `README.md` ตัวเลขนับ test (เขียน 16 แต่จริง 94+188) + เทียบ light/dark ของ Client `/#/studio` กับ public `/#/read/{slug}`
