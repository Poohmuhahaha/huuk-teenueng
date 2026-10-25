<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { useRoute } from 'vue-router'
import {
  useBrand, useBrandFonts, useDeleteFont, useSaveBrand, useUploadFont, useUploadBrandImage,
  usePermission,
} from '@/core/queries'
import { t } from '@/core/i18n'
import InfoTip from '@/components/ui/InfoTip.vue'
import DockBar from '@/components/ui/DockBar.vue'
import type { Brand, FontAsset } from '@/mock/db'
import {
  base64ToBytes, checkImageBounds, fitWithin, imageDimensions,
  resizeToMaxSide, resizeUrlToMaxSide, IMAGE_MAX_SIDE,
} from '@/core/images'
import type { BrandImageKind } from '@/core/images'
import api from '@/api/index'

import BrandIdentityStep from '@/components/brand/BrandIdentityStep.vue'
import BrandVoiceStep from '@/components/brand/BrandVoiceStep.vue'
import BrandLogosStep from '@/components/brand/BrandLogosStep.vue'
import BrandPaletteStep from '@/components/brand/BrandPaletteStep.vue'
import BrandTypographyStep from '@/components/brand/BrandTypographyStep.vue'
import BrandStyleStep from '@/components/brand/BrandStyleStep.vue'
import BrandMoodboardStep from '@/components/brand/BrandMoodboardStep.vue'
import BrandFinishedStep from '@/components/brand/BrandFinishedStep.vue'
import BrandPdfView from '@/components/brand/BrandPdfView.vue'
import BrandBgPopover from '@/components/brand/BrandBgPopover.vue'

const { data: brand, isPending, isError, error: loadError, refetch } = useBrand()
const saveBrand = useSaveBrand()
const { can } = usePermission()
const canWrite = computed(() => can('brand.write'))
const route = useRoute()
const onboarding = computed(() => route.query.onboarding === '1')
const doDraft = ref('')
const dontDraft = ref('')
const paletteDraft = ref<string[]>(['', '', '', ''])

// Palette slots with the role they play and where the color shows up.
const PALETTE_SLOTS = ['1', '2', '3', '4'] as const
const PALETTE_ROLES: Record<(typeof PALETTE_SLOTS)[number], string> = {
  '1': 'brand.palettePrimary',
  '2': 'brand.paletteSecondary',
  '3': 'brand.paletteTertiary',
  '4': 'brand.paletteSupporting',
}
const PALETTE_USAGE: Record<(typeof PALETTE_SLOTS)[number], string> = {
  '1': 'brand.paletteUsagePrimary',
  '2': 'brand.paletteUsageSecond',
  '3': 'brand.paletteUsageThird',
  '4': 'brand.paletteUsageFourth',
}
const moodboardDraft = ref('')
// ---- style tokens (Figma-flavored design controls) ----
const styleDraft = ref({ radius: 12, fillOpacity: 100, strokeWidth: 0, shadow: 'soft' })
const styleNotice = ref('')
const styleErr = ref('')
const styleBusy = ref(false)

// ---- step wizard (minimal card flow, reference layout) ----
const STEPS = [
  { key: 'identity', title: 'brand.stepIdentity' },
  { key: 'voice', title: 'brand.stepVoice' },
  { key: 'logos', title: 'brand.logos' },
  { key: 'palette', title: 'brand.paletteLabel' },
  { key: 'type', title: 'brand.typography' },
  { key: 'style', title: 'brand.styleTitle' },
  { key: 'mood', title: 'brand.moodboard' },
] as const
const step = ref(0)
const finished = ref(false)

function restartStep(): void {
  finished.value = false
  step.value = 0
}

// ---- Brand Card Background Customization (Desktop & Mobile Independent) ----
const defaultBrandBg = '/brand-default-bg.jpg'
const customBgDesktop = ref<string>(
  typeof window !== 'undefined'
    ? localStorage.getItem('huuk_brand_bg_desktop') || localStorage.getItem('huuk_brand_bg') || defaultBrandBg
    : defaultBrandBg,
)
const customBgMobile = ref<string>(
  typeof window !== 'undefined'
    ? localStorage.getItem('huuk_brand_bg_mobile') || ''
    : '',
)

const activeBrandBg = computed(() => customBgDesktop.value || defaultBrandBg)

const bgTargetDevice = ref<'desktop' | 'mobile'>('desktop')
const currentActiveBg = computed(() =>
  bgTargetDevice.value === 'desktop' ? customBgDesktop.value : customBgMobile.value,
)

const hasCustomBg = computed(() => Boolean(customBgDesktop.value || customBgMobile.value))

const customBgStyle = computed(() => {
  const d = customBgDesktop.value || customBgMobile.value || defaultBrandBg
  const m = customBgMobile.value || customBgDesktop.value || defaultBrandBg
  return {
    '--brand-bg-desktop': `url("${d}")`,
    '--brand-bg-mobile': `url("${m}")`,
  } as Record<string, string>
})

const bgMenuOpen = ref(false)
const bgUrlDraft = ref('')
const bgToolRef = ref<HTMLElement | null>(null)

function onPickBgFile(event: Event): void {
  const input = event.target as HTMLInputElement
  const file = input.files?.[0]
  input.value = ''
  if (!file) return
  const reader = new FileReader()
  reader.onload = () => {
    const result = String(reader.result || '')
    if (bgTargetDevice.value === 'desktop') {
      customBgDesktop.value = result
      if (typeof window !== 'undefined') {
        try {
          localStorage.setItem('huuk_brand_bg_desktop', result)
          localStorage.setItem('huuk_brand_bg', result)
        } catch (err) {
          console.warn('Could not cache background to localStorage', err)
        }
      }
    } else {
      customBgMobile.value = result
      if (typeof window !== 'undefined') {
        try {
          localStorage.setItem('huuk_brand_bg_mobile', result)
        } catch (err) {
          console.warn('Could not cache mobile background to localStorage', err)
        }
      }
    }
  }
  reader.readAsDataURL(file)
}

function applyBgUrl(urlVal?: string): void {
  const url = (typeof urlVal === 'string' ? urlVal : bgUrlDraft.value).trim()
  if (!url) return
  if (bgTargetDevice.value === 'desktop') {
    customBgDesktop.value = url
    if (typeof window !== 'undefined') {
      try {
        localStorage.setItem('huuk_brand_bg_desktop', url)
        localStorage.setItem('huuk_brand_bg', url)
      } catch (err) {
        console.warn('Could not cache background to localStorage', err)
      }
    }
  } else {
    customBgMobile.value = url
    if (typeof window !== 'undefined') {
      try {
        localStorage.setItem('huuk_brand_bg_mobile', url)
      } catch (err) {
        console.warn('Could not cache mobile background to localStorage', err)
      }
    }
  }
  bgUrlDraft.value = ''
}

function clearBg(): void {
  if (bgTargetDevice.value === 'desktop') {
    customBgDesktop.value = ''
    if (typeof window !== 'undefined') {
      try {
        localStorage.removeItem('huuk_brand_bg_desktop')
        if (!customBgMobile.value) localStorage.removeItem('huuk_brand_bg')
      } catch {}
    }
  } else {
    customBgMobile.value = ''
    if (typeof window !== 'undefined') {
      try {
        localStorage.removeItem('huuk_brand_bg_mobile')
      } catch {}
    }
  }
  bgUrlDraft.value = ''
}

function onGlobalClick(e: MouseEvent): void {
  const target = e.target as Node
  if (bgMenuOpen.value && bgToolRef.value && !bgToolRef.value.contains(target)) {
    bgMenuOpen.value = false
  }
}

function exportPdf(): void {
  if (typeof window !== 'undefined') {
    window.print()
  }
}

onMounted(() => {
  if (typeof window !== 'undefined') {
    window.addEventListener('click', onGlobalClick)
  }
})

onBeforeUnmount(() => {
  if (typeof window !== 'undefined') {
    window.removeEventListener('click', onGlobalClick)
  }
})

watch(
  brand,
  (b) => {
    if (!b) return
    styleDraft.value = {
      radius: Number.isFinite(b.radius) ? b.radius : 12,
      fillOpacity: b.fillOpacity >= 5 ? b.fillOpacity : 100,
      strokeWidth: Number.isFinite(b.strokeWidth) ? b.strokeWidth : 1,
      shadow: ['none', 'soft', 'strong'].includes(b.shadow) ? b.shadow : 'soft',
    }
  },
  { immediate: true },
)

async function saveStyle(): Promise<void> {
  if (!canWrite.value || busy.value || !brand.value) return
  styleNotice.value = ''
  styleErr.value = ''
  try {
    await saveBrand.mutateAsync({ ...styleDraft.value })
  } catch (e) {
    styleErr.value = e instanceof Error ? e.message : String(e)
  }
}

/** The brand kit as portable CSS (like copying styles out of Figma). */
function styleCss(): string {
  const b = brand.value
  const lines = [':root {']
  const palette = b?.palette ?? []
  const slot = (i: number): string => palette[i] ?? ''
  lines.push(`  --accent: ${slot(0)};`)
  lines.push(`  --brand-1: ${slot(0)};`)
  lines.push(`  --brand-2: ${slot(1)};`)
  lines.push(`  --brand-3: ${slot(2)};`)
  lines.push(`  --brand-4: ${slot(3)};`)
  const fonts = b?.fonts ?? []
  if (fonts[0]) lines.push(`  --font: "${fonts[0]}";`)
  lines.push(`  --radius: ${styleDraft.value.radius}px;`)
  lines.push(`  --accent-fill: ${styleDraft.value.fillOpacity}%;`)
  lines.push(`  --stroke-w: ${styleDraft.value.strokeWidth}px;`)
  lines.push(`  --shadow-depth: ${styleDraft.value.shadow};`)
  lines.push('}')
  return lines.join(String.fromCharCode(10))
}

async function copyStyleCss(): Promise<void> {
  styleNotice.value = ''
  styleErr.value = ''
  styleBusy.value = true
  try {
    const text = styleCss()
    if (navigator.clipboard?.writeText) {
      await navigator.clipboard.writeText(text)
    } else {
      const area = document.createElement('textarea')
      area.value = text
      document.body.appendChild(area)
      area.select()
      document.execCommand('copy')
      area.remove()
    }
    styleNotice.value = t('brand.styleCopied')
  } catch (e) {
    styleErr.value = e instanceof Error ? e.message : String(e)
  } finally {
    styleBusy.value = false
  }
}

function downloadBrandJson(): void {
  styleNotice.value = ''
  styleErr.value = ''
  const blob = new Blob([JSON.stringify(brand.value, null, 2)], { type: 'application/json' })
  const link = document.createElement('a')
  link.href = URL.createObjectURL(blob)
  link.download = 'brand-kit.json'
  document.body.appendChild(link)
  link.click()
  link.remove()
  styleNotice.value = t('brand.styleDownloaded')
}

/**
 * Only the known brand-kit keys survive an import, each one type-checked and
 * clamped, so a hand-edited or foreign JSON file can never break the site.
 */
function sanitizeBrandJson(raw: unknown): Partial<Brand> | null {
  if (!raw || typeof raw !== 'object' || Array.isArray(raw)) return null
  const src = raw as Record<string, unknown>
  const out: Partial<Brand> = {}
  for (const key of ['channel', 'positioning', 'slogan', 'audience', 'voice'] as const) {
    if (typeof src[key] === 'string') out[key] = src[key] as string
  }
  for (const key of ['dos', 'donts', 'palette', 'fonts', 'logos', 'moodboard'] as const) {
    const value = src[key]
    if (Array.isArray(value)) {
      out[key] = value.filter((item): item is string => typeof item === 'string' && item.trim().length > 0)
    }
  }
  const clamp = (value: unknown, min: number, max: number): number | undefined =>
    typeof value === 'number' && Number.isFinite(value)
      ? Math.min(max, Math.max(min, Math.round(value)))
      : undefined
  const radius = clamp(src.radius, 0, 24)
  if (radius !== undefined) out.radius = radius
  const fillOpacity = clamp(src.fillOpacity, 5, 100)
  if (fillOpacity !== undefined) out.fillOpacity = fillOpacity
  const strokeWidth = clamp(src.strokeWidth, 0, 3)
  if (strokeWidth !== undefined) out.strokeWidth = strokeWidth
  if (typeof src.shadow === 'string' && ['none', 'soft', 'strong'].includes(src.shadow)) {
    out.shadow = src.shadow
  }
  return Object.keys(out).length ? out : null
}

function readText(file: File): Promise<string> {
  return new Promise((resolve, reject) => {
    const reader = new FileReader()
    reader.onload = () => resolve(String(reader.result ?? ''))
    reader.onerror = () => reject(new Error(t('brand.styleImportInvalid')))
    reader.readAsText(file)
  })
}

async function importBrandJson(event: Event): Promise<void> {
  const input = event.target as HTMLInputElement
  const file = input.files?.[0]
  input.value = ''
  if (!file || !canWrite.value) return
  styleNotice.value = ''
  styleErr.value = ''
  styleBusy.value = true
  try {
    let parsed: unknown
    try {
      parsed = JSON.parse(await readText(file))
    } catch {
      throw new Error(t('brand.styleImportInvalid'))
    }
    const patch = sanitizeBrandJson(parsed)
    if (!patch) throw new Error(t('brand.styleImportInvalid'))
    await saveBrand.mutateAsync(patch)
    styleNotice.value = t('brand.styleImported')
  } catch (e) {
    styleErr.value = e instanceof Error ? e.message : String(e)
  } finally {
    styleBusy.value = false
  }
}

const copiedSlot = ref('')

/** Black or white text, whichever stays readable on this swatch. */
function readableOn(color: string): string {
  const hex = color.trim().replace('#', '')
  const full = hex.length === 3 ? hex.split('').map((c) => c + c).join('') : hex
  if (!/^[0-9a-fA-F]{6}$/.test(full)) return 'var(--ink)'
  const value = parseInt(full, 16)
  const [r, g, b] = [(value >> 16) & 255, (value >> 8) & 255, value & 255].map((channel) => {
    const s = channel / 255
    return s <= 0.03928 ? s / 12.92 : ((s + 0.055) / 1.055) ** 2.4
  })
  return 0.2126 * r + 0.7152 * g + 0.0722 * b > 0.45 ? '#171717' : '#ffffff'
}

function hexToRgb(hex: string): string {
  const clean = (hex || '').replace('#', '').trim()
  if (!clean) return 'rgb(0, 0, 0)'
  const full = clean.length === 3 ? clean.split('').map((c) => c + c).join('') : clean
  if (!/^[0-9a-fA-F]{6}$/.test(full)) return 'rgb(0, 0, 0)'
  const val = parseInt(full, 16)
  const r = (val >> 16) & 255
  const g = (val >> 8) & 255
  const b = val & 255
  return `rgb(${r}, ${g}, ${b})`
}

function brandInitials(name?: string): string {
  if (!name) return 'BK'
  const parts = name.trim().split(/\s+/)
  if (parts.length >= 2) {
    return (parts[0][0] + parts[1][0]).toUpperCase()
  }
  return name.slice(0, 2).toUpperCase()
}

async function copyHex(slot: string): Promise<void> {
  const value = paletteDraft.value[Number(slot) - 1] || ''
  if (!value) return
  try {
    if (navigator.clipboard?.writeText) {
      await navigator.clipboard.writeText(value)
    } else {
      const area = document.createElement('textarea')
      area.value = value
      document.body.appendChild(area)
      area.select()
      document.execCommand('copy')
      area.remove()
    }
    copiedSlot.value = slot
    window.setTimeout(() => {
      if (copiedSlot.value === slot) copiedSlot.value = ''
    }, 1600)
  } catch {
    // clipboard unavailable — the hex field still allows a manual copy
  }
}
const fontError = ref('')
const uploading = ref(false)
const imageError = ref('')
const imageNotice = ref('')
const uploadingSlot = ref('')

// Logo slots and the moodboard are image URLs stored in the brand kit; images
// are uploaded to the server (or pasted as https URLs).
const uploadImage = useUploadBrandImage()
const LOGO_LABELS = {
  0: 'brand.logoMain',
  1: 'brand.logoSecondary',
  2: 'brand.logoSocial',
} as Record<number, string>

const logoSlots = computed(() => {
  const rows = brand.value?.logos ?? []
  return [0, 1, 2].map((i) => ({ index: i, url: rows[i] ?? '' }))
})
const moodboard = computed(() => brand.value?.moodboard ?? [])

async function saveLogos(logos: string[]): Promise<void> {
  try {
    await saveBrand.mutateAsync({ logos })
  } catch (e) {
    imageError.value = e instanceof Error ? e.message : String(e)
  }
}

async function saveMoodboard(moodboardList: string[]): Promise<void> {
  try {
    await saveBrand.mutateAsync({ moodboard: moodboardList })
  } catch (e) {
    imageError.value = e instanceof Error ? e.message : String(e)
  }
}

/**
 * Uploads a picked file. Oversized images are downscaled to the 500 px cap
 * automatically (so nobody has to pre-resize a logo); the server still
 * enforces the hard bounds as a safety net.
 */
async function storeImage(kind: BrandImageKind, file: File): Promise<string> {
  const original = (await readFile(file)).split(',')[1] ?? ''
  const size = imageDimensions(base64ToBytes(original))
  if (!size) throw new Error(t('brand.imageUnreadable'))

  let data = original
  let name = file.name
  let resized = false
  const plan = fitWithin(size.width, size.height, IMAGE_MAX_SIDE)
  if (plan.needed) {
    const out = await resizeToMaxSide(file, file.name, IMAGE_MAX_SIDE)
    data = out.data
    name = out.name
    resized = out.resized
    if (!resized) {
      // No canvas support: fall back to the server's verdict.
      const boundsError = checkImageBounds(kind, size.width, size.height)
      if (boundsError) throw new Error(boundsError)
    }
  }

  const after = imageDimensions(base64ToBytes(data)) ?? size
  const boundsError = checkImageBounds(kind, after.width, after.height)
  if (boundsError) throw new Error(boundsError)
  const { url } = await uploadImage.mutateAsync({ name, data, kind })
  if (resized) imageNotice.value = t('brand.imageResized', { size: `${after.width}×${after.height}` })
  return url
}

/** "Fix size" for images stored before the cap: re-encode and replace them. */
async function fixStoredImage(kind: BrandImageKind, url: string): Promise<void> {
  if (!canWrite.value || !url.startsWith('/api/brand/images/')) return
  imageError.value = ''
  imageNotice.value = ''
  uploadingSlot.value = `fix-${url}`
  try {
    const out = await resizeUrlToMaxSide(url, IMAGE_MAX_SIDE)
    if (!out.resized) {
      imageNotice.value = t('brand.imageWithinBounds')
      return
    }
    const uploaded = await uploadImage.mutateAsync({ name: out.name, data: out.data, kind })
    if (kind === 'logo') {
      const next = [0, 1, 2].map((i) => logoSlots.value[i]?.url ?? '')
      const index = next.findIndex((u) => u === url)
      if (index >= 0) next[index] = uploaded.url
      await saveLogos(next)
    } else {
      await saveMoodboard(moodboard.value.map((u) => (u === url ? uploaded.url : u)))
    }
    // The oversized original is no longer referenced.
    const oldName = url.split('/').pop() ?? ''
    if (oldName) await api.deleteBrandImage(oldName).catch(() => undefined)
    imageNotice.value = t('brand.imageResized', {
      size: `${uploaded.width}×${uploaded.height}`,
    })
  } catch (e) {
    imageError.value = e instanceof Error ? e.message : String(e)
  } finally {
    uploadingSlot.value = ''
  }
}

/** True when a stored upload is larger than the cap (checked on render). */
const oversized = ref<Record<string, boolean>>({})
function noteRenderedSize(url: string, event: Event): void {
  if (!url.startsWith('/api/brand/images/')) return
  const img = event.target as HTMLImageElement
  const over = fitWithin(img.naturalWidth, img.naturalHeight, IMAGE_MAX_SIDE).needed
  if (over !== oversized.value[url]) oversized.value = { ...oversized.value, [url]: over }
}

async function pickLogo(index: number, event: Event): Promise<void> {
  const input = event.target as HTMLInputElement
  const file = input.files?.[0]
  input.value = ''
  if (!file || !canWrite.value) return
  imageError.value = ''
  uploadingSlot.value = `logo-${index}`
  try {
    const url = await storeImage('logo', file)
    const next = [0, 1, 2].map((i) => logoSlots.value[i]?.url ?? '')
    next[index] = url
    await saveLogos(next)
  } catch (e) {
    imageError.value = e instanceof Error ? e.message : String(e)
  } finally {
    uploadingSlot.value = ''
  }
}

async function setLogoUrl(index: number, event: Event): Promise<void> {
  const value = (event.target as HTMLInputElement).value.trim()
  if (!canWrite.value) return
  imageError.value = ''
  const next = [0, 1, 2].map((i) => logoSlots.value[i]?.url ?? '')
  next[index] = value
  await saveLogos(next)
}

async function clearLogo(index: number): Promise<void> {
  if (!canWrite.value) return
  const next = [0, 1, 2].map((i) => logoSlots.value[i]?.url ?? '')
  next[index] = ''
  await saveLogos(next)
}

async function addMoodboard(event: Event): Promise<void> {
  const input = event.target as HTMLInputElement
  const files = Array.from(input.files ?? [])
  input.value = ''
  if (!files.length || !canWrite.value) return
  imageError.value = ''
  uploadingSlot.value = 'moodboard'
  try {
    const urls: string[] = []
    for (const file of files) urls.push(await storeImage('moodboard', file))
    await saveMoodboard([...moodboard.value, ...urls].slice(0, 12))
  } catch (e) {
    imageError.value = e instanceof Error ? e.message : String(e)
  } finally {
    uploadingSlot.value = ''
  }
}

async function addMoodboardUrl(): Promise<void> {
  const value = moodboardDraft.value.trim()
  if (!canWrite.value || !value) return
  imageError.value = ''
  moodboardDraft.value = ''
  await saveMoodboard([...moodboard.value, value].slice(0, 12))
}

async function removeMoodboard(index: number): Promise<void> {
  if (!canWrite.value) return
  await saveMoodboard(moodboard.value.filter((_, i) => i !== index))
}

// Fonts installed on most devices; uploaded fonts are listed separately.
const suggestedFonts = [
  'Noto Sans Thai', 'Inter', 'Kanit', 'Sarabun', 'Prompt', 'IBM Plex Sans Thai', 'DejaVu Sans',
]

const { data: fontAssets } = useBrandFonts()
const uploadFont = useUploadFont()
const deleteFont = useDeleteFont()
const importedFonts = computed<FontAsset[]>(() => fontAssets.value ?? [])

watch(
  brand,
  (b) => {
    if (!b) return
    paletteDraft.value = [0, 1, 2, 3].map((i) => b.palette[i] ?? '')
  },
  { immediate: true },
)
const busy = computed(() => saveBrand.isPending.value)
const writeError = computed(() => {
  const e = saveBrand.error.value
  return e ? (e instanceof Error ? e.message : String(e)) : ''
})

async function save(key: 'channel' | 'positioning' | 'slogan' | 'audience' | 'voice', e: Event): Promise<void> {
  if (!canWrite.value || busy.value) return
  try {
    await saveBrand.mutateAsync({ [key]: (e.target as HTMLInputElement).value })
  } catch {
    // error surfaced through writeError
  }
}

async function savePalette(): Promise<void> {
  if (!canWrite.value || busy.value || !brand.value) return
  const palette = paletteDraft.value.map((c) => c.trim()).filter(Boolean)
  try {
    await saveBrand.mutateAsync({ palette })
  } catch {
    // error surfaced through writeError
  }
}

async function selectFont(family: string): Promise<void> {
  if (!canWrite.value || busy.value || brand.value?.fonts[0] === family) return
  fontError.value = ''
  try {
    await saveBrand.mutateAsync({ fonts: [family] })
  } catch (e) {
    fontError.value = e instanceof Error ? e.message : String(e)
  }
}

function readFile(file: File): Promise<string> {
  return new Promise((resolve, reject) => {
    const reader = new FileReader()
    reader.onload = () => resolve(String(reader.result ?? ''))
    reader.onerror = () => reject(new Error('could not read the file'))
    reader.readAsDataURL(file)
  })
}

async function importFonts(event: Event): Promise<void> {
  const input = event.target as HTMLInputElement
  const files = Array.from(input.files ?? [])
  input.value = ''
  if (!files.length || !canWrite.value) return
  fontError.value = ''
  uploading.value = true
  try {
    let last: string | null = null
    for (const file of files) {
      const base64 = (await readFile(file)).split(',')[1] ?? ''
      const list = await uploadFont.mutateAsync({ name: file.name, data: base64 })
      const family = list.find((f) => f.family && f.name.toLowerCase().startsWith(file.name.replace(/\.[^.]+$/, '').toLowerCase().replace(/[^a-z0-9]+/g, '-').replace(/^-+|-+$/g, '')))?.family
      last = family ?? list[list.length - 1]?.family ?? last
    }
    if (last) await selectFont(last)
  } catch (e) {
    fontError.value = e instanceof Error ? e.message : String(e)
  } finally {
    uploading.value = false
  }
}

async function removeFont(asset: FontAsset): Promise<void> {
  if (!canWrite.value) return
  if (typeof window !== 'undefined' && !window.confirm(`${t('brand.fontDelete')}: ${asset.family}?`)) return
  fontError.value = ''
  try {
    await deleteFont.mutateAsync(asset.name)
    if (brand.value?.fonts[0] === asset.family) await saveBrand.mutateAsync({ fonts: [] })
  } catch (e) {
    fontError.value = e instanceof Error ? e.message : String(e)
  }
}

async function addDo(val?: string): Promise<void> {
  const value = (typeof val === 'string' ? val : doDraft.value).trim()
  if (!canWrite.value || !brand.value || !value || busy.value) return
  try {
    await saveBrand.mutateAsync({ dos: [...brand.value.dos, value] })
    doDraft.value = ''
  } catch {
    // keep the draft so nothing typed is lost
  }
}

async function removeDo(index: number): Promise<void> {
  if (!canWrite.value || !brand.value || busy.value) return
  const next = [...brand.value.dos]
  next.splice(index, 1)
  await saveBrand.mutateAsync({ dos: next })
}

async function addDont(val?: string): Promise<void> {
  const value = (typeof val === 'string' ? val : dontDraft.value).trim()
  if (!canWrite.value || !brand.value || !value || busy.value) return
  try {
    await saveBrand.mutateAsync({ donts: [...brand.value.donts, value] })
    dontDraft.value = ''
  } catch {
    // keep the draft so nothing typed is lost
  }
}

async function removeDont(index: number): Promise<void> {
  if (!canWrite.value || !brand.value || busy.value) return
  const next = [...brand.value.donts]
  next.splice(index, 1)
  await saveBrand.mutateAsync({ donts: next })
}
</script>

<template>
  <div class="brandpage" :class="{ 'has-custom-bg': hasCustomBg && (step === 0 || step === 1 || finished) }" :style="customBgStyle">
    <div v-if="onboarding" class="brand-note">
      <strong>{{ t('brand.onboardingTitle') }}</strong>
      <InfoTip :text="t('brand.onboarding')" />
    </div>

    <div v-if="isPending" class="muted">Loading brand kit…</div>
    <div v-else-if="isError" class="card">
      <p class="muted">Could not load the brand kit: {{ loadError?.message }}</p>
      <button class="btn" @click="() => refetch()">Try again</button>
    </div>
    <template v-else-if="brand">
      <p v-if="writeError" class="autherr" role="alert">{{ writeError }}</p>

      <!-- 0 · identity -->
      <BrandIdentityStep
        v-show="step === 0 && !finished"
        :brand="brand"
        :can-write="canWrite"
        :busy="busy"
        @save="save"
      />

      <!-- 1 · voice -->
      <BrandVoiceStep
        v-show="step === 1 && !finished"
        :brand="brand"
        :can-write="canWrite"
        :busy="busy"
        :has-custom-bg="hasCustomBg"
        @save="save"
        @add-do="addDo"
        @remove-do="removeDo"
        @add-dont="addDont"
        @remove-dont="removeDont"
      />

      <!-- 2 · logos -->
      <BrandLogosStep
        v-show="step === 2 && !finished"
        :logo-slots="logoSlots"
        :can-write="canWrite"
        :busy="busy"
        :uploading-slot="uploadingSlot"
        :oversized="oversized"
        :logo-labels="LOGO_LABELS"
        @note-rendered-size="noteRenderedSize"
        @clear-logo="clearLogo"
        @pick-logo="pickLogo"
        @set-logo-url="setLogoUrl"
        @fix-stored-image="fixStoredImage"
      />

      <!-- 3 · palette -->
      <BrandPaletteStep
        v-show="step === 3 && !finished"
        :palette-draft="paletteDraft"
        :can-write="canWrite"
        :busy="busy"
        :copied-slot="copiedSlot"
        :readable-on="readableOn"
        :palette-slots="PALETTE_SLOTS"
        :palette-roles="PALETTE_ROLES"
        :palette-usage="PALETTE_USAGE"
        @update-slot="(i, val) => { paletteDraft[i] = val }"
        @save-palette="savePalette"
        @copy-hex="copyHex"
      />

      <!-- 4 · typography -->
      <BrandTypographyStep
        v-show="step === 4 && !finished"
        :brand="brand"
        :can-write="canWrite"
        :imported-fonts="importedFonts"
        :suggested-fonts="suggestedFonts"
        :uploading="uploading"
        :font-error="fontError"
        @select-font="selectFont"
        @remove-font="removeFont"
        @import-fonts="importFonts"
      />

      <!-- 5 · style -->
      <BrandStyleStep
        v-show="step === 5 && !finished"
        :style-draft="styleDraft"
        :can-write="canWrite"
        :busy="busy"
        :style-busy="styleBusy"
        :style-notice="styleNotice"
        :style-err="styleErr"
        @save-style="saveStyle"
        @set-shadow="(opt) => { styleDraft.shadow = opt; saveStyle() }"
        @copy-style-css="copyStyleCss"
        @download-brand-json="downloadBrandJson"
        @import-brand-json="importBrandJson"
      />

      <!-- 6 · moodboard -->
      <BrandMoodboardStep
        v-show="step === 6 && !finished"
        :moodboard="moodboard"
        :can-write="canWrite"
        :uploading-slot="uploadingSlot"
        :oversized="oversized"
        :moodboard-draft="moodboardDraft"
        :image-notice="imageNotice"
        :image-error="imageError"
        @update:moodboard-draft="(val) => { moodboardDraft = val }"
        @note-rendered-size="noteRenderedSize"
        @remove-moodboard="removeMoodboard"
        @add-moodboard="addMoodboard"
        @add-moodboard-url="addMoodboardUrl"
        @fix-stored-image="fixStoredImage"
      />

      <!-- Finished State -->
      <BrandFinishedStep
        v-show="finished"
        :brand="brand"
        @export-pdf="exportPdf"
        @restart-step="restartStep"
      />

      <!-- PRINTABLE BRAND GUIDELINE BOOK: 7 Chapters of the 16:9 Brand Card -->
      <BrandPdfView
        :brand="brand"
        :active-brand-bg="activeBrandBg"
        :custom-bg-style="customBgStyle"
        :logo-slots="logoSlots"
        :palette-draft="paletteDraft"
        :style-draft="styleDraft"
        :moodboard="moodboard"
        :logo-labels="LOGO_LABELS"
        :palette-slots="PALETTE_SLOTS"
        :palette-roles="PALETTE_ROLES"
        :palette-usage="PALETTE_USAGE"
        :readable-on="readableOn"
        :hex-to-rgb="hexToRgb"
        :brand-initials="brandInitials"
      />

      <DockBar label="Brand steps">
        <div class="brand-dock-row">
          <!-- Chapter Navigation Tabs/Buttons -->
          <nav class="brand-book-tabs" role="tablist" aria-label="Brand chapters">
            <button
              v-for="(s, i) in STEPS"
              :key="s.key"
              type="button"
              role="tab"
              :aria-selected="step === i && !finished"
              class="brand-book-tab w-auto"
              :class="{ active: step === i && !finished }"
              :title="`${i + 1}. ${t(s.title)}`"
              @click="step = i; finished = false"
            >
              <span class="tab-label">{{ t(s.title) }}</span>
            </button>
          </nav>

          <!-- Tools (Background + PDF) placed on the right -->
          <div class="brand-dock-tools">
            <div ref="bgToolRef" class="brand-bg-tool">
              <button
                type="button"
                class="btn-tool w-auto"
                :class="{ active: bgMenuOpen || hasCustomBg }"
                title="Edit background image for Desktop & Mobile"
                aria-label="Edit background image"
                @click.stop="bgMenuOpen = !bgMenuOpen"
              >
                <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
                  <rect width="18" height="18" x="3" y="3" rx="2" ry="2" />
                  <circle cx="9" cy="9" r="2" />
                  <path d="m21 15-3.086-3.086a2 2 0 0 0-2.828 0L6 21" />
                </svg>
                <span class="tool-label">Background</span>
                <span v-if="hasCustomBg" class="custom-badge" title="Custom background active">●</span>
                <svg class="chevron-arrow" width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
                  <path d="m6 9 6 6 6-6" />
                </svg>
              </button>

              <BrandBgPopover
                :is-open="bgMenuOpen"
                :bg-target-device="bgTargetDevice"
                :custom-bg-desktop="customBgDesktop"
                :custom-bg-mobile="customBgMobile"
                :current-active-bg="currentActiveBg"
                @close="bgMenuOpen = false"
                @set-target-device="(d) => { bgTargetDevice = d }"
                @pick-bg-file="onPickBgFile"
                @clear-bg="clearBg"
                @apply-bg-url="applyBgUrl"
              />
            </div>

            <!-- Export to PDF button -->
            <button
              type="button"
              class="btn-tool btn-pdf w-auto"
              title="Export Brand Guideline Book to PDF"
              aria-label="Export Brand Guideline Book to PDF"
              @click="exportPdf"
            >
              <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
                <path d="M15 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V7Z" />
                <path d="M14 2v4a2 2 0 0 0 2 2h4" />
                <path d="M10 9H8" />
                <path d="M16 13H8" />
                <path d="M16 17H8" />
              </svg>
              <span class="tool-label">Export PDF</span>
            </button>
          </div>
        </div>
      </DockBar>
    </template>
  </div>
</template>
