// Huuk — Wireframe Generator (Figma plugin) — complete
// Run: Figma desktop → Plugins → Development → Import plugin from manifest… → Run
// Creates 4 pages:
//   Huuk — Design System  : colors · type scale · ALL components (56)
//   Huuk — Screens        : every screen + every tab (21 frames)
//   Huuk — Overlays       : every sheet/popup/modal/drawer (14)
//   Huuk — States         : per-screen state matrix
// Tokens follow portfolio/design.md (monochrome ink-on-paper, square corners).

const C = {
  ink:    { r: 0.0,  g: 0.0,  b: 0.0  },
  body:   { r: 0.10, g: 0.10, b: 0.10 },
  muted:  { r: 0.42, g: 0.42, b: 0.42 },
  faint:  { r: 0.64, g: 0.64, b: 0.64 },
  line:   { r: 0.894,g: 0.894,b: 0.894},
  paper:  { r: 1.0,  g: 1.0,  b: 1.0  },
  entry:  { r: 0.949,g: 0.651,b: 0.620},
  step:   { r: 0.663,g: 0.898,b: 0.874},
  success:{ r: 0.737,g: 0.922,b: 0.663},
  wash:   { r: 0.97, g: 0.97, b: 0.97 },
}

const FONTS = [
  { family: 'Inter', style: 'Regular' },
  { family: 'Inter', style: 'Medium' },
  { family: 'Inter', style: 'Semi Bold' },
  { family: 'Inter', style: 'Bold' },
]

function solid(color, opacity = 1) { return { type: 'SOLID', color: { ...color }, opacity } }

let SERIF = 'Inter'
async function loadSerif() {
  const cands = [['Tinos', 'Bold'], ['Georgia', 'Bold'], ['Times New Roman', 'Bold'],
    ['Liberation Serif', 'Bold'], ['Noto Serif', 'Bold']]
  for (const [family, style] of cands) {
    try { await figma.loadFontAsync({ family, style }); SERIF = family; return } catch (e) {}
  }
}

async function main() {
  for (const f of FONTS) { try { await figma.loadFontAsync(f) } catch (e) {} }
  await loadSerif()

  const p1 = figma.createPage(); p1.name = 'Huuk — Design System'
  const p2 = figma.createPage(); p2.name = 'Huuk — Screens'
  const p3 = figma.createPage(); p3.name = 'Huuk — Overlays'
  const p4 = figma.createPage(); p4.name = 'Huuk — States'

  buildDesignSystem(p1)
  buildScreens(p2)
  buildOverlays(p3)
  buildStates(p4)

  figma.currentPage = p1
  figma.notify('Huuk: design system + screens + overlays + states created ✓')
  figma.closePlugin()
}

// ---------- primitives ----------
function rect(w, h, fill, stroke, strokeW) {
  const r = figma.createRectangle(); r.resize(w, h)
  r.fills = fill ? [solid(fill)] : []
  if (stroke) { r.strokes = [solid(stroke)]; r.strokeWeight = strokeW || 1 }
  r.cornerRadius = 0
  return r
}
function text(chars, size, weight, color) {
  const t = figma.createText()
  t.fontName = { family: 'Inter', style: weight || 'Regular' }
  t.fontSize = size; t.characters = chars; t.fills = [solid(color || C.ink)]
  return t
}
function place(parent, node, x, y) { parent.appendChild(node); node.x = x; node.y = y; return node }
function frame(name, w, h, fill) {
  const f = figma.createFrame(); f.name = name; f.resize(w, h)
  f.fills = fill ? [solid(fill)] : [solid(C.paper)]; f.cornerRadius = 0; f.clipsContent = false
  return f
}
function sectionTitle(parent, label, x, y) { place(parent, text(label, 11, 'Semi Bold', C.faint), x, y) }

// =========================================================
// 1) DESIGN SYSTEM — colors, type, ALL components
// =========================================================
const BUTTONS = ['Button/Solid', 'Button/Ghost', 'Button/Small', 'Button/Danger', 'Icon Button', 'Tab', 'Segmented Control']
const FORMS = ['Text Field', 'Textarea', 'Select', 'Checkbox', 'Radio / Segmented', 'Search Field', 'Date Field',
  'Range Slider', 'Number Input', 'Color Picker', 'File Upload']
const NAV = ['Navbar', 'Nav Link', 'Nav Group', 'Avatar Button', 'Hub Tabs', 'Carousel Tabs', 'Status Funnel']
const DATA = ['KPI Card', 'Budget Card', 'Bar Chart', 'Calendar Grid', 'Feed Grid', 'Table (Master)', 'Table (Transactions)',
  'Stat Tile', 'Definition List']
const LABELS = ['Chip', 'Feature Tag', 'Status Chip', 'Badge', 'Platform Status Row', 'Lock Chip']
const FEEDBACK = ['Alert / Error', 'Notice (success)', 'Empty State', 'Loading / Skeleton', 'Info Tip', 'Banner']
const OVERLAYS = ['Modal', 'Sheet / Dialog', 'Drawer', 'Popup Menu', 'Fullscreen Overlay']
const BRANDKIT = ['Logo Slot', 'Palette Tile', 'Swatch + Hex', 'Font Sample', 'Moodboard Item', 'Card Base']

const ALL_COMPONENTS = [
  ["Buttons", ["Button/Solid", "Button/Ghost", "Button/Small", "Button/Danger", "Icon Button", "Tab", "Segmented Control"]],
  ["Forms & inputs", ["Text Field", "Textarea", "Select", "Checkbox", "Radio / Segmented", "Search Field", "Date Field", "Range Slider", "Number Input", "Color Picker", "File Upload"]],
  ["Navigation", ["Navbar", "Nav Link", "Nav Group", "Avatar Button", "Hub Tabs", "Carousel Tabs", "Status Funnel"]],
  ["Data display", ["KPI Card", "Budget Card", "Bar Chart", "Calendar Grid", "Feed Grid", "Table (Master)", "Table (Transactions)", "Stat Tile", "Definition List"]],
  ["Labels & status", ["Chip", "Feature Tag", "Status Chip", "Badge", "Platform Status Row", "Lock Chip"]],
  ["Feedback & states", ["Alert / Error", "Notice (success)", "Empty State", "Loading / Skeleton", "Info Tip", "Banner"]],
  ["Overlays", ["Modal", "Sheet / Dialog", "Drawer", "Popup Menu", "Fullscreen Overlay"]],
  ["Brand kit", ["Logo Slot", "Palette Tile", "Swatch + Hex", "Font Sample", "Moodboard Item", "Card Base"]],
  ["CSS \u00b7 Ads", ["ads-link"]],
  ["CSS \u00b7 Buttons", ["btn", "btn-primary", "chipbtn", "conn-btn", "expand-btn", "menubtn", "platbtn", "strip-btn", "switchlink", "toolbtn"]],
  ["CSS \u00b7 Cards & brand", ["brand-logo", "camp-item", "camp-item-head", "card", "cards", "cover", "font-check", "font-choice", "font-import", "font-import-plus", "font-name", "font-preview", "font-preview-sample", "font-remove", "font-sample", "heroimg", "identity-card", "identity-hint", "identity-meta", "identity-preview", "linkitem", "logo-preview", "logo-remove", "logo-slot", "lp-card", "lp-logo", "mood-add", "mood-fix", "mood-item", "moodboard", "palette-caption", "palette-card", "palette-copy", "palette-meta", "palette-stats", "palette-strip", "palette-tile", "palette-tile-label", "palette-tile-role", "palette-tile-sample", "palette-tile-star", "panel", "panel-note", "pcard", "ph", "plan-card", "plans-card", "slide-card", "strip-label", "strip-link", "style-panel", "ws-item"]],
  ["CSS \u00b7 Chips & badges", ["badge-danger", "badge-success", "badge-warning", "chip", "chiprow", "dark", "dot", "identity-badge", "is-primary", "plan-badge", "plan-tag", "status-dot"]],
  ["CSS \u00b7 Elements", ["avatar", "avatar-wrap", "body", "cell", "feed", "img-bounds", "loginpage", "mt", "oversize-fix", "platform-wrap", "platrow", "row", "spacer", "style-export", "style-label", "style-row", "style-sample", "style-segmented", "style-value", "surface-raised", "sw"]],
  ["CSS \u00b7 Forms & inputs", ["daynum", "dp-caret", "dp-field", "editable", "field", "lbl", "num", "palette-field", "palette-hex", "style-num", "style-range", "ws-caret"]],
  ["CSS \u00b7 Navigation & layout", ["app", "brand", "brand-home", "brandpage", "deck", "footer", "lang", "links", "lp-brand", "lp-by", "lp-hint", "lp-lang", "lp-submit", "lp-switch", "lp-ws", "main", "navdate", "navdrop", "navgroup", "navlink", "side", "slide", "socialcheck", "socialmini", "socialname", "ws-actions", "ws-check", "ws-count", "ws-form", "ws-head", "ws-name", "ws-trigger", "ws-wrap"]],
  ["CSS \u00b7 Overlays & menus", ["avatar-menu", "backdrop", "dp-day", "dp-grid", "dp-head", "dp-modal", "dp-nav", "font-pick", "guide-head", "guide-overlay", "guide-panel", "modal", "navmenu", "navmenu-link", "pick-list", "pick-row", "platmenu", "plogin", "plogin-actions", "plogin-body", "plogin-brand", "plogin-card", "plogin-close", "plogin-full", "plogin-note", "plogin-title", "pop-actions", "pop-block", "pop-connect", "settings-body", "ws-menu"]],
  ["CSS \u00b7 Plans & campaigns", ["camp-editor", "camp-list", "camp-status", "plan-cta", "plan-current", "plan-list", "plan-price", "plans-foot", "plans-head", "plans-sub", "planspage"]],
  ["CSS \u00b7 States & feedback", ["actions", "active", "autherr", "completed", "conn-name", "conn-row", "connected", "connecting", "danger", "disabled", "done", "dragging", "empty", "error", "fading", "full", "ghost", "hollow", "infotip", "infotip-bubble", "infotip-wrap", "muted", "oauth-banner", "oknote", "on", "open", "paused", "popular", "router-link-active", "selected", "warnbox", "wide", "wire", "wire-edit"]],
  ["CSS \u00b7 Tables & charts", ["bar", "barlabels", "bars", "cal", "camp-grid", "card-scroll", "day", "font-grid", "grid2", "grid3", "grid4", "grid6", "linkgrid", "palette-grid", "plans-grid", "shellgrid", "socialbar", "socialbar-manage", "socialbar-title", "strip-bar", "strip-bars", "tbl", "tblwrap", "today", "topbar"]],
  ["CSS \u00b7 Tabs & steps", ["dow", "dp-dow", "identity-download", "sortable", "step", "step-mark", "steps", "steps-list", "tab", "tabs"]],
  ["Vue components (shared)", ["AppShell", "BarChart", "BudgetCards", "CalendarGrid", "CardBase", "CarouselTabs", "ChangePasswordModal", "Chip", "ContentEditor", "CopyBar", "DatePickerPopup", "ErrorBoundary", "FeedGrid", "GuideHero", "HashtagGroups", "HubTabs", "InfoTip", "KpiCard", "LiveSection", "LoginModal", "MasterTable", "PlatformLogin", "ProfilePanel", "ProtectedModal", "ScreensDeck", "SettingsPanel", "SocialMediaBar", "StatusChip", "StatusFunnel", "TxnTable"]],
  ["Vue pages", ["AdsPage", "AnalyzePage", "BrandPage", "CalendarPage", "CampaignsPage", "ContentPage", "DashboardPage", "FeedPage", "FinancePage", "HashtagsPage", "IdeasPage", "LivePage", "LoginPage", "MembersPage", "NotFoundPage", "PerformancePage", "PlanPage", "PlannerPage", "PlansPage", "PromotePage", "ReadPage", "RegisterPage", "SettingsPage", "StudioPage", "WorkspaceSetupPage"]],
]

function drawComponentBody(comp, name, w, h) {
  // section-specific placeholder visuals
  if (name.startsWith('Button')) {
    const solidBtn = name.includes('Solid')
    comp.fills = solidBtn ? [solid(C.ink)] : []
    comp.strokes = [solid(C.ink)]
    const t = text(name.split('/')[1] || name, 13, 'Medium', solidBtn ? C.paper : C.ink)
    comp.appendChild(t); t.x = (w - t.width) / 2; t.y = (h - t.height) / 2
  } else if (name === 'Tab' || name === 'Feature Tag' || name === 'Chip' || name === 'Badge') {
    comp.fills = []; comp.strokes = [solid(C.line)]
    const t = text(name.toLowerCase(), 11, 'Regular', C.muted); comp.appendChild(t); t.x = 10; t.y = (h - t.height) / 2
  } else if (name === 'Status Chip' || name === 'Lock Chip') {
    comp.fills = [solid(C.ink)]; const t = text('status', 11, 'Medium', C.paper); comp.appendChild(t); t.x = 10; t.y = (h - t.height) / 2
  } else if (name.includes('Field') || name === 'Input' || name === 'Textarea' || name === 'Select' || name === 'Number Input') {
    comp.fills = [solid(C.paper)]; comp.strokes = [solid(C.line)]
    const t = text('placeholder', 12, 'Regular', C.faint); comp.appendChild(t); t.x = 12; t.y = 12
  } else if (name === 'Search Field') {
    comp.fills = [solid(C.paper)]; comp.strokes = [solid(C.line)]
    const t = text('Search…', 12, 'Regular', C.faint); comp.appendChild(t); t.x = 12; t.y = 12
  } else if (name === 'KPI Card' || name === 'Stat Tile') {
    comp.fills = [solid(C.paper)]; comp.strokes = [solid(C.line)]
    comp.appendChild(place(comp, text('Label', 10, 'Regular', C.muted), 12, 10))
    comp.appendChild(place(comp, text('128', 26, 'Bold', C.ink), 12, 28))
  } else if (name === 'Budget Card') {
    comp.fills = [solid(C.paper)]; comp.strokes = [solid(C.line)]
    comp.appendChild(place(comp, text('Income', 10, 'Regular', C.muted), 12, 12))
    comp.appendChild(place(comp, text('฿ 0', 18, 'Bold', C.ink), 12, 30))
  } else if (name === 'Card Base' || name === 'Moodboard Item' || name === 'Logo Slot') {
    comp.fills = [solid(C.paper)]; comp.strokes = [solid(C.line)]
    comp.appendChild(place(comp, rect(w, h * 0.55, C.wash, C.line, 1), 0, 0))
    comp.appendChild(place(comp, text(name, 11, 'Regular', C.muted), 10, h * 0.62))
  } else if (name === 'Bar Chart') {
    comp.fills = [solid(C.paper)]; comp.strokes = [solid(C.line)]
    for (let i = 0; i < 5; i++) comp.appendChild(place(comp, rect(12, 20 + i * 12, C.ink), 12 + i * 22, h - 20 - (20 + i * 12)))
  } else if (name === 'Calendar Grid' || name === 'Feed Grid') {
    comp.fills = [solid(C.paper)]; comp.strokes = [solid(C.line)]
    const cols = 3, cell = w / cols
    for (let i = 0; i < 9; i++) comp.appendChild(place(comp, rect(cell - 6, h / 3 - 6, C.wash, C.line, 1), (i % 3) * cell + 3, Math.floor(i / 3) * (h / 3) + 3))
  } else if (name.startsWith('Table')) {
    comp.fills = [solid(C.paper)]; comp.strokes = [solid(C.line)]
    for (let i = 0; i < 4; i++) comp.appendChild(place(comp, rect(w, 1, C.line), 0, 26 + i * 24))
    comp.appendChild(place(comp, text('Header', 10, 'Semi Bold', C.muted), 8, 8))
  } else if (name === 'Modal' || name === 'Sheet / Dialog' || name === 'Drawer' || name === 'Popup Menu' || name === 'Fullscreen Overlay') {
    comp.fills = [solid(C.paper)]; comp.strokes = [solid(C.ink)]
    comp.appendChild(place(comp, rect(w, 34, C.ink), 0, 0))
    comp.appendChild(place(comp, text(name, 12, 'Medium', C.paper), 12, 9))
  } else if (name === 'Alert / Error') {
    comp.fills = [solid(C.paper)]; comp.strokes = [solid(C.ink)]
    comp.appendChild(place(comp, text('! message', 12, 'Regular', C.body), 12, 12))
  } else if (name === 'Notice (success)') {
    comp.fills = [solid(C.success)]; comp.appendChild(place(comp, text('Success', 12, 'Medium', C.ink), 12, 12))
  } else if (name === 'Empty State' || name === 'Loading / Skeleton') {
    comp.fills = [solid(C.wash)]; comp.strokes = [solid(C.line)]
    comp.appendChild(place(comp, text(name, 11, 'Regular', C.muted), 12, (h - 14) / 2))
  } else if (name === 'Info Tip') {
    comp.fills = [solid(C.ink)]; const t = text('i', 12, 'Bold', C.paper); comp.appendChild(t); t.x = w / 2 - 3; t.y = h / 2 - 8
  } else if (name === 'Banner') {
    comp.fills = [solid(C.paper)]; comp.strokes = [solid(C.line)]
    comp.appendChild(place(comp, text('banner message', 12, 'Regular', C.body), 12, 12))
  } else if (name === 'Swatch + Hex' || name === 'Palette Tile') {
    comp.fills = [solid(C.ink)]; comp.appendChild(place(comp, text('#000000', 10, 'Regular', C.paper), 8, h - 20))
  } else if (name === 'Font Sample') {
    comp.fills = [solid(C.paper)]; comp.strokes = [solid(C.line)]
    comp.appendChild(place(comp, text('Aa Bb 123 สวัสดี', 16, 'Bold', C.ink), 12, 12))
  } else if (name === 'Checkbox' || name === 'Radio / Segmented') {
    comp.fills = []; comp.strokes = [solid(C.line)]
    comp.appendChild(place(comp, rect(16, 16, C.ink), 12, (h - 16) / 2))
    comp.appendChild(place(comp, text('option', 12, 'Regular', C.body), 36, (h - 14) / 2))
  } else if (name === 'Range Slider') {
    comp.fills = [solid(C.paper)]
    comp.appendChild(place(comp, rect(w - 24, 2, C.line), 12, h / 2))
    comp.appendChild(place(comp, rect(14, 14, C.ink), w / 2, h / 2 - 7))
  } else if (name === 'Color Picker') {
    comp.fills = [solid(C.ink)]; comp.strokes = [solid(C.line)]
  } else if (name === 'File Upload') {
    comp.fills = [solid(C.paper)]; comp.strokes = [solid(C.line)]
    comp.appendChild(place(comp, text('Upload image', 12, 'Medium', C.ink), 12, (h - 14) / 2))
  } else if (name === 'Navbar') {
    comp.fills = [solid(C.paper)]; comp.strokes = [solid(C.line)]
    comp.appendChild(place(comp, text('huuk', 11, 'Bold', C.ink), 10, 20))
    ;['Home', 'Plan', 'Content', 'Promote', 'Analyze', 'Settings'].forEach((n, i) =>
      comp.appendChild(place(comp, text(n, 7, 'Regular', C.muted), 52 + i * 28, 23)))
  } else if (name === 'Nav Link' || name === 'Nav Group') {
    comp.fills = []; comp.appendChild(place(comp, text('Nav', 12, 'Regular', name === 'Nav Group' ? C.ink : C.muted), 10, (h - 14) / 2))
    if (name === 'Nav Group') comp.appendChild(place(comp, text('▾', 12, 'Regular', C.muted), w - 24, (h - 14) / 2))
  } else if (name === 'Avatar Button') {
    comp.fills = [solid(C.ink)]; const t = text('P', 14, 'Bold', C.paper); comp.appendChild(t); t.x = w / 2 - 5; t.y = h / 2 - 9
  } else if (name === 'Hub Tabs' || name === 'Carousel Tabs') {
    comp.fills = [solid(C.paper)]; comp.strokes = [solid(C.line)]
    ;['A', 'B', 'C'].forEach((n, i) => {
      const on = i === 0
      comp.appendChild(place(comp, rect(58, 24, on ? C.ink : C.paper, on ? null : C.line, 1), 8 + i * 62, 6))
      comp.appendChild(place(comp, text(n, 11, 'Semi Bold', on ? C.paper : C.muted), 8 + i * 62 + 24, 11))
    })
  } else if (name === 'Status Funnel') {
    comp.fills = [solid(C.paper)]; comp.strokes = [solid(C.line)]
    ;['Start', 'Design', 'Dev', 'Done'].forEach((n, i) => {
      comp.appendChild(place(comp, rect(74, 24, i === 0 ? C.ink : C.wash, C.line, 1), 8 + i * 80, 6))
      comp.appendChild(place(comp, text(n, 10, 'Medium', i === 0 ? C.paper : C.body), 8 + i * 80 + 20, 11))
    })
  } else {
    comp.fills = [solid(C.paper)]; comp.strokes = [solid(C.line)]
    comp.appendChild(place(comp, text(name, 11, 'Regular', C.muted), 10, (h - 14) / 2))
  }
}

function createComp(name, w, h) {
  const c = figma.createComponent(); c.name = name
  c.resize(w, h); c.fills = []; c.strokes = []; c.cornerRadius = 0
  drawComponentBody(c, name, w, h)
  return c
}

function buildDesignSystem(page) {
  let est = 680
  for (const [, names] of ALL_COMPONENTS) est += 22 + Math.ceil(names.length / 4) * 96 + 30
  est += 80
  const root = frame('Design System', 1360, est, C.paper)
  page.appendChild(root); root.x = 0; root.y = 0

  place(root, text('Huuk — Design System', 34, 'Bold', C.ink), 48, 40)
  place(root, text('monochrome ink-on-paper · square corners · serif display / sans UI (' + (ALL_COMPONENTS.reduce((a, g) => a + g[1].length, 0)) + ' components)', 12, 'Regular', C.muted), 48, 88)

  sectionTitle(root, 'COLOR', 48, 130)
  const colors = [['ink', C.ink], ['paper', C.paper], ['muted', C.muted], ['faint', C.faint], ['line', C.line],
    ['entry', C.entry], ['step', C.step], ['success', C.success], ['wash', C.wash]]
  colors.forEach((c, i) => {
    place(root, rect(120, 64, c[1], C.line, 1), 48 + i * 132, 152)
    place(root, text(c[0], 10, 'Semi Bold', C.muted), 48 + i * 132, 222)
  })

  sectionTitle(root, 'TYPE', 48, 270)
  const types = [['Display', 40, 'Bold'], ['Heading 1', 28, 'Bold'], ['Heading 2', 20, 'Bold'],
    ['Body', 14, 'Regular'], ['Label', 11, 'Semi Bold'], ['Eyebrow', 10, 'Semi Bold']]
  let ty = 296
  for (const [name, size, w] of types) {
    place(root, text(name, size, w, C.ink), 48, ty)
    place(root, text(`${size}px · ${w}`, 10, 'Regular', C.faint), 470, ty + size * 0.4)
    ty += size + 22
  }

  // components grid
  let gy = 660
  for (const [group, names] of ALL_COMPONENTS) {
    sectionTitle(root, group.toUpperCase(), 48, gy)
    gy += 22
    const perRow = 4
    names.forEach((n, i) => {
      const col = i % perRow, row = Math.floor(i / perRow)
      const w = 220, h = 56
      const c = createComp(n, w, h)
      place(root, c, 48 + col * 240, gy + row * 96)
      place(root, text(n, 9, 'Regular', C.faint), 48 + col * 240, gy + row * 96 + h + 14)
    })
    gy += Math.ceil(names.length / perRow) * 96 + 30
  }
}

// =========================================================
// 2) SCREENS — every screen + every tab
// =========================================================
const SCREENS = [
  { name: 'Login (public)', tabs: [], sections: [
    ['Card', ['brand (logo + workspace)', 'Email', 'Password', 'BTN Log in', 'link: No account? Register']],
    ['States', ['idle · submitting · 401 · 429']],
  ]},
  { name: 'Register (public)', tabs: [], sections: [
    ['Card', ['Name · Email · Password (≥12)', 'BTN Create account', 'link: Already have an account?']],
    ['States', ['closed notice · validation · 409']],
  ]},
  { name: 'Plans (public)', tabs: [], sections: [
    ['3 plan cards', ['Free · Pro (Most popular) · Business', 'price · features', 'BTN Continue with Free · Contact sales']],
    ['Header', ['Back to workspace / Log in']],
  ]},
  { name: 'Welcome (public)', tabs: [], sections: [['Card', ['onboarding title/body', 'Workspace name', 'BTN Create workspace']]] },
  { name: 'Home', tabs: [], tabsInfo: 'Home', sections: [
    ['KPIs', ['6 cards: Total · Posted · Pending · WIP · Views · Likes']],
    ['Live section', ['Synced {time} · BTN Sync now', 'account chips (Meta/IG)', 'posts grid (likes/comments/shares/views)']],
    ['Active campaigns', ['alerts · table (Campaign/Status/Spend) · BTN Open Ads']],
    ['Insights', ['Planned vs posted chart · By platform · By status · Top 5 (+Edit)']],
    ['States', ['loading · empty · error+Try · reconnect (Reconnect Meta) · read-only']],
  ]},
  { name: 'Plan — Monthly', tabs: ['Monthly', 'Calendar', 'Ideas', 'Hashtags'], sections: [
    ['Header', ['N topics · M today · BTN + New topic']],
    ['Master table', ['search · sortable headers · select checkbox · StatusChip']],
    ['Row editor', ['Topic · Pillar · Format · Goal · Date · Time · Status', 'Hook · Caption · CTA · Hashtag group', 'StatusFunnel · platform chips · CopyBar']],
    ['Buttons', ['BTN Save row · Duplicate · Delete · Lock/Unlock · Request edit']],
    ['States', ['loading · empty · error · busy · validation · conflict 409 · locked-by-other · no-perm']],
  ]},
  { name: 'Plan — Calendar', tabs: ['Monthly', 'Calendar', 'Ideas', 'Hashtags'], sections: [
    ['Controls', ['week-start (Sunday/Monday)', 'filters: Pillar · Platform · Format · Status', 'toggles: pillar · platform · status']],
    ['Grid', ['CalendarGrid WEEK 1–6']],
  ]},
  { name: 'Plan — Ideas', tabs: ['Monthly', 'Calendar', 'Ideas', 'Hashtags'], sections: [
    ['Table', ['Topic · Format · Idea · Link · Done · BTN Promote']],
    ['Capture', ['Topic · Format · Promote to month · Idea · Link · BTN Add idea']],
  ]},
  { name: 'Plan — Hashtags', tabs: ['Monthly', 'Calendar', 'Ideas', 'Hashtags'], sections: [
    ['Groups', ['HashtagGroups (group + add tag)', 'BTN Try again']],
  ]},
  { name: 'Content — Studio', tabs: ['Studio', 'Feed preview'], sections: [
    ['List', ['search · status tabs (All/Draft/Review/Scheduled/Published/Archived)', 'items · BTN + New']],
    ['Editor', ['title · status chip · save-state', 'fields: slug · type · summary · body(toolbar) · tags · hero · SEO']],
    ['Buttons', ['Save · Publish▾ (Publish now/Schedule/Unpublish) · Review · Duplicate · History · Archive · Delete']],
    ['States', ['loading · empty · error · saving/saved/unsaved · conflict 409 · not-logged-in']],
  ]},
  { name: 'Content — Feed preview', tabs: ['Studio', 'Feed preview'], sections: [
    ['Controls', ['SocialMediaBar · platform tabs · Show up to · Size select']],
    ['Grid', ['canvas meta line · FeedGrid 3×3']],
  ]},
  { name: 'Promote — Campaigns', tabs: ['Campaigns', 'Meta Ads'], sections: [
    ['List', ['name · status · dates · N linked · budget']],
    ['Editor', ['name · status · objective · start/end · platforms · pillars · budget · metric · target · hashtags · owner · notes']],
    ['Schedule + link', ['schedule table · link content (checkbox)']],
    ['Buttons', ['New campaign · Save · Delete']],
  ]},
  { name: 'Promote — Meta Ads', tabs: ['Campaigns', 'Meta Ads'], sections: [
    ['Header', ['Synced {time} · opt-in (Allow/Turn off) · BTN Sync ads']],
    ['Summary', ['account picker · KPI tiles (Spend/Results/CTR/CPC) · BTN Boost']],
    ['Table', ['Campaign · Objective · Status · Budget · Spend · Results · CTR · CPC · Pause/Resume']],
    ['Detail + dialogs', ['Settings · Ad sets/Ads · Edit budget · Duplicate · audit', 'dialogs: budget · opt-in · boost']],
  ]},
  { name: 'Analyze — Performance', tabs: ['Performance', 'Finance'], sections: [
    ['Import', ['Platform · Month · BTN Import · result']],
    ['Goals', ['table: Platform · Start · Goal · Now · Delta']],
    ['Charts', ['Views by month · Per-post +7 table · LiveSection(table)']],
  ]},
  { name: 'Analyze — Finance', tabs: ['Performance', 'Finance'], sections: [
    ['Summary', ['BudgetCards (Income/Expense/Balance)']],
    ['Charts', ['Income by month · Expense by month']],
    ['Transactions', ['TxnTable']],
    ['Log', ['Date · Amount · Kind · Sub-category · BTN Add transaction']],
  ]},
  { name: 'Settings — Brand', tabs: ['Brand', 'Workspace', 'Members', 'Connections'], sections: [
    ['Identity', ['Channel · Positioning · Slogan · Audience · Voice · Do/Do not (input+Add)']],
    ['Logos ×3', ['preview · Upload image · URL · remove · Resize']],
    ['Palette', ['live strip · 4 tiles (color+role+★+sample) · hex · Copy']],
    ['Typography', ['preview · imported (pick/Remove) · suggested · Import font']],
    ['Style', ['radius · fill · stroke (range+number) · shadows (None/Soft/Strong)']],
    ['Export / Moodboard', ['Copy CSS · Download/Upload JSON · moodboard grid (+Add images)']],
  ]},
  { name: 'Settings — Workspace', tabs: ['Brand', 'Workspace', 'Members', 'Connections'], sections: [
    ['Config', ['Year · Owner · Workspace name']],
    ['Options', ['5 lists (Pillars/Formats/Goals/Statuses/Platforms) + Add']],
    ['Users / Accounts', ['users (name/role/×) + Add', 'accounts (name/email/sessions/×)', 'BTN Create sign-in account → temp password + Copy']],
    ['Permissions', ['Require login · Roles matrix + Add role + Save permissions']],
  ]},
  { name: 'Settings — Members', tabs: ['Brand', 'Workspace', 'Members', 'Connections'], sections: [
    ['Add', ['email + BTN Add']],
    ['List', ['name · email · role chip · Owner badge · You chip · BTN Remove']],
    ['States', ['not-owner notice · empty · added/removed notice']],
  ]},
  { name: 'Settings — Connections', tabs: ['Brand', 'Workspace', 'Members', 'Connections'], sections: [
    ['Platforms', ['rows (status dot · name · connect/manage) → PlatformLogin']],
    ['States', ['disconnected · connected · error (reconnect) · connecting/syncing']],
  ]},
  { name: 'Read — index (public)', tabs: [], sections: [['Header', ['brand · BTN All posts · BTN Back to app']], ['List', ['published list (title · excerpt · author · date)']], ['States', ['loading · empty']]] },
  { name: 'Read — article (public)', tabs: [], sections: [['Article', ['meta (author/date) · title · excerpt · hero · markdown body · tags']], ['States', ['loading · 404 not found']]] },
  { name: '404', tabs: [], sections: [['Card', ['title · message', 'BTN Back to the first screen · Open the planner']]] },
]

const MOBILE = { w: 430, h: 932, m: 30 }

function ctaFor(name) {
  const n = name.toLowerCase()
  if (n.includes('login')) return 'LOG IN'
  if (n.includes('register')) return 'CREATE ACCOUNT'
  if (n.includes('plans')) return 'CONTINUE WITH FREE'
  if (n.includes('welcome')) return 'CREATE WORKSPACE'
  if (n.includes('home')) return 'SYNC NOW'
  if (n.startsWith('plan')) return 'SAVE ROW'
  if (n.includes('content')) return 'PUBLISH'
  if (n.includes('promote')) return 'SAVE'
  if (n.includes('analyze')) return 'IMPORT'
  if (n.includes('settings')) return 'SAVE'
  if (n.includes('read')) return 'ALL POSTS'
  if (n.includes('404')) return 'BACK TO APP'
  return 'NEXT'
}

// Minimal screen card: serif uppercase title · label + underline rows · black pill CTA.
function drawScreen(page, s, ox, oy) {
  const { w: W, h: H, m } = MOBILE
  const f = frame(s.name, W, H, C.paper)
  f.strokes = [solid(C.line)]; f.strokeWeight = 1
  page.appendChild(f); f.x = ox; f.y = oy

  // title — serif, uppercase, centered
  const pretty = s.name.replace(/\s*\(public\)/i, '').replace(/\s*—\s*/g, '\n').toUpperCase()
  const t = text(pretty, 40, 'Bold', C.ink)
  t.fontName = { family: SERIF, style: 'Bold' }
  t.textAlignHorizontal = 'CENTER'
  t.textAutoResize = 'HEIGHT'
  t.resize(W - m * 2, t.height)
  place(f, t, m, 44)

  // tabs line (faint, centered)
  if (s.tabs && s.tabs.length) {
    const tb = text(s.tabs.join('   ·   ').toUpperCase(), 10, 'Semi Bold', C.faint)
    tb.textAlignHorizontal = 'CENTER'; tb.textAutoResize = 'HEIGHT'; tb.resize(W - m * 2, tb.height)
    place(f, tb, m, 44 + t.height + 12)
  }

  // content rows — label + underline (label near the middle, like the reference)
  const items = (s.sections[0] ? s.sections[0][1] : []).slice(0, 5)
  let cy = 320
  for (const it of items) {
    const label = text(String(it), 15, 'Regular', C.body)
    label.textAutoResize = 'HEIGHT'; label.resize(W - m * 2, label.height)
    place(f, label, m, cy)
    place(f, rect(W - m * 2, 1.5, C.ink), m, cy + 34)
    cy += 72
  }

  // bottom pill CTA (rounded — this card style intentionally overrides the square chrome)
  const bw = W - m * 2, bh = 56, by = H - bh - 40
  const pill = rect(bw, bh, C.ink); pill.cornerRadius = bh / 2
  place(f, pill, m, by)
  const cta = text(ctaFor(s.name), 14, 'Semi Bold', C.paper)
  cta.x = m + (bw - cta.width) / 2; cta.y = by + (bh - cta.height) / 2
  f.appendChild(cta)

  return f
}

function buildScreens(page) {
  const title = text('Huuk — Screens', 30, 'Bold', C.ink)
  page.appendChild(title); title.x = 0; title.y = -140
  const cols = 4, gapX = 40, gapY = 60
  const nodes = SCREENS.map((s) => drawScreen(page, s, 0, 0))
  nodes.forEach((n, idx) => {
    const row = Math.floor(idx / cols), col = idx % cols
    n.x = col * (MOBILE.w + gapX)
    n.y = row * (MOBILE.h + gapY) + 80
  })
}

// =========================================================
// 3) OVERLAYS — every sheet/popup/modal/drawer
// =========================================================
const OVERS = [
  ['Account menu', 'Popup', ['ProfilePanel (name + Save)', 'Social connect rows', 'Workspace switcher (list + Create/Rename/Delete)', 'Workspace settings · Guide · Change password · Sign out everywhere · Log out']],
  ['Settings sheet', 'Dialog', ['config · options · permissions · accounts (เหมือน Settings › Workspace)']],
  ['Login modal', 'Modal', ['mode login/register · Name? · Email · Password · Cancel · submit']],
  ['Change password modal', 'Modal', ['Current · New · Confirm · Cancel · Confirm/Close · mismatch']],
  ['Protected cell modal', 'Modal', ['title · message · ไม่ต้องแสดง 5 นาที · Cancel · Confirm']],
  ['Platform connect', 'Full', ['account/token/expires/last sync/media · Scopes', 'Open profile · Reconnect · View live · Sync now · Disconnect · ×', 'not-connected: profile URL/@handle + Connect']],
  ['OAuth page picker', 'Modal', ['title · hint · page list (Connect this Page) · Close']],
  ['Guide', 'Modal', ['guide content · Close']],
  ['Date picker popup', 'Popup', ['date grid · Select date · Clear · Today']],
  ['Ads · Edit budget', 'Modal', ['daily budget input · Save · Cancel']],
  ['Ads · opt-in confirm', 'Modal', ['confirm text · Confirm · Cancel']],
  ['Ads · Boost a post', 'Modal', ['Name · Objective · Daily budget · Days · Countries · Post · Create paused campaign · Cancel']],
  ['Publish menu', 'Popup', ['Publish now · schedule datetime · Schedule · Unpublish']],
  ['Revisions drawer', 'Drawer', ['version list (#n · note · author · time) · preview · Restore this version · Close']],
]

function drawOverlay(page, o, ox, oy) {
  const { w: W, h: H, m } = MOBILE
  const f = frame(o[0], W, H, C.paper)
  f.strokes = [solid(C.line)]; f.strokeWeight = 1
  page.appendChild(f); f.x = ox; f.y = oy

  const t = text(o[0].toUpperCase(), 30, 'Bold', C.ink)
  t.fontName = { family: SERIF, style: 'Bold' }
  t.textAlignHorizontal = 'CENTER'; t.textAutoResize = 'HEIGHT'; t.resize(W - m * 2, t.height)
  place(f, t, m, 44)

  const kind = text(o[1].toUpperCase(), 10, 'Semi Bold', C.faint)
  kind.textAlignHorizontal = 'CENTER'; kind.textAutoResize = 'HEIGHT'; kind.resize(W - m * 2, kind.height)
  place(f, kind, m, 44 + t.height + 12)

  let cy = 320
  for (const it of o[2].slice(0, 5)) {
    const lab = text(String(it), 14, 'Regular', C.body)
    lab.textAutoResize = 'HEIGHT'; lab.resize(W - m * 2, lab.height)
    place(f, lab, m, cy)
    place(f, rect(W - m * 2, 1.5, C.ink), m, cy + 30)
    cy += 68
  }

  const bw = W - m * 2, bh = 56, by = H - bh - 40
  const pill = rect(bw, bh, C.ink); pill.cornerRadius = bh / 2
  place(f, pill, m, by)
  const c = text('CONFIRM', 14, 'Semi Bold', C.paper)
  c.x = m + (bw - c.width) / 2; c.y = by + (bh - c.height) / 2
  f.appendChild(c)
  return f
}

function buildOverlays(page) {
  const title = text('Huuk — Overlays', 30, 'Bold', C.ink)
  page.appendChild(title); title.x = 0; title.y = -140
  const cols = 4, gapX = 40, gapY = 60
  const nodes = OVERS.map((o) => drawOverlay(page, o, 0, 0))
  nodes.forEach((n, idx) => {
    const row = Math.floor(idx / cols), col = idx % cols
    n.x = col * (MOBILE.w + gapX)
    n.y = row * (MOBILE.h + gapY) + 80
  })
}

// =========================================================
// 4) STATES — per-screen state matrix
// =========================================================
const STATES = [
  ['Home', ['loading', 'data', 'empty', 'error', 'read-only', 'reconnect']],
  ['Plan (Monthly/Calendar/Ideas/Hashtags)', ['loading', 'data', 'empty', 'error', 'busy', 'validation', 'conflict 409', 'locked-by-other', 'no-perm']],
  ['Content (Studio/Feed)', ['loading', 'data', 'empty', 'error', 'saving/saved/unsaved', 'conflict 409', 'status ทุกแบบ', 'not-logged-in']],
  ['Promote (Campaigns/Ads)', ['loading', 'data', 'empty', 'error', 'read-only', 'reconnect', 'busy', 'validation']],
  ['Analyze (Performance/Finance)', ['loading', 'data', 'empty', 'error', 'import pending/success/error', 'no-perm']],
  ['Settings (Brand/Workspace/Members/Connections)', ['loading', 'data', 'error', 'no-perm/read-only', 'busy', 'validation', 'reconnect', 'onboarding']],
  ['Welcome', ['loading', 'idle', 'submitting', 'validation', 'error', 'success']],
  ['Login', ['idle', 'submitting', '401', '429', 'success']],
  ['Register', ['closed', 'idle', 'submitting', 'validation', '409', 'success']],
  ['Plans', ['idle', 'busy', 'current', 'error']],
  ['Read', ['loading', 'data', 'empty', '404 not-found']],
  ['404', ['static']],
]

function buildStates(page) {
  const W = 1100, pad = 40
  let y = 120
  const rowH = []
  for (const [name, list] of STATES) { const rows = Math.ceil(list.length / 6); y += 34 + rows * 30 + 14 }
  const f = frame('Screen States', W, y + 20, C.paper); f.strokes = [solid(C.line)]
  page.appendChild(f); f.x = 0; f.y = 0
  f.appendChild(place(f, text('Huuk — Screen States', 28, 'Bold', C.ink), pad, 44))
  f.appendChild(place(f, text('ทุก state ต่อหน้าจอ (สำหรับ wireframe)', 12, 'Regular', C.muted), pad, 84))
  let cy = 120
  for (const [name, list] of STATES) {
    f.appendChild(place(f, text(name, 13, 'Semi Bold', C.ink), pad, cy))
    cy += 24
    list.forEach((st, i) => {
      const col = i % 6, row = Math.floor(i / 6)
      const x = pad + col * 168, yy = cy + row * 30
      f.appendChild(place(f, rect(156, 22, C.wash, C.line, 1), x, yy))
      f.appendChild(place(f, text(st, 10, 'Regular', C.body), x + 8, yy + 5))
    })
    cy += Math.ceil(list.length / 6) * 30 + 14
  }
}

main()
