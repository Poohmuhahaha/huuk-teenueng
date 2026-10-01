// Brand image rules shared by the Brand page and the mock API: which slots
// exist, the accepted formats, and the pixel bounds the server enforces.
export type BrandImageKind = 'logo' | 'moodboard'

export interface ImageBounds {
  minWidth: number
  minHeight: number
  maxWidth: number
  maxHeight: number
}

export const IMAGE_BOUNDS: Record<BrandImageKind, ImageBounds> = {
  // Every brand image is capped at 500x500 px so the kit stays light.
  logo: { minWidth: 32, minHeight: 32, maxWidth: 500, maxHeight: 500 },
  moodboard: { minWidth: 32, minHeight: 32, maxWidth: 500, maxHeight: 500 },
}

/** Accepted raster formats (the server sniffs the same list). */
export const IMAGE_EXTENSIONS = ['png', 'jpg', 'jpeg', 'webp', 'gif']

export function imageMime(name: string): string {
  const ext = name.split('.').pop()?.toLowerCase() ?? ''
  switch (ext) {
    case 'png': return 'image/png'
    case 'webp': return 'image/webp'
    case 'gif': return 'image/gif'
    case 'jpg':
    case 'jpeg': return 'image/jpeg'
    default: return ''
  }
}

/**
 * Pixel dimensions from an image's header (PNG, JPEG, GIF, WebP) — enough to
 * validate a file before uploading it, without loading the whole image.
 * Returns null when the header is not recognized.
 */
export function imageDimensions(bytes: Uint8Array): { width: number; height: number } | null {
  const u16be = (i: number): number => (bytes[i] << 8) | bytes[i + 1]
  const u16le = (i: number): number => bytes[i] | (bytes[i + 1] << 8)
  const u32be = (i: number): number =>
    ((bytes[i] << 24) | (bytes[i + 1] << 16) | (bytes[i + 2] << 8) | bytes[i + 3]) >>> 0

  const png = bytes.length >= 24 && bytes[0] === 0x89 && bytes[1] === 0x50 && bytes[2] === 0x4e && bytes[3] === 0x47
  if (png) return { width: u32be(16), height: u32be(20) }

  const gif = bytes.length >= 10 && bytes[0] === 0x47 && bytes[1] === 0x49 && bytes[2] === 0x46 && bytes[3] === 0x38
  if (gif) return { width: u16le(6), height: u16le(8) }

  const jpeg = bytes.length >= 4 && bytes[0] === 0xff && bytes[1] === 0xd8
  if (jpeg) {
    let i = 2
    while (i + 9 < bytes.length) {
      if (bytes[i] !== 0xff) {
        i += 1
        continue
      }
      const marker = bytes[i + 1]
      const isFrame = marker >= 0xc0 && marker <= 0xcf && marker !== 0xc4 && marker !== 0xc8 && marker !== 0xcc
      if (isFrame) return { width: u16be(i + 7), height: u16be(i + 5) }
      if (marker === 0xd8 || (marker >= 0xd0 && marker <= 0xd7)) {
        i += 2
        continue
      }
      const len = u16be(i + 2)
      if (len < 2) return null
      i += 2 + len
    }
    return null
  }

  const webp =
    bytes.length >= 30 &&
    bytes[0] === 0x52 && bytes[1] === 0x49 && bytes[2] === 0x46 && bytes[3] === 0x46 &&
    bytes[8] === 0x57 && bytes[9] === 0x45 && bytes[10] === 0x42 && bytes[11] === 0x50
  if (webp) {
    const chunk = String.fromCharCode(bytes[12], bytes[13], bytes[14], bytes[15])
    if (chunk === 'VP8 ') {
      return { width: u16le(26) & 0x3fff, height: u16le(28) & 0x3fff }
    }
    if (chunk === 'VP8L') {
      const bits = bytes[21] | (bytes[22] << 8) | (bytes[23] << 16) | (bytes[24] << 24)
      return { width: (bits & 0x3fff) + 1, height: ((bits >> 14) & 0x3fff) + 1 }
    }
    if (chunk === 'VP8X') {
      return {
        width: (bytes[24] | (bytes[25] << 8) | (bytes[26] << 16)) + 1,
        height: (bytes[27] | (bytes[28] << 8) | (bytes[29] << 16)) + 1,
      }
    }
    return null
  }

  return null
}

/**
 * Validates an image against its slot's bounds. Returns a user-facing error
 * message, or null when the image is acceptable.
 */
export function checkImageBounds(
  kind: BrandImageKind,
  width: number,
  height: number,
): string | null {
  const b = IMAGE_BOUNDS[kind]
  if (width < b.minWidth || height < b.minHeight) {
    return `${kind} images must be at least ${b.minWidth}×${b.minHeight} px (got ${width}×${height})`
  }
  if (width > b.maxWidth || height > b.maxHeight) {
    return `${kind} images must be at most ${b.maxWidth}×${b.maxHeight} px (got ${width}×${height})`
  }
  return null
}

/** Human hint for a slot, e.g. "PNG/JPG/WebP/GIF · 96×96 – 2048×2048 px". */
export function boundsHint(kind: BrandImageKind): string {
  const b = IMAGE_BOUNDS[kind]
  return `PNG/JPG/WebP/GIF · ${b.minWidth}×${b.minHeight} – ${b.maxWidth}×${b.maxHeight} px`
}

export function base64ToBytes(data: string): Uint8Array {
  const clean = data.replace(/^data:[^,]+,/, '')
  if (typeof atob === 'function') {
    const raw = atob(clean)
    const out = new Uint8Array(raw.length)
    for (let i = 0; i < raw.length; i += 1) out[i] = raw.charCodeAt(i)
    return out
  }
  // Node (tests): Buffer is available even outside the browser.
  return new Uint8Array(Buffer.from(clean, 'base64'))
}

/** Longest side a stored brand image may have. */
export const IMAGE_MAX_SIDE = IMAGE_BOUNDS.logo.maxWidth

/**
 * Target size when fitting `width`x`height` into a `max`-sided square. Pure so
 * the decision is testable; the canvas work lives in `resizeToMaxSide`.
 */
export function fitWithin(
  width: number,
  height: number,
  max: number = IMAGE_MAX_SIDE,
): { width: number; height: number; needed: boolean } {
  const longest = Math.max(width, height)
  if (longest <= max || longest === 0) return { width, height, needed: false }
  const scale = max / longest
  return {
    width: Math.max(1, Math.round(width * scale)),
    height: Math.max(1, Math.round(height * scale)),
    needed: true,
  }
}

/** Whether the browser can re-encode an image (canvas + toBlob). */
export function canResizeImages(): boolean {
  return typeof document !== 'undefined' && typeof createImageBitmap === 'function'
}

async function blobToBase64(blob: Blob): Promise<string> {
  const buffer = new Uint8Array(await blob.arrayBuffer())
  let binary = ''
  for (let i = 0; i < buffer.length; i += 1) binary += String.fromCharCode(buffer[i])
  return typeof btoa === 'function' ? btoa(binary) : Buffer.from(buffer).toString('base64')
}

/**
 * Downscales an image so its longest side fits `max`, returning a base64
 * payload ready for the upload endpoint. Falls back to the original bytes when
 * the browser cannot re-encode (the server still enforces the hard cap).
 */
export async function resizeToMaxSide(
  blob: Blob,
  name: string,
  max: number = IMAGE_MAX_SIDE,
): Promise<{ data: string; name: string; resized: boolean }> {
  const original = await blobToBase64(blob)
  if (!canResizeImages()) return { data: original, name, resized: false }
  try {
    const bitmap = await createImageBitmap(blob)
    const target = fitWithin(bitmap.width, bitmap.height, max)
    if (!target.needed) {
      bitmap.close?.()
      return { data: original, name, resized: false }
    }
    const canvas = document.createElement('canvas')
    canvas.width = target.width
    canvas.height = target.height
    const ctx = canvas.getContext('2d')
    if (!ctx) return { data: original, name, resized: false }
    ctx.drawImage(bitmap, 0, 0, target.width, target.height)
    bitmap.close?.()
    const mime = imageMime(name) === 'image/png' ? 'image/png' : 'image/jpeg'
    const out: Blob | null = await new Promise((resolve) => canvas.toBlob(resolve, mime, 0.9))
    if (!out) return { data: original, name, resized: false }
    const ext = mime === 'image/png' ? 'png' : 'jpg'
    const stem = name.replace(/\.[^.]+$/, '')
    return { data: await blobToBase64(out), name: `${stem}.${ext}`, resized: true }
  } catch {
    return { data: original, name, resized: false }
  }
}

/** Re-encodes an already-stored image URL (same origin) within the cap. */
export async function resizeUrlToMaxSide(
  url: string,
  max: number = IMAGE_MAX_SIDE,
): Promise<{ data: string; name: string; resized: boolean; width: number; height: number }> {
  const res = await fetch(url)
  if (!res.ok) throw new Error(`could not read the stored image (${res.status})`)
  const blob = await res.blob()
  const name = url.split('/').pop()?.split('?')[0] || 'image.png'
  const size = imageDimensions(new Uint8Array(await blob.arrayBuffer()))
  const out = await resizeToMaxSide(blob, name, max)
  return { ...out, width: size?.width ?? 0, height: size?.height ?? 0 }
}
