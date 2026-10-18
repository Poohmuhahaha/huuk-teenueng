// @vitest-environment happy-dom
import { describe, expect, it, vi } from 'vitest'
import { mount } from '@vue/test-utils'

vi.mock('vue-router', () => ({
  useRoute: () => ({ path: '/b', query: {}, params: {} }),
  useRouter: () => ({ replace: vi.fn() }),
}))

vi.mock('@/core/screens', () => ({
  screens: [
    {
      path: '/a', to: '/a', label: 'A', match: ['/a'], sheet: '', access: '',
      component: { template: '<div>A</div>' },
    },
    {
      path: '/b', to: '/b', label: 'B', match: ['/b'], sheet: '', access: '',
      component: { template: '<div>B</div>' },
    },
  ],
  screenIndexOf: () => 0,
}))

import ScreensDeck from '@/components/layout/ScreensDeck.vue'

describe('ScreensDeck boot index', () => {
  it('paints the route card active on first render, never card 0', () => {
    const w = mount(ScreensDeck)
    const slides = w.findAll('.slide:not(.spacer)')
    expect(slides[1].classes()).toContain('active')
    expect(slides[0].classes()).not.toContain('active')
    w.unmount()
  })
})
