// @vitest-environment happy-dom
import { describe, expect, it } from 'vitest'
import { mount } from '@vue/test-utils'
import BarChart from '@/components/ui/BarChart.vue'

describe('BarChart component', () => {
  const sampleItems = [
    { label: '2026-01', value: 15000 },
    { label: '2026-02', value: 17200 },
    { label: '2026-09', value: 20600 },
    { label: '2026-10', value: 34600 },
  ]

  it('renders empty state when no items provided', () => {
    const wrapper = mount(BarChart, {
      props: { items: [] },
    })
    expect(wrapper.text()).toContain('No data yet.')
  })

  it('renders horizontal bars by default (variant="bars")', () => {
    const wrapper = mount(BarChart, {
      props: { items: sampleItems },
    })
    expect(wrapper.text()).toContain('2026-01')
    expect(wrapper.text()).toContain('15,000')
    expect(wrapper.findAll('.group.border-b').length).toBe(4)
    expect(wrapper.attributes('role')).toBe('img')
  })

  it('renders full 12-month timeline equalizer in towers variant', () => {
    const wrapper = mount(BarChart, {
      props: {
        items: sampleItems,
        variant: 'towers',
        currency: '฿',
        card: false,
      },
    })
    // Full 12-month labels
    expect(wrapper.text()).toContain('Jan')
    expect(wrapper.text()).toContain('Feb')
    expect(wrapper.text()).toContain('Mar')
    expect(wrapper.text()).toContain('Apr')
    expect(wrapper.text()).toContain('May')
    expect(wrapper.text()).toContain('Jun')
    expect(wrapper.text()).toContain('Jul')
    expect(wrapper.text()).toContain('Aug')
    expect(wrapper.text()).toContain('Sep')
    expect(wrapper.text()).toContain('Oct')
    expect(wrapper.text()).toContain('Nov')
    expect(wrapper.text()).toContain('Dec')

    // Values for active months
    expect(wrapper.text()).toContain('฿15,000')
    expect(wrapper.text()).toContain('฿34,600')

    // 12 columns x 8 rows = 96 total squares
    const activeSquares = wrapper.findAll('.tower-square-active')
    const ghostSquares = wrapper.findAll('.tower-square-ghost')
    expect(activeSquares.length).toBeGreaterThan(0)
    expect(ghostSquares.length).toBeGreaterThan(0)
    expect(activeSquares.length + ghostSquares.length).toBe(96)

    // 12-column grid layout
    expect(wrapper.find('.grid-cols-12').exists()).toBe(true)
    expect(wrapper.classes()).toContain('w-full')
    expect(wrapper.classes()).not.toContain('border')
  })

  it('renders edge-to-edge smooth spline area wave (variant="area")', () => {
    const wrapper = mount(BarChart, {
      props: {
        items: sampleItems,
        variant: 'area',
        currency: '฿',
        card: false,
      },
    })
    expect(wrapper.text()).toContain('Jan')
    expect(wrapper.text()).toContain('Oct')
    expect(wrapper.text()).toContain('฿15,000')
    expect(wrapper.text()).toContain('฿34,600')

    const svg = wrapper.find('svg')
    expect(svg.exists()).toBe(true)
    expect(wrapper.findAll('path').length).toBe(2)
    expect(wrapper.findAll('circle').length).toBe(4)
  })

  it('renders card border when card=true', () => {
    const wrapper = mount(BarChart, {
      props: {
        items: sampleItems,
        variant: 'towers',
        card: true,
      },
    })
    expect(wrapper.classes()).toContain('border')
    expect(wrapper.classes()).toContain('rounded-xl')
  })
})
