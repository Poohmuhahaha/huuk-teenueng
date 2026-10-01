// @vitest-environment happy-dom
// Brand kit JSON: the redesigned palette cards plus sanitized import/export.
import { afterEach, describe, expect, it, vi } from 'vitest'
import { mount, type DOMWrapper } from '@vue/test-utils'
import { QueryClient, VueQueryPlugin } from '@tanstack/vue-query'
import BrandPage from '@/pages/BrandPage.vue'
import router from '@/app/router'
import * as api from '@/mock/api'
import type { Brand } from '@/mock/db'

function brand(patch: Partial<Brand>): Brand {
  return {
    channel: '', positioning: '', slogan: '', audience: '', voice: '',
    dos: [], donts: [], palette: [], fonts: [], logos: [], moodboard: [],
    radius: 12, fillOpacity: 100, strokeWidth: 1, shadow: 'soft', ...patch,
  }
}

function mountWithPlugins(component: unknown) {
  const qc = new QueryClient({ defaultOptions: { queries: { retry: false } } })
  return mount(component as never, {
    global: { plugins: [router, [VueQueryPlugin, { queryClient: qc }]] },
  })
}

/** BrandPage mounted on /brand, waiting until setup loaded so inputs are enabled. */
async function openBrandPage() {
  await router.push('/brand')
  await router.isReady()
  const w = mountWithPlugins(BrandPage)
  await vi.waitFor(() => {
    const input = w.find('.palette-field input.field')
    expect(input.exists()).toBe(true)
    expect(input.attributes('disabled')).toBeUndefined()
  }, { timeout: 8000 })
  return w
}

/** The hidden file input of the Export row (accept=".json,application/json"). */
function brandJsonInput(w: ReturnType<typeof mountWithPlugins>): DOMWrapper<Element> {
  const input = w
    .findAll('input[type="file"]')
    .find((el) => (el.attributes('accept') ?? '').includes('json'))
  if (!input) throw new Error('brand-kit JSON file input not found')
  return input
}

async function chooseFile(input: DOMWrapper<Element>, file: File): Promise<void> {
  Object.defineProperty(input.element, 'files', { value: [file], configurable: true })
  await input.trigger('change')
}

afterEach(async () => {
  // Tests in this file mutate the shared in-memory brand: restore a known kit.
  await api.saveBrand(brand({ palette: ['#111111', '#555555', '#999999', '#CCCCCC'] })).catch(() => undefined)
})

describe('brand kit JSON import/export', () => {
  it('renders four redesigned palette cards with roles, samples and copy buttons', async () => {
    const w = await openBrandPage()

    const cards = w.findAll('.palette-card')
    expect(cards).toHaveLength(4)
    for (const card of cards) {
      const tile = card.find('.palette-tile')
      expect(tile.exists()).toBe(true)
      expect(tile.find('.palette-tile-role').exists()).toBe(true)
      expect(tile.find('.palette-tile-sample').exists()).toBe(true)
      expect(card.find('.palette-copy').exists()).toBe(true)
    }
    expect(w.text()).toContain('buttons, links, nav')

    // The Export row still offers both exports plus the JSON import control.
    expect(brandJsonInput(w).exists()).toBe(true)
    const exportButtons = w.findAll('.style-export button').map((b) => b.text())
    expect(exportButtons).toContain('Copy CSS')
    expect(exportButtons).toContain('Download JSON')
    w.unmount()
  }, 15000)

  it('imports a brand-kit JSON, keeping known keys and clamping the rest', async () => {
    await api.saveBrand({ palette: [] })
    const w = await openBrandPage()
    const before = await api.getBrand()

    const payload = {
      palette: ['#112233', '#445566'],
      radius: 99,
      fillOpacity: 2,
      strokeWidth: 9,
      shadow: 'bogus',
      unknownKey: 'x',
      channel: 'Imported',
      dos: ['a', '', 7],
    }
    const file = new File([JSON.stringify(payload)], 'brand-kit.json', { type: 'application/json' })
    await chooseFile(brandJsonInput(w), file)

    await vi.waitFor(() => {
      const status = w.find('.style-export [role="status"]')
      expect(status.exists()).toBe(true)
      expect(status.text()).toContain('Brand kit imported')
    }, { timeout: 8000 })

    const after = await api.getBrand()
    expect(after.palette).toEqual(['#112233', '#445566'])
    expect(after.radius).toBe(24)
    expect(after.fillOpacity).toBe(5)
    expect(after.strokeWidth).toBe(3)
    // An unknown shadow tier is dropped, so the previous valid value survives.
    expect(after.shadow).toBe(before.shadow)
    expect(after.shadow).not.toBe('bogus')
    expect(after.channel).toBe('Imported')
    expect(after.dos).toEqual(['a'])
    w.unmount()
  }, 15000)

  it('rejects a non-JSON file and leaves the saved brand untouched', async () => {
    const w = await openBrandPage()
    const before = await api.getBrand()

    const file = new File(['not json at all'], 'notes.json', { type: 'application/json' })
    await chooseFile(brandJsonInput(w), file)

    await vi.waitFor(() => {
      const alert = w.find('.style-export [role="alert"]')
      expect(alert.exists()).toBe(true)
      expect(alert.text()).toContain('not a brand-kit JSON')
    }, { timeout: 8000 })

    expect(await api.getBrand()).toEqual(before)
    w.unmount()
  }, 15000)
})
