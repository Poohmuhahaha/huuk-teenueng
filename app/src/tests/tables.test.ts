// @vitest-environment happy-dom
import { describe, expect, it } from 'vitest'
import { mount } from '@vue/test-utils'
import MasterTable from '@/components/tables/MasterTable.vue'
import TxnTable from '@/components/tables/TxnTable.vue'
import type { Post, Txn } from '@/mock/db'

function post(over: Partial<Post> & { id: string; topic: string }): Post {
  return {
    month: 2, pillar: 'Pillar I', format: 'Short-video', goal: '10K subscribe',
    date: '2026-02-03', time: '08:00', status: 'Start', hook: '', caption: '',
    cta: '', hashtagGroup: 'Reach', hashtags: [], imageUrl: '', note: '',
    done: false, platforms: ['Instagram'], ...over,
  }
}

const rows: Post[] = [
  post({ id: 'a', topic: 'Morning Vlog' }),
  post({ id: 'b', topic: 'Budget Breakfast' }),
  post({ id: 'c', topic: 'Desk Setup Tour' }),
]

describe('MasterTable (TanStack Table v9)', () => {
  it('renders one row per post', () => {
    const w = mount(MasterTable, { props: { rows } })
    expect(w.findAll('tbody tr')).toHaveLength(3)
  })

  it('sorts by topic ascending then descending on header click', async () => {
    const w = mount(MasterTable, { props: { rows } })
    const topicTh = w.findAll('thead th')[1]
    await topicTh.trigger('click')
    expect(w.findAll('tbody tr')[0].text()).toContain('Budget Breakfast')
    await topicTh.trigger('click')
    expect(w.findAll('tbody tr')[0].text()).toContain('Morning Vlog')
  })

  it('filters rows by search text', async () => {
    const w = mount(MasterTable, { props: { rows } })
    await w.find('input[placeholder="Search topic"]').setValue('desk')
    expect(w.findAll('tbody tr')).toHaveLength(1)
    expect(w.find('tbody tr').text()).toContain('Desk Setup Tour')
  })

  it('emits select with the clicked post', async () => {
    const w = mount(MasterTable, { props: { rows } })
    await w.findAll('tbody tr')[1].trigger('click')
    expect(w.emitted('select')).toBeTruthy()
    expect((w.emitted('select') as Post[][])[0][0].id).toBe('b')
  })

  it('toggles row checkboxes and select-all', async () => {
    const w = mount(MasterTable, { props: { rows } })
    const boxes = w.findAll('tbody input[type="checkbox"]')
    await boxes[0].setValue(true)
    expect(w.text()).toContain('1 selected')
    await w.find('thead input[type="checkbox"]').setValue(true)
    expect(w.text()).toContain('3 selected')
  })
})

describe('TxnTable (TanStack Table v9)', () => {
  const txns: Txn[] = [
    { id: 't1', date: '2026-02-03', amount: 15000, kind: 'IN', category: 'Sponsorship', sub: 'A' },
    { id: 't2', date: '2026-02-09', amount: 3200, kind: 'OUT', category: 'Gear', sub: 'Mic' },
  ]

  it('renders rows and sorts by amount (numbers go desc-first per v9 auto direction)', async () => {
    const w = mount(TxnTable, { props: { rows: txns, currency: '฿' } })
    expect(w.findAll('tbody tr')).toHaveLength(2)
    const amountTh = w.findAll('thead th')[1]
    await amountTh.trigger('click')
    expect(w.findAll('tbody tr')[0].text()).toContain('15,000')
    await amountTh.trigger('click')
    expect(w.findAll('tbody tr')[0].text()).toContain('3,200')
  })
})
