// @vitest-environment happy-dom
// Brand Identity page: logo slots and the moodboard are interactive (upload or
// paste a URL, remove), and uploaded images come back as usable URLs.
import { beforeEach, describe, expect, it } from 'vitest'
import { flushPromises, mount } from '@vue/test-utils'
import { QueryClient, VueQueryPlugin } from '@tanstack/vue-query'
import BrandPage from '@/pages/BrandPage.vue'
import router from '@/app/router'
import * as api from '@/mock/api'
import { brand } from '@/mock/db'

const settle = (ms = 400): Promise<void> => new Promise((r) => setTimeout(r, ms))

/** Base64 of a PNG header with the given dimensions (parsers read the IHDR). */
function pngHeader(width: number, height: number): string {
  const bytes = new Uint8Array(24)
  bytes.set([0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a], 0)
  const view = new DataView(bytes.buffer)
  view.setUint32(8, 13)
  bytes.set([0x49, 0x48, 0x44, 0x52], 12)
  view.setUint32(16, width)
  view.setUint32(20, height)
  return Buffer.from(bytes).toString('base64')
}

const PNG_500 = pngHeader(500, 500)

function mountBrand() {
  const qc = new QueryClient({ defaultOptions: { queries: { retry: false } } })
  return mount(BrandPage, {
    global: { plugins: [router, [VueQueryPlugin, { queryClient: qc }]] },
  })
}

describe('BrandPage images', () => {
  beforeEach(() => {
    brand.logos = []
    brand.moodboard = []
  })

  it('renders the three logo slots and the moodboard', async () => {
    const w = mountBrand()
    await settle()
    await flushPromises()
    expect(w.findAll('.logo-slot')).toHaveLength(3)
    expect(w.text()).toContain('Main logo')
    expect(w.text()).toContain('Secondary logo')
    expect(w.text()).toContain('Social logo')
    expect(w.find('.moodboard').exists()).toBe(true)
    expect(w.find('.mood-add').exists()).toBe(true)
    w.unmount()
  })

  it('stores a pasted logo URL and a moodboard URL, and removes them', async () => {
    const w = mountBrand()
    await settle()
    await flushPromises()

    const logoInputs = w.findAll('.logo-slot input.field')
    await logoInputs[0].setValue('https://cdn.example/logo.png')
    await logoInputs[0].trigger('change')
    await settle()
    await flushPromises()
    expect(brand.logos[0]).toBe('https://cdn.example/logo.png')
    expect(w.find('.logo-preview img').exists()).toBe(true)

    await w.find('input.mood-url').setValue('https://cdn.example/ref.jpg')
    await w.find('.mood-add-btn').trigger('click')
    await settle()
    await flushPromises()
    expect(brand.moodboard).toEqual(['https://cdn.example/ref.jpg'])
    expect(w.findAll('.mood-item')).toHaveLength(1)

    await w.find('.mood-item .logo-remove').trigger('click')
    await settle()
    await flushPromises()
    expect(brand.moodboard).toEqual([])

    await w.find('.logo-preview .logo-remove').trigger('click')
    await settle()
    await flushPromises()
    expect(brand.logos[0]).toBe('')
    w.unmount()
  })

  it('turns an uploaded image into a data URL and rejects other types', async () => {
    const { url, width, height } = await api.uploadBrandImage('logo.png', PNG_500, 'logo')
    expect(url).toBe(`data:image/png;base64,${PNG_500}`)
    expect([width, height]).toEqual([500, 500])
    await expect(api.uploadBrandImage('logo.svg', PNG_500, 'logo')).rejects.toThrow(/unsupported/i)
  })

  it('enforces the 500x500 cap before uploading', async () => {
    await expect(api.uploadBrandImage('small.png', pngHeader(16, 16), 'logo')).rejects.toThrow(
      /at least 32×32/,
    )
    await expect(api.uploadBrandImage('big.png', pngHeader(900, 900), 'logo')).rejects.toThrow(
      /at most 500×500/,
    )
    await expect(
      api.uploadBrandImage('wide.png', pngHeader(800, 200), 'moodboard'),
    ).rejects.toThrow(/at most 500×500/)
    const ok = await api.uploadBrandImage('ref.png', pngHeader(500, 500), 'moodboard')
    expect([ok.width, ok.height]).toEqual([500, 500])
  })

  it('offers a resize action for an oversized stored image', async () => {
    brand.logos = ['/api/brand/images/old-big.jpg']
    const w = mountBrand()
    await settle()
    await flushPromises()

    // happy-dom does not decode images, so simulate the browser reporting a
    // 1200x1200 render for the stored file.
    const img = w.find('.logo-preview img')
    expect(img.exists()).toBe(true)
    Object.defineProperty(img.element, 'naturalWidth', { value: 1200, configurable: true })
    Object.defineProperty(img.element, 'naturalHeight', { value: 1200, configurable: true })
    await img.trigger('load')
    await flushPromises()

    const fix = w.find('.oversize-fix')
    expect(fix.exists()).toBe(true)
    expect(fix.text()).toContain('Resize this image')
    w.unmount()
  })

  it('edits the style tokens and exports the brand kit', async () => {
    const w = mountBrand()
    await settle()
    await flushPromises()

    // Corner radius slider + number input stay in sync and persist.
    const radius = w.find('.style-row input.style-num[type=number]')
    await radius.setValue('4')
    await radius.trigger('change')
    await settle()
    await flushPromises()
    expect(brand.radius).toBe(4)

    // Shadow segmented control.
    const strong = w.findAll('.style-segmented button').find((b) => b.text() === 'Strong')!
    await strong.trigger('click')
    await settle()
    await flushPromises()
    expect(brand.shadow).toBe('strong')

    // Export: copy the brand CSS.
    const written: string[] = []
    Object.defineProperty(navigator, 'clipboard', {
      configurable: true,
      value: { writeText: async (text: string) => { written.push(text); return undefined } },
    })
    const copy = w.findAll('.style-export button').find((b) => b.text() === 'Copy CSS')!
    await copy.trigger('click')
    await settle()
    await flushPromises()
    expect(written).toHaveLength(1)
    expect(written[0]).toContain(`--accent: ${brand.palette[0]};`)
    expect(written[0]).toContain('--radius: 4px;')
    expect(written[0]).toContain('--shadow-depth: strong;')
    expect(w.text()).toContain('copied to the clipboard')
    w.unmount()
  })

  it('shows the allowed size range for each slot', async () => {
    const w = mountBrand()
    await settle()
    await flushPromises()
    expect(w.text()).toContain('32×32 – 500×500 px')
    w.unmount()
  })
})
