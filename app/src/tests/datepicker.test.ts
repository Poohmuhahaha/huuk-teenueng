// @vitest-environment happy-dom
import { describe, expect, it } from 'vitest'
import { mount } from '@vue/test-utils'
import DatePickerPopup from '@/components/ui/DatePickerPopup.vue'

function mountPicker(props: Record<string, unknown> = {}) {
  return mount(DatePickerPopup, {
    props: { modelValue: null, ...props },
    // Render the teleported popup inline so wrapper.find() can reach it.
    global: { stubs: { teleport: true } },
  })
}

describe('DatePickerPopup', () => {
  it('shows a placeholder when empty and opens the calendar on click', async () => {
    const w = mountPicker()
    expect(w.find('.dp-field').text()).toMatch(/select date/i)
    expect(w.find('.dp-modal').exists()).toBe(false)
    await w.find('.dp-field').trigger('click')
    expect(w.find('.dp-modal').exists()).toBe(true)
    expect(w.findAll('.dp-day').length).toBeGreaterThan(27)
    w.unmount()
  })

  it('opens on the selected month and emits YYYY-MM-DD when a day is picked', async () => {
    const w = mountPicker({ modelValue: '2026-03-15' })
    expect(w.find('.dp-field').text()).toContain('2026-03-15')
    await w.find('.dp-field').trigger('click')
    expect(w.find('.dp-head').text()).toMatch(/march 2026/i)
    expect(w.find('.dp-day.selected').text()).toBe('15')
    await w.findAll('.dp-day').find((b) => b.text() === '20')!.trigger('click')
    expect(w.emitted('update:modelValue')).toEqual([['2026-03-20']])
    expect(w.find('.dp-modal').exists()).toBe(false)
    w.unmount()
  })

  it('clears the value and closes via backdrop or Escape', async () => {
    const w = mountPicker({ modelValue: '2026-03-15' })
    await w.find('.dp-field').trigger('click')
    await w.findAll('.dp-modal .actions .btn')[0].trigger('click')
    expect(w.emitted('update:modelValue')).toEqual([[null]])

    await w.find('.dp-field').trigger('click')
    expect(w.find('.dp-modal').exists()).toBe(true)
    await w.find('.backdrop').trigger('click')
    expect(w.find('.dp-modal').exists()).toBe(false)

    await w.find('.dp-field').trigger('click')
    window.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape' }))
    await w.vm.$nextTick()
    expect(w.find('.dp-modal').exists()).toBe(false)
    w.unmount()
  })

  it('steps months and picks today', async () => {
    const w = mountPicker({ modelValue: '2026-03-15' })
    await w.find('.dp-field').trigger('click')
    const nav = w.findAll('.dp-nav')
    await nav[1].trigger('click')
    expect(w.find('.dp-head').text()).toMatch(/april 2026/i)
    await nav[0].trigger('click')
    await nav[0].trigger('click')
    expect(w.find('.dp-head').text()).toMatch(/february 2026/i)

    const n = new Date()
    const today = `${n.getFullYear()}-${String(n.getMonth() + 1).padStart(2, '0')}-${String(n.getDate()).padStart(2, '0')}`
    await w.findAll('.dp-modal .actions .btn')[1].trigger('click')
    expect(w.emitted('update:modelValue')).toEqual([[today]])
    w.unmount()
  })

  it('stays closed when disabled', async () => {
    const w = mountPicker({ disabled: true })
    await w.find('.dp-field').trigger('click')
    expect(w.find('.dp-modal').exists()).toBe(false)
    w.unmount()
  })
})
