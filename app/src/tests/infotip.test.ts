// @vitest-environment happy-dom
import { describe, expect, it } from 'vitest'
import { mount } from '@vue/test-utils'
import InfoTip from '@/components/ui/InfoTip.vue'

describe('InfoTip', () => {
  it('hides the hint until the (i) button is clicked', async () => {
    const w = mount(InfoTip, { props: { text: 'When on, Huuk can pause campaigns.' } })
    expect(w.find('.infotip-bubble').exists()).toBe(false)
    expect(w.find('.infotip').attributes('aria-expanded')).toBe('false')

    await w.find('.infotip').trigger('click')
    expect(w.find('.infotip-bubble').exists()).toBe(true)
    expect(w.find('.infotip-bubble').text()).toContain('pause campaigns')
    expect(w.find('.infotip').attributes('aria-expanded')).toBe('true')

    await w.find('.infotip').trigger('click')
    expect(w.find('.infotip-bubble').exists()).toBe(false)
    w.unmount()
  })
})
