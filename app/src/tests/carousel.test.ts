// @vitest-environment happy-dom
import { afterEach, describe, expect, it, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import CarouselTabs from '@/components/ui/CarouselTabs.vue'

// happy-dom has no Pointer Capture; stub it so we can assert when it is used.
const captureDescriptor = Object.getOwnPropertyDescriptor(HTMLElement.prototype, 'setPointerCapture')

function stubPointerCapture(): ReturnType<typeof vi.fn> {
  const capture = vi.fn()
  Object.defineProperty(HTMLElement.prototype, 'setPointerCapture', {
    configurable: true,
    writable: true,
    value: capture,
  })
  return capture
}

afterEach(() => {
  if (captureDescriptor) Object.defineProperty(HTMLElement.prototype, 'setPointerCapture', captureDescriptor)
  else delete (HTMLElement.prototype as { setPointerCapture?: unknown }).setPointerCapture
})

function drag(w: ReturnType<typeof mount>, from: number, to: number): Promise<void> {
  const track = w.find('.tabs')
  return (async () => {
    await track.trigger('pointerdown', { clientX: from, pointerId: 1 })
    await track.trigger('pointermove', { clientX: to, pointerId: 1 })
    await track.trigger('pointerup', { clientX: to, pointerId: 1 })
  })()
}

describe('CarouselTabs slide (M0)', () => {
  it('slides left to advance one screen', async () => {
    const w = mount(CarouselTabs, { props: { items: [1, 2, 3], modelValue: 1 } })
    await drag(w, 100, 40)
    expect(w.emitted('update:modelValue')?.[0]).toEqual([2])
  })

  it('slides right to go back one screen', async () => {
    const w = mount(CarouselTabs, { props: { items: [1, 2, 3], modelValue: 2 } })
    await drag(w, 40, 100)
    expect(w.emitted('update:modelValue')?.[0]).toEqual([1])
  })

  it('ignores short drags under the 40px threshold', async () => {
    const w = mount(CarouselTabs, { props: { items: [1, 2, 3], modelValue: 1 } })
    await drag(w, 100, 90)
    expect(w.emitted('update:modelValue')).toBeFalsy()
  })

  it('stops at the ends', async () => {
    const w = mount(CarouselTabs, { props: { items: [1, 2, 3], modelValue: 3 } })
    await drag(w, 100, 40)
    expect(w.emitted('update:modelValue')?.[0]).toEqual([3])
  })

  it('arrow keys step forward and back', async () => {
    const w = mount(CarouselTabs, { props: { items: [1, 2, 3], modelValue: 2 } })
    await w.find('.tabs').trigger('keydown', { key: 'ArrowRight' })
    expect(w.emitted('update:modelValue')?.[0]).toEqual([3])
    await w.find('.tabs').trigger('keydown', { key: 'ArrowLeft' })
    expect(w.emitted('update:modelValue')?.[1]).toEqual([1])
  })

  // Chrome retargets `click` to the element that captured the pointer, so
  // capturing on pointerdown broke every tab click. Capture only on drag.
  it('does not capture on a plain tap and still selects the tab', async () => {
    const capture = stubPointerCapture()
    const w = mount(CarouselTabs, { props: { items: [1, 2, 3], modelValue: 1 } })
    await w.find('.tabs').trigger('pointerdown', { clientX: 100, pointerId: 1 })
    expect(capture).not.toHaveBeenCalled()
    await w.findAll('.tab')[2].trigger('click')
    expect(w.emitted('update:modelValue')?.[0]).toEqual([3])
  })

  it('captures only once the drag threshold is crossed', async () => {
    const capture = stubPointerCapture()
    const w = mount(CarouselTabs, { props: { items: [1, 2, 3], modelValue: 1 } })
    const track = w.find('.tabs')
    await track.trigger('pointerdown', { clientX: 100, pointerId: 1 })
    await track.trigger('pointermove', { clientX: 95, pointerId: 1 })
    expect(capture).not.toHaveBeenCalled()
    await track.trigger('pointermove', { clientX: 60, pointerId: 1 })
    expect(capture).toHaveBeenCalledTimes(1)
  })

  it('lets touch swipe natively instead of stepping', async () => {
    const w = mount(CarouselTabs, { props: { items: [1, 2, 3], modelValue: 1 } })
    const track = w.find('.tabs')
    await track.trigger('pointerdown', { pointerType: 'touch', clientX: 100, pointerId: 1 })
    await track.trigger('pointermove', { pointerType: 'touch', clientX: 40, pointerId: 1 })
    await track.trigger('pointerup', { pointerType: 'touch', clientX: 40, pointerId: 1 })
    expect(w.emitted('update:modelValue')).toBeFalsy()
  })
})
