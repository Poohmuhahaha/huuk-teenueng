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

  it('renders vertical square towers (variant="towers")', () => {
    const wrapper = mount(BarChart, {
      props: {
        items: sampleItems,
        variant: 'towers',
        currency: '฿',
        card: false,
      },
    })
    expect(wrapper.text()).toContain('Jan')
    expect(wrapper.text()).toContain("'26")
    expect(wrapper.text()).toContain('Feb')
    expect(wrapper.text()).toContain('Sep')
    expect(wrapper.text()).toContain('Oct')

    expect(wrapper.text()).toContain('฿15,000')
    expect(wrapper.text()).toContain('฿34,600')

    const activeSquares = wrapper.findAll('.tower-square-active')
    const ghostSquares = wrapper.findAll('.tower-square-ghost')
    expect(activeSquares.length).toBeGreaterThan(0)
    expect(ghostSquares.length).toBeGreaterThan(0)
    expect(activeSquares.length + ghostSquares.length).toBe(32)

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
    // Month legend markers
    expect(wrapper.text()).toContain('Jan')
    expect(wrapper.text()).toContain("'26")
    expect(wrapper.text()).toContain('Oct')
    expect(wrapper.text()).toContain('฿15,000')
    expect(wrapper.text()).toContain('฿34,600')

    // SVG wave elements
    const svg = wrapper.find('svg')
    expect(svg.exists()).toBe(true)
    const paths = wrapper.findAll('path')
    expect(paths.length).toBe(2) // 1 for gradient area, 1 for stroke line
    const dots = wrapper.findAll('circle')
    expect(dots.length).toBe(4) // 4 month dots

    expect(wrapper.classes()).toContain('w-full')
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
