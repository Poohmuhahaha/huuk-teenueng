# Huuk / Teenueng — Figma UI Build Spec

สเปกสำหรับสร้างหน้าจอใน Figma ให้ตรงกับ `portfolio/design.md` (design system) และโครงหน้าแอปใน `app/src/`.
สร้าง component ให้ผูกกับ **Variables/Styles จาก Track B** (`../tokens/tokens-studio.json`) เสมอ.

---

## 1. Frames & Breakpoints

| Frame | Width | หมายเหตุ |
|---|---|---|
| Desktop | 1440 | Full-bleed, gutter `clamp(24,5vw,80)` = 80 ที่ 1440 |
| Tablet | 900 | Hero collapse 1 คอลัมน์ |
| Mobile | 390 | Grids เป็น 1 คอลัมน์, ปุ่มเต็มความกว้าง |
| Work card | 1200×630 | Media 16:9 hairline frame |
| Avatar | 72×72 | Ink frame 1px, object-fit cover |

**Layout:** ไม่มี centred max-width — ใช้ `--wrap: 100%` + gutter. Band เปิดด้วย padding-block `clamp(48px,8vw,80px)` และปิดด้วย hairline 1px.

---

## 2. Screens (จาก `app/src/core/screens.ts`)

### Public / Onboarding (ไม่มี app shell)
| # | Frame | Route | องค์ประกอบ |
|---|---|---|---|
| P1 | Login | `/#/login` | Wordmark, email/password, CTA solid, link → register |
| P2 | Register | `/#/register` | name/email/password (min 12) |
| P3 | Plans | `/#/plans` | 3 tier card (Team card chrome), featured = inverted ink |
| P4 | Workspace setup | `/#/welcome` | Field + solid CTA, full-screen centre |
| P5 | Public reader (index) | `/#/read` | List of published items |
| P6 | Public reader (article) | `/#/read/:slug` | Serif title, markdown body, tags |

### App deck (shell + ScreensDeck) — 14 การ์ด
| # | Frame | Route | องค์ประกอบหลัก |
|---|---|---|---|
| 1 | Brand | `/#/brand` | CI form, palette swatches, logos, moodboard, font upload |
| 2 | Planner (month) | `/#/planner/:month` | Master table 01–12, filter bar, row editor, lock state |
| 3 | Calendar | `/#/calendar` | Calendar grid, pillar/platform/format/status filters |
| 4 | Feed | `/#/feed` | Platform tabs, SocialMediaBar, 9-post canvas grid |
| 5 | Dashboard | `/#/dashboard` | KPI cards, pillar bar chart, top-5 table, Ads glance |
| 6 | Performance | `/#/performance` | Metrics tables, import button, +7d table |
| 7 | Ideas | `/#/ideas` | Idea list + toggle + promote |
| 8 | Hashtags | `/#/hashtags` | Hashtag group cards + add tag |
| 9 | Finance | `/#/finance` | Ledger table, income/expense/balance charts |
| 10 | Live | `/#/live` | Account chips, mirrored post cards, sync/reconnect |
| 11 | Campaigns | `/#/campaigns/:id?` | Campaign list + detail (brief, schedule, linked content) |
| 12 | Ads | `/#/ads` | Account picker, summary tiles, campaign tree, audit, dialogs |
| 13 | Members | `/#/members` | Owner + member rows, add/remove (owner-only) |
| 14 | Content Studio | `/#/studio/:id?` | List + Markdown editor + live preview + revisions |

**Role variant:** role `Client` → deck เหลือเฉพาะ Content Studio.

---

## 3. Components (chrome ทั้งหมดมุม 0px)

| Component | องค์ประกอบ | States |
|---|---|---|
| `site-header` | frosted paper 88% + blur(12), 68px, hairline bottom; wordmark + lang toggle + solid btn-sm | rest / scrolled |
| `wordmark` | Tinos 700 18, lowercase | ink / paper (footer) |
| `lang-toggle` | label 12 uppercase; active ink w600 | rest / hover / active |
| `button-solid` | ink fill, paper text, radius 0, padding 16/28 | rest / hover+focus (lift) |
| `button-ghost` | transparent, ink border/text | rest / hover+focus |
| `button-small` | solid 13px, 10/18 | rest / hover / focus |
| `work-card` | 16:9 grayscale media hairline frame → product name (heading-2) → meta → desc → tag wrap | rest / hover |
| `feature-tag` | 11 uppercase +0.08em, 1px line, 5/10 | static |
| `team-card` | 72 avatar ink frame + name (title) + role label + bio | rest / hover (Level-2 lift) |
| `principle-col` | 21 Tinos 700 title + body-sm muted | static |
| `capability-row` | hairline top, row-title serif, faint counter `01–04` right | rest / hover |
| `text-input` | white, 1px line, radius 0, body-sm | rest / focus (2px ink ring, offset 3) |
| `toast` | inverted ink chip | enter/exit (reveal) |
| `hero-studio-scene` | ink frame; SVG sketch + floor + 2 stick figures | animated |
| `site-footer` | solid ink, paper wordmark/links | rest / hover underline |
| `empty-state` | sketch greys in hairline frame + caption | static |

**Elevation:** ระดับเดียวที่ใช้คือ `hard-lift` = `translate(-3px,-3px)` + shadow `6px 6px 0 ink` (blur 0) เฉพาะ hover/focus ของปุ่มกับ team-card; ที่ rest เป็น flat hairline ทั้งหมด.

---

## 4. Typography mapping (ผูก Text Styles)

| Style | Family | Size | Weight | Line | Tracking | Case |
|---|---|---|---|---|---|---|
| display-1 | Tinos | 92 | 700 | 1.02 | −2% | — |
| display-2 | Tinos | 80 | 700 | 1.02 | −2% | — |
| heading-1 | Tinos | 44 | 700 | auto | −1.5% | — |
| heading-2 | Tinos | 36 | 700 | 1.04 | −1.5% | — |
| title | Tinos | 28 | 700 | auto | −1% | — |
| row-title | Tinos | 30 | 400 | auto | −1% | — |
| heading-3 | Tinos | 21 | 700 | auto | −1% | — |
| wordmark | Tinos | 18 | 700 | auto | −1% | lowercase |
| lede | Inter | 20 | 400 | 1.55 | −1.1% | — |
| body-md | Inter | 17 | 400 | 1.55 | −1.1% | — |
| body-sm | Inter | 16 | 400 | 1.55 | −1.1% | — |
| button | Inter | 15 | 500 | auto | −1% | — |
| caption | Inter | 14 | 400 | auto | auto | — |
| eyebrow | Inter | 12 | 400 | auto | +22% | UPPERCASE |
| label | Inter | 12 | 400–600 | 1.7 | +8% | UPPERCASE |
| tag | Inter | 11 | 400 | auto | +8% | UPPERCASE |

---

## 5. Color usage (Variables)

| Token | ใช้กับ |
|---|---|
| `paper` | canvas, card/field surface, footer text |
| `ink` | CTA fill, focus ring, active state, ทุก structural border ของ interactive, footer surface |
| `line` | hairline: band rules, tag border, media frames |
| `muted` | lede, secondary body, eyebrow at rest |
| `faint` | metadata, caption, counter, inactive lang |
| `ink-body` | work-card description |
| `sketch-*` / `floor` | **เฉพาะ** ใน hero illustration |

**ห้าม:** ใส่ hue ใด ๆ, รัศมีมุมใน chrome, drop-shadow เบลอ, centred max-width, หรือใช้ inversion ซ้ำในหน้า (สงวน ink surface ไว้ที่ footer เดียว).

---

## 6. Motion spec

| รายการ | ค่า |
|---|---|
| Ease | cubic-bezier(0.22, 1, 0.36, 1) |
| Reveal | opacity 0→1, y 24→0, 0.6s, inView −12% bottom |
| Hero stagger | y 28→0, 0.7s, 0.09s stagger |
| Interactive | 0.15–0.2s translate/shadow/color |
| Reduced motion | `prefers-reduced-motion: reduce` → static opacity 1 ทุกอย่าง |

ใน Figma: ทำ prototype “After delay” + Smart Animate ตามค่าข้างบน (Figma ไม่มีในตัว แต่ใส่ interaction note ไว้ได้).

---

## 7. Component → App page matrix

| Component | ใช้ในหน้า |
|---|---|
| `site-header`, `wordmark`, `lang-toggle`, `button-small` | ทุกหน้า (AppShell) |
| `work-card`, `feature-tag`, `capability-row` | Brand, Campaigns |
| `team-card` | Members, Plans (featured = inverted) |
| `text-input` | Brand, Settings, Login/Register/Plans, Content editor |
| `toast` | AppShell (OAuth callback, save feedback) |
| `hero-studio-scene` | Dashboard / Live hero section |
| `empty-state` | ทุกหน้าที่มี list (Studio, Ideas, Ads, Live) |
| MasterTable / TxnTable | Planner, Finance, Performance |

---

## 8. Parity checklist (ก่อน handoff)

- [ ] ทุกสี/ฟอนต์/spacing ผูกกับ Variables (Track B) ไม่มี hard-coded
- [ ] Boolean/props ครบสำหรับ states: default / hover / focus / disabled
- [ ] Desktop 1440 + Mobile 390 ตรวจแล้ว collapse ถูก
- [ ] ระยะ gutters 80 (desktop) / ไม่มี max-width กลาง
- [ ] Focus ring 2px offset 3 (flip เป็น paper ใน footer)
- [ ] Photography ใช้ `grayscale(1) contrast(1.04)` ใน 16:9 frame
- [ ] `prefers-reduced-motion` note ครบ
