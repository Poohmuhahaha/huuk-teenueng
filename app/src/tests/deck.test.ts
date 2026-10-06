// @vitest-environment happy-dom
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { mount } from '@vue/test-utils'

const replace = vi.fn()

vi.mock('vue-router', () => ({
  useRoute: () => ({ path: '/a', query: {}, params: {} }),
  useRouter: () => ({ replace }),
}))

vi.mock('@/core/screens', () => ({
  screens: [
    {
      path: '/a', to: '/a', label: 'A', match: ['/a'], sheet: '', access: '',
      component: { template: '<div><input class="probe" /></div>' },
    },
    {
      path: '/b', to: '/b', label: 'B', match: ['/b'], sheet: '', access: '',
      component: { template: '<div>B</div>' },
    },
  ],
  screenIndexOf: () => 0,
}))

import ScreensDeck from '@/components/layout/ScreensDeck.vue'

describe('ScreensDeck keyboard handling', () => {
  beforeEach(() => replace.mockClear())

  function activeIndex(w: ReturnType<typeof mount>): number {
    return w.findAll('.slide:not(.spacer)').findIndex((slide) => slide.classes().includes('active'))
  }

  it('never steals arrow keys from inputs inside a slide', async () => {
    const w = mount(ScreensDeck)
    expect(activeIndex(w)).toBe(0)

    await w.find('.probe').trigger('keydown', { key: 'ArrowRight' })
    expect(activeIndex(w)).toBe(0)
    expect(replace).not.toHaveBeenCalled()
  })

  it('still slides with arrow keys when the deck itself is focused', async () => {
    const w = mount(ScreensDeck)
    await w.find('.deck').trigger('keydown', { key: 'ArrowRight' })
    expect(activeIndex(w)).toBe(1)
  })

  it('scrolls natively without starting a pointer drag (touch or mouse)', async () => {
    const w = mount(ScreensDeck)
    const deck = w.find('.deck')
    for (const pointerType of ['touch', 'mouse']) {
      await deck.trigger('pointerdown', { pointerType, button: 0, clientX: 100 })
      expect(deck.classes()).not.toContain('dragging')
      await deck.trigger('pointermove', { pointerType, clientX: 40 })
      await deck.trigger('pointerup', { pointerType, clientX: 40 })
    }
    expect(replace).not.toHaveBeenCalled()
  })

  it('flips one card per horizontal touchpad gesture, ignores vertical scroll', async () => {
    const w = mount(ScreensDeck)
    const deck = w.find('.deck')
    await deck.trigger('wheel', { deltaX: 1, deltaY: 40 })
    expect(replace).not.toHaveBeenCalled()
    await deck.trigger('wheel', { deltaX: 40, deltaY: 2 })
    expect(replace).toHaveBeenCalledWith(expect.objectContaining({ path: '/b' }))
  })

  it('no longer renders the deck meta row or the dots', () => {
    const w = mount(ScreensDeck)
    expect(w.find('.deck-meta').exists()).toBe(false)
    expect(w.find('.dots').exists()).toBe(false)
  })

  it('flanks the deck with two invisible spacer slides outside card indexing', async () => {
    const w = mount(ScreensDeck)
    expect(w.findAll('.slide.spacer')).toHaveLength(2)
    expect(w.findAll('.slide:not(.spacer)')).toHaveLength(2)
    // Arrow-key navigation still addresses real cards only.
    await w.find('.deck').trigger('keydown', { key: 'ArrowRight' })
    expect(activeIndex(w)).toBe(1)
  })
})
