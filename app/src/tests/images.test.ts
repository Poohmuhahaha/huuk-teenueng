// Image header parsing + slot bounds: the same rules the server enforces.
import { describe, expect, it } from 'vitest'
import {
  boundsHint, checkImageBounds, fitWithin, imageDimensions, imageMime, base64ToBytes,
} from '@/core/images'

function png(width: number, height: number): Uint8Array {
  const bytes = new Uint8Array(24)
  bytes.set([0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a], 0)
  const view = new DataView(bytes.buffer)
  view.setUint32(8, 13)
  bytes.set([0x49, 0x48, 0x44, 0x52], 12)
  view.setUint32(16, width)
  view.setUint32(20, height)
  return bytes
}

function gif(width: number, height: number): Uint8Array {
  const bytes = new Uint8Array(13)
  bytes.set([0x47, 0x49, 0x46, 0x38, 0x39, 0x61], 0)
  bytes[6] = width & 0xff
  bytes[7] = width >> 8
  bytes[8] = height & 0xff
  bytes[9] = height >> 8
  return bytes
}

function jpeg(width: number, height: number): Uint8Array {
  const bytes = new Uint8Array(2 + 18 + 19)
  let i = 0
  bytes.set([0xff, 0xd8], i)
  i += 2
  // APP0 with a 16-byte payload (length 18) to exercise segment skipping.
  bytes.set([0xff, 0xe0, 0x00, 0x12], i)
  i += 4 + 16
  // SOF0: length, precision, height, width, components.
  bytes.set([0xff, 0xc0, 0x00, 0x11, 0x08], i)
  i += 5
  bytes[i] = height >> 8
  bytes[i + 1] = height & 0xff
  bytes[i + 2] = width >> 8
  bytes[i + 3] = width & 0xff
  return bytes
}

function webpVp8x(width: number, height: number): Uint8Array {
  const bytes = new Uint8Array(30)
  bytes.set([0x52, 0x49, 0x46, 0x46], 0)
  bytes.set([0x57, 0x45, 0x42, 0x50], 8)
  bytes.set([0x56, 0x50, 0x38, 0x58], 12)
  const w = width - 1
  const h = height - 1
  bytes.set([w & 0xff, (w >> 8) & 0xff, (w >> 16) & 0xff], 24)
  bytes.set([h & 0xff, (h >> 8) & 0xff, (h >> 16) & 0xff], 27)
  return bytes
}

describe('imageDimensions', () => {
  it('reads every supported format', () => {
    expect(imageDimensions(png(640, 480))).toEqual({ width: 640, height: 480 })
    expect(imageDimensions(gif(320, 200))).toEqual({ width: 320, height: 200 })
    expect(imageDimensions(jpeg(1200, 630))).toEqual({ width: 1200, height: 630 })
    expect(imageDimensions(webpVp8x(800, 600))).toEqual({ width: 800, height: 600 })
  })

  it('returns null for unknown headers', () => {
    expect(imageDimensions(new Uint8Array([1, 2, 3, 4]))).toBeNull()
    expect(imageDimensions(new TextEncoder().encode('not an image at all'))).toBeNull()
  })

  it('decodes base64 payloads', () => {
    const encoded = Buffer.from(png(96, 96)).toString('base64')
    expect(imageDimensions(base64ToBytes(encoded))).toEqual({ width: 96, height: 96 })
    expect(imageDimensions(base64ToBytes(`data:image/png;base64,${encoded}`))).toEqual({
      width: 96,
      height: 96,
    })
  })
})

describe('brand image bounds', () => {
  it('caps every slot at 500x500 with a 32x32 floor', () => {
    for (const kind of ['logo', 'moodboard'] as const) {
      expect(checkImageBounds(kind, 500, 500)).toBeNull()
      expect(checkImageBounds(kind, 32, 32)).toBeNull()
      expect(checkImageBounds(kind, 16, 16)).toMatch(/at least 32×32/)
      expect(checkImageBounds(kind, 501, 501)).toMatch(/at most 500×500/)
      expect(checkImageBounds(kind, 900, 200)).toMatch(/at most 500×500/)
    }
  })

  it('describes the allowed range and formats for the UI', () => {
    expect(boundsHint('logo')).toBe('PNG/JPG/WebP/GIF · 32×32 – 500×500 px')
    expect(boundsHint('moodboard')).toBe('PNG/JPG/WebP/GIF · 32×32 – 500×500 px')
  })

  it('computes the downscale target for oversized images', () => {
    expect(fitWithin(500, 500)).toEqual({ width: 500, height: 500, needed: false })
    expect(fitWithin(300, 200)).toEqual({ width: 300, height: 200, needed: false })
    // Longest side lands exactly on the cap, aspect ratio preserved.
    expect(fitWithin(1000, 500)).toEqual({ width: 500, height: 250, needed: true })
    expect(fitWithin(800, 200)).toEqual({ width: 500, height: 125, needed: true })
    expect(fitWithin(1200, 1200)).toEqual({ width: 500, height: 500, needed: true })
    expect(fitWithin(0, 0).needed).toBe(false)
  })

  it('treats a custom cap as the target', () => {
    expect(fitWithin(1000, 1000, 250)).toEqual({ width: 250, height: 250, needed: true })
  })

  it('maps file extensions to mime types', () => {
    expect(imageMime('a.PNG')).toBe('image/png')
    expect(imageMime('a.jpeg')).toBe('image/jpeg')
    expect(imageMime('a.webp')).toBe('image/webp')
    expect(imageMime('a.gif')).toBe('image/gif')
    expect(imageMime('a.svg')).toBe('')
  })
})
