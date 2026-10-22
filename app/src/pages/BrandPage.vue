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
  base64ToBytes, boundsHint, checkImageBounds, fitWithin, imageDimensions,
  resizeToMaxSide, resizeUrlToMaxSide, IMAGE_MAX_SIDE,
} from '@/core/images'
import type { BrandImageKind } from '@/core/images'

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
const styleDraft = ref({ radius: 12, fillOpacity: 100, strokeWidth: 1, shadow: 'soft' })
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

function capitalize(text: string): string {
  return text.charAt(0).toUpperCase() + text.slice(1)
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

function applyBgUrl(): void {
  const url = bgUrlDraft.value.trim()
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
import api from '@/api/index'
const LOGO_LABELS = ['brand.logoMain', 'brand.logoSecondary', 'brand.logoSocial']
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

async function saveMoodboard(moodboard: string[]): Promise<void> {
  try {
    await saveBrand.mutateAsync({ moodboard })
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

async function addDo(): Promise<void> {
  const value = doDraft.value.trim()
  if (!canWrite.value || !brand.value || !value || busy.value) return
  try {
    await saveBrand.mutateAsync({ dos: [...brand.value.dos, value] })
    doDraft.value = ''
  } catch {
    // keep the draft so nothing typed is lost
  }
}

async function addDont(): Promise<void> {
  const value = dontDraft.value.trim()
  if (!canWrite.value || !brand.value || !value || busy.value) return
  try {
    await saveBrand.mutateAsync({ donts: [...brand.value.donts, value] })
    dontDraft.value = ''
  } catch {
    // keep the draft so nothing typed is lost
  }
}
</script>

<template>
  <div class="brandpage" :class="{ 'has-custom-bg': hasCustomBg }" :style="customBgStyle">
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
      <section v-show="step === 0 && !finished" class="brand-body brand-hero">
        <p class="brand-prompt">{{ t('brand.stepIdentityHint') }}</p>
        <input id="brand-channel" class="field" :value="brand.channel" :disabled="!canWrite || busy"
          :title="canWrite ? '' : t('auth.noPerm')" :aria-label="t('brand.stepIdentityHint')" @change="save('channel', $event)" />
      </section>

      <!-- 1 · voice -->
      <section v-show="step === 1 && !finished" class="brand-body brand-voice-step">
        <div class="brand-voice-layout">
          <div class="brand-voice-fields">
            <div class="brand-field-group">
              <label class="lbl" for="brand-positioning">Positioning</label>
              <input id="brand-positioning" class="field" :value="brand.positioning" :disabled="!canWrite || busy"
                :title="canWrite ? '' : t('auth.noPerm')" @change="save('positioning', $event)" />
            </div>
            <div class="brand-field-group">
              <label class="lbl" for="brand-slogan">Slogan (bio)</label>
              <input id="brand-slogan" class="field" :value="brand.slogan" :disabled="!canWrite || busy"
                :title="canWrite ? '' : t('auth.noPerm')" @change="save('slogan', $event)" />
            </div>
            <div class="brand-field-group">
              <label class="lbl" for="brand-audience">Audience</label>
              <input id="brand-audience" class="field" :value="brand.audience" :disabled="!canWrite || busy"
                :title="canWrite ? '' : t('auth.noPerm')" @change="save('audience', $event)" />
            </div>
            <div class="brand-field-group">
              <label class="lbl" for="brand-voice">Voice + tone</label>
              <input id="brand-voice" class="field" :value="brand.voice" :disabled="!canWrite || busy"
                :title="canWrite ? '' : t('auth.noPerm')" @change="save('voice', $event)" />
            </div>
          </div>
          <div class="brand-voice-panels">
            <div class="panel">
              <strong>Do</strong>
              <ul style="padding-left: 18px; margin: 6px 0;"><li v-for="d in brand.dos" :key="d">{{ d }}</li></ul>
              <div class="row"><input class="field" style="flex: 1;" v-model="doDraft" placeholder="add rule" :disabled="!canWrite || busy"
                :title="canWrite ? '' : t('auth.noPerm')" @keyup.enter="addDo" /><button class="btn" :disabled="!canWrite || busy || !doDraft.trim()"
                :title="canWrite ? '' : t('auth.noPerm')" @click="addDo">Add</button></div>
            </div>
            <div class="panel">
              <strong>Don't</strong>
              <ul style="padding-left: 18px; margin: 6px 0;"><li v-for="d in brand.donts" :key="d">{{ d }}</li></ul>
              <div class="row"><input class="field" style="flex: 1;" v-model="dontDraft" placeholder="add rule" :disabled="!canWrite || busy"
                :title="canWrite ? '' : t('auth.noPerm')" @keyup.enter="addDont" /><button class="btn" :disabled="!canWrite || busy || !dontDraft.trim()"
                :title="canWrite ? '' : t('auth.noPerm')" @click="addDont">Add</button></div>
            </div>
          </div>
        </div>
      </section>

      <!-- 1 · logos -->
      <section v-show="step === 2 && !finished" class="brand-body">
        <div class="grid3">
          <div v-for="slot in logoSlots" :key="slot.index" class="logo-slot">
            <div class="logo-preview">
              <img v-if="slot.url" :src="slot.url" :alt="t(LOGO_LABELS[slot.index])"
                @load="noteRenderedSize(slot.url, $event)" />
              <span v-else class="muted">{{ t(LOGO_LABELS[slot.index]) }}</span>
              <button v-if="slot.url" type="button" class="logo-remove" :disabled="!canWrite"
                :title="t('brand.imageRemove')" :aria-label="t('brand.imageRemove')" @click="clearLogo(slot.index)">
                ×
              </button>
            </div>
            <div class="row">
              <label class="btn" :class="{ disabled: !canWrite || uploadingSlot === `logo-${slot.index}` }">
                {{ uploadingSlot === `logo-${slot.index}` ? t('brand.imageUploading') : t('brand.imageUpload') }}
                <input type="file" accept=".png,.jpg,.jpeg,.webp,.gif" hidden
                  :disabled="!canWrite || uploadingSlot === `logo-${slot.index}`"
                  @change="pickLogo(slot.index, $event)" />
              </label>
              <input class="field" style="flex: 1;" :value="slot.url" :disabled="!canWrite || busy"
                :placeholder="t('brand.imageUrl')" @change="setLogoUrl(slot.index, $event)" />
            </div>
            <span class="muted img-bounds">{{ boundsHint('logo') }}</span>
            <button v-if="oversized[slot.url]" type="button" class="btn oversize-fix"
              :disabled="!canWrite || uploadingSlot === `fix-${slot.url}`" @click="fixStoredImage('logo', slot.url)">
              {{ uploadingSlot === `fix-${slot.url}` ? t('brand.imageFixing') : t('brand.imageFix') }}
            </button>
          </div>
        </div>
      </section>

      <!-- 2 · palette -->
      <section v-show="step === 3 && !finished" class="brand-body">
        <div class="panel">
          <div class="palette-strip">
            <span class="strip-label muted">{{ t('brand.palettePreview') }}</span>
            <span class="strip-btn" :style="{ background: paletteDraft[0] || 'var(--accent)' }">
              {{ t('brand.paletteSample') }}
            </span>
            <span class="strip-link" :style="{ color: paletteDraft[0] || 'var(--accent)' }">
              {{ t('brand.paletteLink') }}
            </span>
            <span class="strip-bars" :aria-label="t('brand.paletteBars')">
              <span v-for="i in 4" :key="i" class="strip-bar"
                :style="{ background: paletteDraft[i - 1] || 'var(--wash)', height: `${10 + i * 3}px` }" />
            </span>
          </div>
          <div class="palette-grid">
            <div v-for="slot in PALETTE_SLOTS" :key="slot" class="palette-card"
              :class="{ 'is-primary': slot === '1' }">
              <label class="palette-tile-label" :title="t('brand.palettePick')">
                <input type="color" :value="paletteDraft[Number(slot) - 1] || '#0a0a0a'"
                  :disabled="!canWrite || busy" :aria-label="`Color ${slot}`"
                  @input="paletteDraft[Number(slot) - 1] = ($event.target as HTMLInputElement).value"
                  @change="savePalette" />
                <span class="palette-tile"
                  :style="{ background: paletteDraft[Number(slot) - 1] || 'var(--wash)' }">
                  <span class="palette-tile-role">{{ t(PALETTE_ROLES[slot]) }}</span>
                  <span v-if="slot === '1'" class="palette-tile-star" aria-hidden="true">★</span>
                  <span class="palette-tile-sample"
                    :style="{ color: readableOn(paletteDraft[Number(slot) - 1] || '#0a0a0a') }">Aa</span>
                </span>
              </label>
              <div class="palette-meta">
                <p class="palette-caption">{{ t(PALETTE_USAGE[slot]) }}</p>
                <div class="palette-stats">
                  <label class="palette-field">
                    <input class="field palette-hex" v-model="paletteDraft[Number(slot) - 1]"
                      :disabled="!canWrite || busy" placeholder="#0a0a0a" :aria-label="`Color hex ${slot}`"
                      @change="savePalette" />
                  </label>
                  <button type="button" class="palette-copy" :disabled="!paletteDraft[Number(slot) - 1]"
                    @click="copyHex(slot)">
                    {{ copiedSlot === slot ? t('brand.paletteCopied') : t('brand.paletteCopy') }}
                  </button>
                </div>
              </div>
            </div>
          </div>
        </div>
      </section>

      <!-- 3 · typography -->
      <section v-show="step === 4 && !finished" class="brand-body">
        <div class="panel">
          <div class="font-preview" :style="{ fontFamily: `'${brand.fonts[0] || 'Inter'}', var(--font)` }">
            <span class="font-preview-sample">Aa Bb 123 · สวัสดี</span>
            <span class="muted">{{ t('brand.fontPreview') }}</span>
          </div>
          <div class="font-grid">
            <div v-for="f in importedFonts" :key="f.name" class="font-choice"
              :class="{ active: brand.fonts[0] === f.family }">
              <button type="button" class="font-pick" :disabled="!canWrite" @click="selectFont(f.family)">
                <span class="font-sample" :style="{ fontFamily: `'${f.family}', var(--font)` }">Aa Bb 123</span>
                <span class="font-name">{{ f.family }}</span>
                <span v-if="brand.fonts[0] === f.family" class="font-check" aria-hidden="true">✓</span>
              </button>
              <button type="button" class="font-remove" :disabled="!canWrite" :title="t('brand.fontDelete')"
                :aria-label="`${t('brand.fontDelete')} ${f.family}`" @click="removeFont(f)">×</button>
            </div>
            <button v-for="s in suggestedFonts" :key="s" type="button" class="font-choice font-pick"
              :class="{ active: brand.fonts[0] === s }" :style="{ fontFamily: `'${s}', var(--font)` }"
              :disabled="!canWrite" @click="selectFont(s)">
              <span class="font-sample">Aa Bb 123</span>
              <span class="font-name">{{ s }}</span>
              <span v-if="brand.fonts[0] === s" class="font-check" aria-hidden="true">✓</span>
            </button>
            <label class="font-choice font-import" :class="{ disabled: !canWrite || uploading }"
              :title="t('brand.fontHint')">
              <span class="font-import-plus" aria-hidden="true">+</span>
              <span class="font-name">{{ uploading ? t('brand.uploading') : t('brand.fontImport') }}</span>
              <input type="file" accept=".woff2,.woff,.ttf,.otf" multiple hidden
                :disabled="!canWrite || uploading" @change="importFonts" />
            </label>
          </div>
          <p v-if="fontError" class="autherr" role="alert">{{ fontError }}</p>
          <p class="muted panel-note">{{ t('brand.applied') }}</p>
        </div>
      </section>

      <!-- 4 · style -->
      <section v-show="step === 5 && !finished" class="brand-body">
        <div class="panel style-panel">
          <div class="style-row">
            <span class="style-label">{{ t('brand.styleRadius') }}</span>
            <input class="field style-range" type="range" min="0" max="24" step="1" v-model.number="styleDraft.radius"
              :disabled="!canWrite || busy" :aria-label="t('brand.styleRadius')" @change="saveStyle" />
            <span class="style-value">
              <input class="field style-num" type="number" min="0" max="24" step="1" v-model.number="styleDraft.radius"
                :disabled="!canWrite || busy" :aria-label="t('brand.styleRadius')" @change="saveStyle" />
              <span class="muted">px</span>
            </span>
          </div>
          <div class="style-row">
            <span class="style-label">{{ t('brand.styleFill') }}</span>
            <input class="field style-range" type="range" min="5" max="100" step="1" v-model.number="styleDraft.fillOpacity"
              :disabled="!canWrite || busy" :aria-label="t('brand.styleFill')" @change="saveStyle" />
            <span class="style-value">
              <input class="field style-num" type="number" min="5" max="100" step="1" v-model.number="styleDraft.fillOpacity"
                :disabled="!canWrite || busy" :aria-label="t('brand.styleFill')" @change="saveStyle" />
              <span class="muted">%</span>
            </span>
          </div>
          <div class="style-row">
            <span class="style-label">{{ t('brand.styleStroke') }}</span>
            <input class="field style-range" type="range" min="0" max="3" step="1" v-model.number="styleDraft.strokeWidth"
              :disabled="!canWrite || busy" :aria-label="t('brand.styleStroke')" @change="saveStyle" />
            <span class="style-value">
              <input class="field style-num" type="number" min="0" max="3" step="1" v-model.number="styleDraft.strokeWidth"
                :disabled="!canWrite || busy" :aria-label="t('brand.styleStroke')" @change="saveStyle" />
              <span class="muted">px</span>
            </span>
          </div>
          <div class="style-row">
            <span class="style-label">{{ t('brand.styleShadow') }}</span>
            <div class="style-segmented" role="group" :aria-label="t('brand.styleShadow')">
              <button v-for="option in ['none', 'soft', 'strong']" :key="option" type="button"
                :class="{ on: styleDraft.shadow === option }" :disabled="!canWrite || busy"
                @click="styleDraft.shadow = option; saveStyle()">{{ t('brand.styleShadow' + capitalize(option)) }}</button>
            </div>
          </div>
          <div class="style-sample">
            <span class="btn btn-primary" :class="{ ghost: true }">{{ t('brand.paletteSample') }}</span>
            <span class="pv-link" :style="{ color: 'var(--accent)' }">{{ t('brand.paletteLink') }}</span>
            <span class="strip-bars">
              <span v-for="i in 4" :key="i" class="strip-bar" :style="{ height: `${8 + i * 3}px` }" />
            </span>
          </div>
          <div class="style-row style-export">
            <span class="style-label">{{ t('brand.styleExport') }}</span>
            <div class="row" style="gap: 8px;">
              <button class="btn" :disabled="styleBusy" @click="copyStyleCss">{{ t('brand.styleCopyCss') }}</button>
              <button class="btn" :disabled="styleBusy" @click="downloadBrandJson">{{ t('brand.styleDownloadJson') }}</button>
              <label class="btn" :class="{ disabled: !canWrite || styleBusy }"
                :title="canWrite ? '' : t('auth.noPerm')">
                {{ t('brand.styleUploadJson') }}
                <input type="file" accept=".json,application/json" hidden
                  :disabled="!canWrite || styleBusy" @change="importBrandJson" />
              </label>
              <span v-if="styleNotice" class="oknote" role="status">{{ styleNotice }}</span>
              <span v-if="styleErr" class="autherr" role="alert">{{ styleErr }}</span>
            </div>
          </div>
        </div>
      </section>

      <!-- 5 · moodboard -->
      <section v-show="step === 6 && !finished" class="brand-body">
        <div class="moodboard">
          <div v-for="(url, i) in moodboard" :key="`${url}-${i}`" class="mood-item">
            <img :src="url" :alt="`${t('brand.moodboard')} ${i + 1}`" @load="noteRenderedSize(url, $event)" />
            <button v-if="oversized[url]" type="button" class="btn oversize-fix mood-fix"
              :disabled="!canWrite || uploadingSlot === `fix-${url}`" @click="fixStoredImage('moodboard', url)">
              {{ uploadingSlot === `fix-${url}` ? t('brand.imageFixing') : t('brand.imageFix') }}
            </button>
            <button type="button" class="logo-remove" :disabled="!canWrite" :title="t('brand.imageRemove')"
              :aria-label="t('brand.imageRemove')" @click="removeMoodboard(i)">×</button>
          </div>
          <label v-if="moodboard.length < 12" class="mood-add" :class="{ disabled: !canWrite }">
            <span>{{ uploadingSlot === 'moodboard' ? t('brand.imageUploading') : t('brand.moodboardAdd') }}</span>
            <input type="file" accept=".png,.jpg,.jpeg,.webp,.gif" multiple hidden
              :disabled="!canWrite || uploadingSlot === 'moodboard'" @change="addMoodboard" />
          </label>
        </div>
        <div class="row mt">
          <input class="field mood-url" style="flex: 1;" v-model="moodboardDraft" :disabled="!canWrite"
            :placeholder="t('brand.imageUrl')" @keyup.enter="addMoodboardUrl" />
          <button class="btn mood-add-btn" :disabled="!canWrite || !moodboardDraft.trim()" @click="addMoodboardUrl">
            {{ t('brand.imageAdd') }}
          </button>
        </div>
        <p class="muted img-bounds">{{ boundsHint('moodboard') }}</p>
        <p v-if="imageNotice" class="oknote" role="status">{{ imageNotice }}</p>
        <p v-if="imageError" class="autherr" role="alert">{{ imageError }}</p>
      </section>

      <!-- Finished State -->
      <section v-show="finished" class="brand-body brand-finished-hero">
        <div class="brand-finished-card">
          <span class="brand-finished-badge">Brand Guidelines Complete</span>
          <div class="brand-finished-name">{{ brand.channel || 'Studio Channel' }}</div>
          <p class="brand-finished-desc">All 7 chapters configured and ready for presentation.</p>
          <div class="brand-finished-actions">
            <button type="button" class="btn btn-primary btn-finished-export" @click="exportPdf">
              Export PDF
            </button>
            <button type="button" class="btn btn-finished-review" @click="restartStep">
              Review Chapters
            </button>
          </div>
        </div>
      </section>

      <!-- PRINTABLE BRAND GUIDELINE BOOK: 7 Chapters of the 16:9 Brand Card -->
      <div id="brand-guideline-pdf" class="brand-pdf-deck" :style="customBgStyle" aria-hidden="true">
        <!-- PAGE 1: 1. BRAND IDENTITY -->
        <section class="pdf-slide pdf-slide-hero" :style="{ backgroundImage: `url('${activeBrandBg}')` }">
          <div class="pdf-cover-shade"></div>
          <div class="pdf-cover-container">
            <header class="pdf-slide-header">
              <span class="pdf-slide-title">1. Brand Identity</span>
              <span class="pdf-header-channel">{{ brand.channel }}</span>
            </header>

            <div class="pdf-hero-content">
              <div class="pdf-hero-title">{{ brand.channel || 'Studio Channel' }}</div>
              <p v-if="brand.positioning" class="pdf-hero-pos">{{ brand.positioning }}</p>
            </div>

            <footer class="pdf-slide-footer">
              <span>{{ brand.channel }}</span>
              <span>1 / 7</span>
            </footer>
          </div>
        </section>

        <!-- PAGE 2: 2. BRAND VOICE -->
        <section class="pdf-slide pdf-slide-dark">
          <header class="pdf-slide-header">
            <span class="pdf-slide-title">2. Brand Voice</span>
            <span class="pdf-header-channel">{{ brand.channel }}</span>
          </header>

          <div class="pdf-slide-body">
            <div class="pdf-voice-meta">
              <div class="pdf-voice-field" v-if="brand.positioning">
                <span class="pdf-field-lbl">Positioning</span>
                <p class="pdf-field-val">{{ brand.positioning }}</p>
              </div>
              <div class="pdf-voice-field" v-if="brand.audience">
                <span class="pdf-field-lbl">Audience</span>
                <p class="pdf-field-val">{{ brand.audience }}</p>
              </div>
              <div class="pdf-voice-field" v-if="brand.voice">
                <span class="pdf-field-lbl">Voice &amp; Tone</span>
                <p class="pdf-field-val">{{ brand.voice }}</p>
              </div>
            </div>

            <div class="pdf-voice-grid">
              <div class="pdf-voice-card pdf-card-do">
                <div class="pdf-card-title">Do</div>
                <ul class="pdf-clean-list">
                  <li v-for="d in brand.dos" :key="d">{{ d }}</li>
                  <li v-if="!brand.dos.length" class="muted">Show the actual craft and process</li>
                </ul>
              </div>
              <div class="pdf-voice-card pdf-card-dont">
                <div class="pdf-card-title">Don't</div>
                <ul class="pdf-clean-list pdf-list-dont">
                  <li v-for="d in brand.donts" :key="d">{{ d }}</li>
                  <li v-if="!brand.donts.length" class="muted">Clickbait and artificial urgency</li>
                </ul>
              </div>
            </div>
          </div>

          <footer class="pdf-slide-footer">
            <span>{{ brand.channel }}</span>
            <span>2 / 7</span>
          </footer>
        </section>

        <!-- PAGE 3: 3. LOGOS -->
        <section class="pdf-slide pdf-slide-dark">
          <header class="pdf-slide-header">
            <span class="pdf-slide-title">3. Logos</span>
            <span class="pdf-header-channel">{{ brand.channel }}</span>
          </header>

          <div class="pdf-slide-body">
            <div class="pdf-logos-grid">
              <div v-for="slot in logoSlots" :key="slot.index" class="pdf-logo-card">
                <div class="pdf-logo-display" :class="{ 'on-photo': slot.index === 2 }" :style="slot.index === 2 ? { backgroundImage: `url('${activeBrandBg}')` } : {}">
                  <img v-if="slot.url" :src="slot.url" :alt="t(LOGO_LABELS[slot.index])" class="pdf-logo-img" />
                  <div v-else class="pdf-logo-placeholder">
                    <span class="pdf-logo-fallback">{{ brandInitials(brand.channel) }}</span>
                  </div>
                </div>
                <div class="pdf-logo-info">
                  <strong>{{ t(LOGO_LABELS[slot.index]) }}</strong>
                  <span class="muted">{{ slot.index === 0 ? 'Primary logo' : slot.index === 1 ? 'Secondary lockup' : 'Social & icon stamp' }}</span>
                </div>
              </div>
            </div>
          </div>

          <footer class="pdf-slide-footer">
            <span>{{ brand.channel }}</span>
            <span>3 / 7</span>
          </footer>
        </section>

        <!-- PAGE 4: 4. PALETTE -->
        <section class="pdf-slide pdf-slide-dark">
          <header class="pdf-slide-header">
            <span class="pdf-slide-title">4. Palette</span>
            <span class="pdf-header-channel">{{ brand.channel }}</span>
          </header>

          <div class="pdf-slide-body">
            <div class="pdf-palette-strip">
              <div v-for="slot in PALETTE_SLOTS" :key="`strip-${slot}`" class="pdf-palette-strip-seg"
                :style="{ background: paletteDraft[Number(slot) - 1] || '#111111', flex: slot === '1' ? '5' : slot === '2' ? '3' : slot === '3' ? '1.5' : '1' }">
                <span :style="{ color: readableOn(paletteDraft[Number(slot) - 1] || '#111111') }">
                  {{ t(PALETTE_ROLES[slot]) }}
                </span>
              </div>
            </div>

            <div class="pdf-palette-grid">
              <div v-for="slot in PALETTE_SLOTS" :key="slot" class="pdf-palette-card">
                <div class="pdf-palette-swatch" :style="{ background: paletteDraft[Number(slot) - 1] || '#111111' }">
                  <span class="pdf-palette-sample" :style="{ color: readableOn(paletteDraft[Number(slot) - 1] || '#111111') }">Aa</span>
                </div>
                <div class="pdf-palette-details">
                  <strong>{{ t(PALETTE_ROLES[slot]) }}</strong>
                  <span class="pdf-hex-code">{{ paletteDraft[Number(slot) - 1] || '#111111' }}</span>
                  <span class="pdf-rgb-code">{{ hexToRgb(paletteDraft[Number(slot) - 1] || '#111111') }}</span>
                  <p class="muted">{{ t(PALETTE_USAGE[slot]) }}</p>
                </div>
              </div>
            </div>
          </div>

          <footer class="pdf-slide-footer">
            <span>{{ brand.channel }}</span>
            <span>4 / 7</span>
          </footer>
        </section>

        <!-- PAGE 5: 5. TYPOGRAPHY -->
        <section class="pdf-slide pdf-slide-dark">
          <header class="pdf-slide-header">
            <span class="pdf-slide-title">5. Typography</span>
            <span class="pdf-header-channel">{{ brand.channel }}</span>
          </header>

          <div class="pdf-slide-body" :style="{ fontFamily: `'${brand.fonts[0] || 'Noto Sans Thai'}', sans-serif` }">
            <div class="pdf-type-lead">
              <div class="pdf-font-name">{{ brand.fonts[0] || 'Noto Sans Thai' }}</div>
              <span class="muted">Primary Typeface</span>
            </div>

            <div class="pdf-type-specimen-box">
              <div class="pdf-type-big-aa">Aa</div>
              <div class="pdf-type-charset">
                <div>ABCDEFGHIJKLMNOPQRSTUVWXYZ · abcdefghijklmnopqrstuvwxyz</div>
                <div>กขคฆงจฉชซฌญฎฏฐฑฒณดตถทธนบปผฝพฟภมยรลวศษสหฬอฮ 0123456789</div>
              </div>
            </div>

            <div class="pdf-type-preview-row">
              <div class="pdf-type-scale-item">
                <span class="pdf-scale-label">Display</span>
                <div class="pdf-scale-display">{{ brand.channel || 'Studio Channel' }}</div>
              </div>
              <div class="pdf-type-scale-item">
                <span class="pdf-scale-label">Body Text</span>
                <p class="pdf-scale-body">เนื้อหาที่จริงใจ ไม่เร่งรีบ ให้ความสำคัญกับกระบวนการและคุณค่าที่แท้จริง</p>
              </div>
            </div>
          </div>

          <footer class="pdf-slide-footer">
            <span>{{ brand.channel }}</span>
            <span>5 / 7</span>
          </footer>
        </section>

        <!-- PAGE 6: 6. STYLE -->
        <section class="pdf-slide pdf-slide-dark">
          <header class="pdf-slide-header">
            <span class="pdf-slide-title">6. Style</span>
            <span class="pdf-header-channel">{{ brand.channel }}</span>
          </header>

          <div class="pdf-slide-body">
            <div class="pdf-style-tokens-grid">
              <div class="pdf-style-token-card">
                <div class="pdf-token-text">
                  <span class="pdf-token-lbl">Corner Radius</span>
                  <strong>{{ styleDraft.radius }}px</strong>
                  <p class="muted">Softened corners for cards, buttons, and media.</p>
                </div>
                <div class="pdf-token-preview">
                  <div class="pdf-preview-box" :style="{ borderRadius: `${styleDraft.radius}px` }"></div>
                </div>
              </div>

              <div class="pdf-style-token-card">
                <div class="pdf-token-text">
                  <span class="pdf-token-lbl">Surface Shadow</span>
                  <strong>{{ capitalize(styleDraft.shadow) }}</strong>
                  <p class="muted">Ambient elevation for overlays and dropdowns.</p>
                </div>
                <div class="pdf-token-preview">
                  <div class="pdf-preview-shadow" :class="`shadow-${styleDraft.shadow}`"></div>
                </div>
              </div>

              <div class="pdf-style-token-card">
                <div class="pdf-token-text">
                  <span class="pdf-token-lbl">Fill Opacity</span>
                  <strong>{{ styleDraft.fillOpacity }}%</strong>
                  <p class="muted">Surface density for solid layers.</p>
                </div>
                <div class="pdf-token-preview">
                  <div class="pdf-preview-fill" :style="{ opacity: styleDraft.fillOpacity / 100 }"></div>
                </div>
              </div>
            </div>
          </div>

          <footer class="pdf-slide-footer">
            <span>{{ brand.channel }}</span>
            <span>6 / 7</span>
          </footer>
        </section>

        <!-- PAGE 7: 7. MOODBOARD -->
        <section class="pdf-slide pdf-slide-dark">
          <header class="pdf-slide-header">
            <span class="pdf-slide-title">7. Moodboard</span>
            <span class="pdf-header-channel">{{ brand.channel }}</span>
          </header>

          <div class="pdf-slide-body">
            <!-- If moodboard has items, show them -->
            <div v-if="moodboard.length > 0" class="pdf-mood-grid">
              <div v-for="(url, i) in moodboard" :key="`${url}-${i}`" class="pdf-mood-item" :style="{ backgroundImage: `url('${url}')` }"></div>
            </div>
            <!-- If no extra moodboard items, show the brand photo hero cleanly -->
            <div v-else class="pdf-mood-hero-single" :style="{ backgroundImage: `url('${activeBrandBg}')` }">
              <div class="pdf-mood-single-shade"></div>
              <div class="pdf-mood-single-caption">
                <strong>Visual References</strong>
                <span class="muted">Brand photography and atmosphere anchor</span>
              </div>
            </div>
          </div>

          <footer class="pdf-slide-footer">
            <span>{{ brand.channel }}</span>
            <span>7 / 7</span>
          </footer>
        </section>
      </div>

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
              <span class="tab-num">{{ i + 1 }}</span>
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
                <svg class="caret" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
                  <path d="m6 9 6 6 6-6" />
                </svg>
              </button>

              <!-- Edit image modal / popover (matching uploaded screenshot) -->
              <div v-if="bgMenuOpen" class="brand-bg-popover edit-image-modal" @click.stop>
                <div class="bg-popover-head">
                  <strong class="edit-image-title">Edit image</strong>
                  <button type="button" class="btn-close w-auto" aria-label="Close" @click="bgMenuOpen = false">
                    <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
                      <path d="M18 6 6 18M6 6l12 12" />
                    </svg>
                  </button>
                </div>

                <!-- Device Target Toggle (Desktop vs Mobile) -->
                <div class="bg-device-toggle">
                  <button
                    type="button"
                    class="bg-device-btn"
                    :class="{ active: bgTargetDevice === 'desktop' }"
                    @click="bgTargetDevice = 'desktop'"
                  >
                    <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                      <rect width="20" height="14" x="2" y="3" rx="2" />
                      <line x1="8" x2="16" y1="21" y2="21" />
                      <line x1="12" x2="12" y1="17" y2="21" />
                    </svg>
                    <span>Desktop (16:9)</span>
                    <span v-if="customBgDesktop" class="device-dot">●</span>
                  </button>
                  <button
                    type="button"
                    class="bg-device-btn"
                    :class="{ active: bgTargetDevice === 'mobile' }"
                    @click="bgTargetDevice = 'mobile'"
                  >
                    <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                      <rect width="14" height="20" x="5" y="2" rx="2" ry="2" />
                      <line x1="12" x2="12.01" y1="18" y2="18" />
                    </svg>
                    <span>Mobile (9:16)</span>
                    <span v-if="customBgMobile" class="device-dot">●</span>
                  </button>
                </div>

                <!-- Preview Area styled like uploaded image -->
                <div class="edit-image-preview-wrap">
                  <!-- Desktop preview -->
                  <div
                    v-if="bgTargetDevice === 'desktop'"
                    class="edit-preview-desktop"
                    :style="customBgDesktop ? { backgroundImage: `url(${customBgDesktop})` } : {}"
                  >
                    <div class="edit-preview-badge">16 : 9</div>
                    <div v-if="!customBgDesktop" class="edit-preview-empty">Default background active</div>
                  </div>
                  <!-- Mobile preview -->
                  <div
                    v-else
                    class="edit-preview-mobile"
                    :style="customBgMobile ? { backgroundImage: `url(${customBgMobile})` } : {}"
                  >
                    <div class="edit-preview-badge">9 : 16</div>
                    <div v-if="!customBgMobile" class="edit-preview-empty">Default mobile background</div>
                  </div>
                </div>

                <div class="bg-popover-body">
                  <div class="row" style="gap: 8px;">
                    <label class="btn btn-sm btn-upload w-auto">
                      <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
                        <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4" />
                        <polyline points="17 8 12 3 7 8" />
                        <line x1="12" x2="12" y1="3" y2="15" />
                      </svg>
                      <span>Upload {{ bgTargetDevice === 'desktop' ? 'Desktop' : 'Mobile' }} Image</span>
                      <input type="file" accept="image/*" hidden @change="onPickBgFile" />
                    </label>
                    <button
                      v-if="currentActiveBg"
                      type="button"
                      class="btn btn-sm btn-remove-bg w-auto"
                      @click="clearBg"
                    >
                      Remove
                    </button>
                  </div>
                  <div class="bg-url-row">
                    <input
                      class="field field-sm"
                      v-model="bgUrlDraft"
                      :placeholder="`or paste ${bgTargetDevice} image URL`"
                      @keyup.enter="applyBgUrl"
                    />
                    <button type="button" class="btn btn-sm w-auto" :disabled="!bgUrlDraft.trim()" @click="applyBgUrl">Apply</button>
                  </div>
                </div>
              </div>
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
