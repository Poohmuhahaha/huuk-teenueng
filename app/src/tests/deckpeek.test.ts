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
import { deckPeek } from '@/core/deck'

describe('ScreensDeck live peek', () => {
  beforeEach(() => {
    replace.mockClear()
    deckPeek.value = null
    vi.useFakeTimers()
  })

  /** Two 370px cards in a 390px frame: B owns the frame past scrollLeft 190. */
  function layout(w: ReturnType<typeof mount>): HTMLElement {
    const deck = w.find('.deck').element as HTMLElement
    let sl = 0
    Object.defineProperty(deck, 'scrollLeft', {
      get: () => sl,
      set: (v: number) => { sl = v },
      configurable: true,
    })
    Object.defineProperty(deck, 'clientWidth', { get: () => 390, configurable: true })
    const cards = w.findAll('.slide:not(.spacer)')
    Object.defineProperty(cards[0].element, 'offsetLeft', { get: () => 0, configurable: true })
    Object.defineProperty(cards[0].element, 'offsetWidth', { get: () => 370, configurable: true })
    Object.defineProperty(cards[1].element, 'offsetLeft', { get: () => 380, configurable: true })
    Object.defineProperty(cards[1].element, 'offsetWidth', { get: () => 370, configurable: true })
    return deck
  }

  async function scrollTo(w: ReturnType<typeof mount>, left: number): Promise<void> {
    const deck = w.find('.deck')
    ;(deck.element as HTMLElement).scrollLeft = left
    await deck.trigger('scroll')
  }

  it('peeks the incoming card the moment it covers over 51% of the frame', async () => {
    const w = mount(ScreensDeck)
    layout(w)
    await scrollTo(w, 0)
    expect(deckPeek.value).toBe('/a')
    // B visible: (590-380)/390 = 53.8% → peek flips mid-drag, no release.
    await scrollTo(w, 200)
    expect(deckPeek.value).toBe('/b')
    expect(replace).not.toHaveBeenCalled()
    w.unmount()
  })

  it('holds the previous peek at the midpoint so the label never flickers', async () => {
    const w = mount(ScreensDeck)
    layout(w)
    await scrollTo(w, 200)
    expect(deckPeek.value).toBe('/b')
    // B visible: (575-380)/390 = 50.0% — neither card owns the frame.
    await scrollTo(w, 185)
    expect(deckPeek.value).toBe('/b')
    w.unmount()
  })

  it('peeks back when the drag returns without crossing', async () => {
    const w = mount(ScreensDeck)
    layout(w)
    await scrollTo(w, 120)
    // A visible: (370-120)/390 = 64% → still A, label never moved.
    expect(deckPeek.value).toBe('/a')
    w.unmount()
  })
})
