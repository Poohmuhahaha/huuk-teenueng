// Brand-driven theming ("CI"): the palette and fonts saved in Brand Identity
// are applied to the whole site as CSS custom properties on :root.
// Invalid values are ignored, and an empty brand restores the default theme.
import type { Brand, FontAsset } from '@/mock/db'

const HEX = /^#(?:[0-9a-f]{3}|[0-9a-f]{6})$/i
const FONT_NAME = /^[a-zA-Z0-9][a-zA-Z0-9 '\-]{0,40}$/

const ACCENT_VARS = [
  '--accent', '--accent-hover', '--accent-ink', '--accent-wash',
  '--accent-line', '--focus', '--selected',
] as const

/** Palette slots exposed for charts/accents (`--brand-1` … `--brand-4`). */
const BRAND_VARS = ['--brand-1', '--brand-2', '--brand-3', '--brand-4'] as const

/** Shadow tiers (the default `soft` tier is the CSS baseline: remove the overrides). */
const SHADOW_NONE: [string, string][] = [
  ['--shadow-xs', 'none'],
  ['--shadow-sm', 'none'],
  ['--shadow-md', 'none'],
  ['--shadow-lg', 'none'],
]
const SHADOW_STRONG: [string, string][] = [
  ['--shadow-xs', '0 1px 3px rgba(15, 23, 42, 0.10)'],
  ['--shadow-sm', '0 2px 6px rgba(15, 23, 42, 0.12), 0 1px 3px rgba(15, 23, 42, 0.08)'],
  ['--shadow-md', '0 8px 26px rgba(15, 23, 42, 0.14)'],
  ['--shadow-lg', '0 28px 70px rgba(15, 23, 42, 0.24)'],
]

/** The 0–24 slider maps to the three radius tiers. */
function radiusOf(value: number, scale: number): string {
  if (value <= 0) return '0px'
  return `${Math.round(value * scale)}px`
}

export interface BrandStyleSummary {
  radius: number
  fillOpacity: number
  strokeWidth: number
  shadow: string
}

/** Normalizes the brand's style tokens (older brands keep the defaults). */
export function brandStyle(brand: Brand | null | undefined): BrandStyleSummary {
  return {
    radius: Number.isFinite(brand?.radius) ? Math.min(24, Math.max(0, brand?.radius ?? 12)) : 12,
    fillOpacity:
      Number.isFinite(brand?.fillOpacity) && (brand?.fillOpacity ?? 100) >= 5
        ? Math.min(100, brand?.fillOpacity ?? 100)
        : 100,
    strokeWidth:
      Number.isFinite(brand?.strokeWidth) ? Math.min(3, Math.max(0, brand?.strokeWidth ?? 1)) : 1,
    shadow: ['none', 'soft', 'strong'].includes(brand?.shadow ?? '') ? (brand?.shadow as string) : 'soft',
  }
}

interface Rgb { r: number; g: number; b: number }

const WHITE: Rgb = { r: 255, g: 255, b: 255 }
const BLACK: Rgb = { r: 0, g: 0, b: 0 }

function toRgb(hex: string): Rgb | null {
  if (!HEX.test(hex)) return null
  let h = hex.slice(1)
  if (h.length === 3) h = h.split('').map((c) => c + c).join('')
  return {
    r: parseInt(h.slice(0, 2), 16),
    g: parseInt(h.slice(2, 4), 16),
    b: parseInt(h.slice(4, 6), 16),
  }
}

function toHex({ r, g, b }: Rgb): string {
  const c = (n: number): string => Math.round(Math.min(255, Math.max(0, n))).toString(16).padStart(2, '0')
  return `#${c(r)}${c(g)}${c(b)}`
}

function mix(a: Rgb, b: Rgb, t: number): Rgb {
  return { r: a.r + (b.r - a.r) * t, g: a.g + (b.g - a.g) * t, b: a.b + (b.b - a.b) * t }
}

/** Relative luminance (WCAG) — decides whether accent text should be black or white. */
function luminance({ r, g, b }: Rgb): number {
  const f = (v: number): number => {
    const s = v / 255
    return s <= 0.03928 ? s / 12.92 : ((s + 0.055) / 1.055) ** 2.4
  }
  return 0.2126 * f(r) + 0.7152 * f(g) + 0.0722 * f(b)
}

const FONT_STYLE_ID = 'huuk-brand-font'

/** Injects (or clears) the @font-face rule for an uploaded font asset. */
function ensureFontFace(asset: FontAsset | undefined): void {
  const existing = document.getElementById(FONT_STYLE_ID)
  if (!asset) {
    existing?.remove()
    return
  }
  const ext = asset.name.split('.').pop()?.toLowerCase() ?? ''
  const format = ext === 'woff2' ? 'woff2' : ext === 'woff' ? 'woff' : ext === 'ttf' ? 'truetype' : 'opentype'
  const css = `@font-face{font-family:"${asset.family}";src:url("${asset.url}") format("${format}");font-display:swap}`
  if (existing) {
    if (existing.textContent !== css) existing.textContent = css
    return
  }
  const style = document.createElement('style')
  style.id = FONT_STYLE_ID
  style.textContent = css
  document.head.appendChild(style)
}

/** Applies a brand's palette/fonts to `root` (defaults to <html>). */
export function applyBrandTheme(
  brand: Brand | null | undefined,
  assets: FontAsset[] = [],
  root: HTMLElement = document.documentElement,
): void {
  const style = root.style
  const palette = (brand?.palette ?? []).map((c) => c.trim()).filter((c) => HEX.test(c))
  const fonts = (brand?.fonts ?? []).map((f) => f.trim()).filter((f) => FONT_NAME.test(f))

  const primary = palette.length ? toRgb(palette[0]) : null
  if (primary) {
    style.setProperty('--accent', toHex(primary))
    style.setProperty('--accent-hover', toHex(mix(primary, BLACK, 0.14)))
    style.setProperty('--accent-ink', luminance(primary) > 0.55 ? toHex(BLACK) : toHex(WHITE))
    style.setProperty('--accent-wash', toHex(mix(primary, WHITE, 0.92)))
    style.setProperty('--accent-line', toHex(mix(primary, WHITE, 0.72)))
    style.setProperty('--focus', toHex(mix(primary, WHITE, 0.25)))
    style.setProperty('--selected', toHex(mix(primary, WHITE, 0.92)))
  } else {
    ACCENT_VARS.forEach((v) => style.removeProperty(v))
  }

  // Every palette slot is available as `--brand-N` (bars cycle through them);
  // slot 1 doubles as the accent above.
  BRAND_VARS.forEach((v, i) => {
    const hex = palette[i] ? toHex(toRgb(palette[i]) as Rgb) : ''
    if (hex) style.setProperty(v, hex)
    else style.removeProperty(v)
  })

  // Style tokens: corner radius, accent fill strength, stroke, shadows.
  const tokens = brandStyle(brand)
  style.setProperty('--radius', `${tokens.radius}px`)
  style.setProperty('--radius-sm', radiusOf(tokens.radius, 0.66))
  style.setProperty('--radius-lg', radiusOf(tokens.radius, 1.33))
  style.setProperty('--stroke-w', `${tokens.strokeWidth}px`)
  const o = tokens.fillOpacity / 100
  const strong = mix(primary ?? WHITE, WHITE, (1 - o) * 0.85)
  style.setProperty('--accent-strong', toHex(strong))
  style.setProperty(
    '--accent-strong-ink',
    luminance(strong) > 0.55 ? toHex(BLACK) : toHex(WHITE),
  )
  const shadows = tokens.shadow === 'none' ? SHADOW_NONE : tokens.shadow === 'strong' ? SHADOW_STRONG : []
  shadows.forEach(([name, value]) => style.setProperty(name, value))
  if (tokens.shadow === 'soft') {
    for (const [name] of SHADOW_STRONG) style.removeProperty(name)
  }

  const requested = fonts[0]
  const asset = requested
    ? assets.find((a) => a.family.toLowerCase() === requested.toLowerCase())
    : undefined
  ensureFontFace(asset)
  if (requested) {
    style.setProperty('--font', `"${asset ? asset.family : requested}", "Noto Sans Thai", "Inter", "Segoe UI", system-ui, sans-serif`)
  } else {
    style.removeProperty('--font')
  }
}
