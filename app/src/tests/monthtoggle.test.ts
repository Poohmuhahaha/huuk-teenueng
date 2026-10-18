// @vitest-environment happy-dom
import { describe, expect, it } from 'vitest'
import { mount } from '@vue/test-utils'
import MonthToggle from '@/components/ui/MonthToggle.vue'

function mountToggle(props: Record<string, unknown> = {}) {
  return mount(MonthToggle, {
    props: { modelValue: 7, year: 2026, ...props },
    // Render the teleported popup inline so wrapper.find() can reach it,
    // and stub the slide-up transition so open/close apply synchronously.
    global: { stubs: { teleport: true, transition: true } },
  })
}

describe('MonthToggle', () => {
  it('shows month and year on the label', () => {
    const w = mountToggle()
    expect(w.find('.month-toggle').text()).toContain('July 2026')
    expect(w.find('button[aria-label="Previous month"]').exists()).toBe(false)
    expect(w.find('button[aria-label="Next month"]').exists()).toBe(false)
    w.unmount()
  })

  it('opens a 12-month grid and jumps straight to January', async () => {
    const w = mountToggle()
    expect(w.find('.month-modal').exists()).toBe(false)
    await w.find('.month-label').trigger('click')
    expect(w.find('.month-modal').exists()).toBe(true)
    expect(w.findAll('.month-cell').length).toBe(12)
    await w.findAll('.month-cell').find((b) => b.text() === 'January')!.trigger('click')
    expect(w.emitted('update:modelValue')).toEqual([[1]])
    expect(w.find('.month-modal').exists()).toBe(false)
    w.unmount()
  })

  it('steps the year without closing the grid', async () => {
    const w = mountToggle()
    await w.find('.month-label').trigger('click')
    await w.find('button[aria-label="Previous year"]').trigger('click')
    expect(w.emitted('update:year')).toEqual([[2025]])
    // Still open — the user can now pick a month in the new year.
    expect(w.find('.month-modal').exists()).toBe(true)
    await w.find('button[aria-label="Next year"]').trigger('click')
    expect(w.emitted('update:year')).toEqual([[2025], [2027]])
    w.unmount()
  })

  it('closes via backdrop or Escape without emitting', async () => {
    const w = mountToggle()
    await w.find('.month-label').trigger('click')
    await w.find('.backdrop').trigger('click')
    expect(w.find('.month-modal').exists()).toBe(false)

    await w.find('.month-label').trigger('click')
    window.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape' }))
    await w.vm.$nextTick()
    expect(w.find('.month-modal').exists()).toBe(false)
    expect(w.emitted('update:modelValue')).toBeUndefined()
    w.unmount()
  })
})
