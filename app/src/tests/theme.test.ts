// @vitest-environment happy-dom
// CI theming: Brand Identity palette/fonts drive the site's CSS variables.
import { afterEach, describe, expect, it, vi } from 'vitest'
import { defineComponent } from 'vue'
import { mount } from '@vue/test-utils'
import { QueryClient, VueQueryPlugin } from '@tanstack/vue-query'
import App from '@/app/App.vue'
import BrandPage from '@/pages/BrandPage.vue'
import router from '@/app/router'
import * as api from '@/mock/api'
import { qk, useSaveBrand } from '@/core/queries'
import { applyBrandTheme, brandStyle } from '@/core/theme'
import type { Brand } from '@/mock/db'

function brand(patch: Partial<Brand>): Brand {
  return {
    channel: '', positioning: '', slogan: '', audience: '', voice: '',
    dos: [], donts: [], palette: [], fonts: [], logos: [], moodboard: [], radius: 12, fillOpacity: 100, strokeWidth: 1, shadow: 'soft', ...patch,
  }
}

function mountWithPlugins(component: unknown) {
  const qc = new QueryClient({ defaultOptions: { queries: { retry: false } } })
  return mount(component as never, {
    global: { plugins: [router, [VueQueryPlugin, { queryClient: qc }]] },
  })
}

afterEach(async () => {
  applyBrandTheme(null)
  await api.saveBrand({ palette: [], fonts: [] }).catch(() => undefined)
})

describe('applyBrandTheme', () => {
  it('maps the first palette color to the accent family', () => {
    const root = document.createElement('div')
    applyBrandTheme(brand({ palette: ['#ff0000', '#00ff00'] }), [], root)
    expect(root.style.getPropertyValue('--accent')).toBe('#ff0000')
    expect(root.style.getPropertyValue('--accent-ink')).toBe('#ffffff')
    expect(root.style.getPropertyValue('--accent-hover')).toBeTruthy()
    expect(root.style.getPropertyValue('--accent-wash')).toBeTruthy()
  })

  it('uses dark ink for light accents', () => {
    const root = document.createElement('div')
    applyBrandTheme(brand({ palette: ['#ffe066'] }), [], root)
    expect(root.style.getPropertyValue('--accent-ink')).toBe('#000000')
  })

  it('applies a sanitized font family', () => {
    const root = document.createElement('div')
    applyBrandTheme(brand({ fonts: ['Kanit'] }), [], root)
    expect(root.style.getPropertyValue('--font')).toContain('"Kanit"')
  })

  it('injects @font-face for an imported font and removes it when cleared', () => {
    const root = document.createElement('div')
    const asset = {
      name: 'kanit.woff2', family: 'Kanit', url: '/api/brand/fonts/kanit.woff2/file',
      size: 1000, uploadedAt: 0,
    }
    applyBrandTheme(brand({ fonts: ['Kanit'] }), [asset], root)
    const face = document.getElementById('huuk-brand-font')
    expect(face?.textContent).toContain('@font-face')
    expect(face?.textContent).toContain('kanit.woff2/file')
    expect(root.style.getPropertyValue('--font')).toContain('"Kanit"')

    applyBrandTheme(brand({}), [], root)
    expect(document.getElementById('huuk-brand-font')).toBeNull()
    expect(root.style.getPropertyValue('--font')).toBe('')
  })

  it('exposes every palette slot as a brand variable for charts', () => {
    const root = document.createElement('div')
    applyBrandTheme(brand({ palette: ['#008ba3', '#359a19', '#d2d0eb', '#bfbde5'] }), [], root)
    expect(root.style.getPropertyValue('--brand-1')).toBe('#008ba3')
    expect(root.style.getPropertyValue('--brand-2')).toBe('#359a19')
    expect(root.style.getPropertyValue('--brand-3')).toBe('#d2d0eb')
    expect(root.style.getPropertyValue('--brand-4')).toBe('#bfbde5')

    // Fewer colors than slots: the missing ones are cleared.
    applyBrandTheme(brand({ palette: ['#008ba3'] }), [], root)
    expect(root.style.getPropertyValue('--brand-1')).toBe('#008ba3')
    expect(root.style.getPropertyValue('--brand-2')).toBe('')

    applyBrandTheme(brand({ palette: [] }), [], root)
    expect(root.style.getPropertyValue('--brand-1')).toBe('')
  })

  it('applies the style tokens to the site', () => {
    const root = document.createElement('div')
    applyBrandTheme(
      brand({ palette: ['#008ba3'], radius: 4, fillOpacity: 50, strokeWidth: 0, shadow: 'strong' }),
      [], root,
    )
    expect(root.style.getPropertyValue('--radius')).toBe('4px')
    expect(root.style.getPropertyValue('--radius-sm')).toBe('3px')
    expect(root.style.getPropertyValue('--radius-lg')).toBe('5px')
    expect(root.style.getPropertyValue('--stroke-w')).toBe('0px')
    expect(root.style.getPropertyValue('--shadow-lg')).not.toBe('')
    expect(root.style.getPropertyValue('--accent-strong')).not.toBe('#008ba3')
    expect(root.style.getPropertyValue('--accent-strong-ink')).not.toBe('')

    // The defaults (and the soft tier) clear the overrides.
    applyBrandTheme(brand({ palette: ['#008ba3'], radius: 12, fillOpacity: 100, strokeWidth: 1, shadow: 'soft' }), [], root)
    expect(root.style.getPropertyValue('--radius')).toBe('12px')
    expect(root.style.getPropertyValue('--shadow-lg')).toBe('')

    // Older brands keep the defaults.
    applyBrandTheme(brand({ palette: ['#008ba3'], radius: undefined, shadow: undefined } as never), [], root)
    expect(root.style.getPropertyValue('--radius')).toBe('12px')
    expect(root.style.getPropertyValue('--shadow-lg')).toBe('')
  })

  it('clamps style token ranges', () => {
    expect(brandStyle({ radius: 99, fillOpacity: 3, strokeWidth: 9, shadow: 'nope' } as never)).toEqual({
      radius: 24, fillOpacity: 100, strokeWidth: 3, shadow: 'soft',
    })
  })

  it('ignores invalid colors/fonts and clears overrides when the brand is empty', () => {
    const root = document.createElement('div')
    applyBrandTheme(brand({ palette: ['red; background: url(x)'], fonts: ['Bad;font{}'] }), [], root)
    expect(root.style.getPropertyValue('--accent')).toBe('')
    expect(root.style.getPropertyValue('--font')).toBe('')

    applyBrandTheme(brand({ palette: ['#123456'], fonts: ['Kanit'] }), [], root)
    expect(root.style.getPropertyValue('--accent')).toBe('#123456')

    applyBrandTheme(brand({}), [], root)
    expect(root.style.getPropertyValue('--accent')).toBe('')
    expect(root.style.getPropertyValue('--font')).toBe('')
  })
})

describe('brand identity editing', () => {
  it('saves palette colors from the Brand Identity page', async () => {
    await router.push('/brand')
    await router.isReady()
    const w = mountWithPlugins(BrandPage)
    // Wait for setup so the inputs are enabled (canWrite).
    await vi.waitFor(() => {
      const input = w.find('.palette-field input.field')
      expect(input.exists()).toBe(true)
      expect(input.attributes('disabled')).toBeUndefined()
    }, { timeout: 8000 })

    expect(w.findAll('.palette-card')).toHaveLength(4)
    expect(w.text()).toContain('buttons, links, nav')
    expect(w.text()).toContain('Live preview')
    const hex = w.findAll('.palette-field input.field')[0]
    await hex.setValue('#112233')
    await hex.trigger('change')

    await vi.waitFor(async () => {
      expect((await api.getBrand()).palette[0]).toBe('#112233')
    }, { timeout: 8000 })
    w.unmount()
  }, 15000)

  it('applies the saved brand to the app root', async () => {
    await api.saveBrand({ palette: ['#0ea5e9'], fonts: ['Kanit'] })
    const w = mountWithPlugins(App)
    await vi.waitFor(() => {
      expect(document.documentElement.style.getPropertyValue('--accent')).toBe('#0ea5e9')
    }, { timeout: 5000 })
    expect(document.documentElement.style.getPropertyValue('--font')).toContain('"Kanit"')
    w.unmount()
  })
})

describe('theme live update (regression)', () => {
  it('applies the theme immediately when the brand is saved', async () => {
    const qc = new QueryClient({ defaultOptions: { queries: { retry: false } } })
    const Probe = defineComponent({
      setup() {
        return { save: useSaveBrand() }
      },
      template: '<div />',
    })
    const w = mount(Probe, { global: { plugins: [[VueQueryPlugin, { queryClient: qc }]] } })
    await w.vm.save.mutateAsync({ palette: ['#abcdef'] })
    expect(document.documentElement.style.getPropertyValue('--accent')).toBe('#abcdef')
    w.unmount()
  })

  it('re-applies after the brand is saved and the query invalidated', async () => {
    const qc = new QueryClient({ defaultOptions: { queries: { retry: false } } })
    const w = mount(App as never, {
      global: { plugins: [router, [VueQueryPlugin, { queryClient: qc }]] },
    })
    await vi.waitFor(() => {
      expect(document.documentElement.style.getPropertyValue('--accent')).toBe('')
    }, { timeout: 5000 })

    await api.saveBrand({ palette: ['#123456'] })
    await qc.invalidateQueries({ queryKey: qk.brand })

    await vi.waitFor(() => {
      expect(document.documentElement.style.getPropertyValue('--accent')).toBe('#123456')
    }, { timeout: 5000 })
    w.unmount()
  })
})
