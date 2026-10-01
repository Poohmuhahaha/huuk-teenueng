// @vitest-environment happy-dom
// Brand font importing/browsing: upload, list, select, delete.
import { afterEach, describe, expect, it, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import { QueryClient, VueQueryPlugin } from '@tanstack/vue-query'
import BrandPage from '@/pages/BrandPage.vue'
import router from '@/app/router'
import * as api from '@/mock/api'

const FAKE_WOFF2 = btoa('wOF2fake')

async function resetFonts(): Promise<void> {
  await api.saveBrand({ fonts: [] }).catch(() => undefined)
  for (const font of await api.listFonts().catch(() => [])) {
    await api.deleteFont(font.name).catch(() => undefined)
  }
}

afterEach(resetFonts)

describe('brand fonts (mock parity)', () => {
  it('uploads, lists and deletes a font', async () => {
    const list = await api.uploadFont('My Brand Font.woff2', FAKE_WOFF2)
    expect(list[0].family).toBe('My Brand Font')
    expect(list[0].url).toContain('/api/brand/fonts/')
    expect(list[0].url.endsWith('/file')).toBe(true)
    expect(list[0].size).toBeGreaterThan(0)

    const listed = await api.listFonts()
    expect(listed.some((f) => f.name === list[0].name)).toBe(true)

    const after = await api.deleteFont(list[0].name)
    expect(after.some((f) => f.name === list[0].name)).toBe(false)
  })

  it('rejects unsupported extensions and empty payloads', async () => {
    await expect(api.uploadFont('font.exe', FAKE_WOFF2)).rejects.toThrow(/unsupported/)
    await expect(api.uploadFont('empty.woff2', '')).rejects.toThrow(/empty/)
  })
})

describe('brand font browser (UI)', () => {
  function mountPage() {
    const qc = new QueryClient({ defaultOptions: { queries: { retry: false } } })
    return mount(BrandPage, {
      global: { plugins: [router, [VueQueryPlugin, { queryClient: qc }]] },
    })
  }

  it('lists imported fonts and selects one as the primary font', async () => {
    await api.uploadFont('Kanit Upload.woff2', FAKE_WOFF2)
    await router.push('/brand')
    await router.isReady()
    const w = mountPage()

    // Wait until setup has loaded and the picker is enabled (canWrite).
    await vi.waitFor(() => {
      const choice = w.findAll('.font-choice').find((c) => c.text().includes('Kanit Upload'))
      expect(choice).toBeTruthy()
      expect(choice?.find('.font-pick').attributes('disabled')).toBeUndefined()
    }, { timeout: 8000 })

    const imported = w.findAll('.font-choice').find((c) => c.text().includes('Kanit Upload'))
    await imported?.find('.font-pick').trigger('click')

    await vi.waitFor(async () => {
      expect((await api.getBrand()).fonts[0]).toBe('Kanit Upload')
    }, { timeout: 8000 })
    w.unmount()
  }, 15000)

  it('suggests common fonts and saves the picked one', async () => {
    await router.push('/brand')
    await router.isReady()
    const w = mountPage()

    await vi.waitFor(() => {
      const choice = w.findAll('.font-choice').find((c) => c.text().trim() === 'Aa Bb 123Kanit')
      expect(choice).toBeTruthy()
      expect(choice?.attributes('disabled')).toBeUndefined()
    }, { timeout: 8000 })

    const pick = w.findAll('.font-choice').find((c) => c.text().trim() === 'Aa Bb 123Kanit')
    await pick?.trigger('click')

    await vi.waitFor(async () => {
      expect((await api.getBrand()).fonts[0]).toBe('Kanit')
    }, { timeout: 8000 })
    w.unmount()
  }, 15000)
})
