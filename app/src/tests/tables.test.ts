// @vitest-environment happy-dom
import { describe, expect, it } from 'vitest'
import { mount } from '@vue/test-utils'
import MasterTable from '@/components/tables/MasterTable.vue'
import TxnTable from '@/components/tables/TxnTable.vue'
import type { Post, Txn } from '@/mock/db'

function post(over: Partial<Post> & { id: string; topic: string }): Post {
  return {
    month: 2, year: 2026, pillar: 'Pillar I', format: 'Short-video', goal: '10K subscribe',
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

  it('limits rows according to pageSize and supports pagination', async () => {
    const manyPosts = [
      post({ id: 'p1', topic: 'P1' }),
      post({ id: 'p2', topic: 'P2' }),
      post({ id: 'p3', topic: 'P3' }),
      post({ id: 'p4', topic: 'P4' }),
    ]
    const w = mount(MasterTable, { props: { rows: manyPosts, pageSize: 2 } })
    expect(w.findAll('tbody tr')).toHaveLength(2)
    expect(w.text()).toContain('Showing 1–2 of 4')
    const nextBtn = w.find('button[aria-label="Next page"]')
    expect(nextBtn.exists()).toBe(true)
    await nextBtn.trigger('click')
    expect(w.findAll('tbody tr')).toHaveLength(2)
    expect(w.text()).toContain('Showing 3–4 of 4')
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

  it('limits rows according to pageSize and paginates', async () => {
    const manyTxns: Txn[] = [
      { id: 'm1', date: '2026-02-01', amount: 100, kind: 'IN', category: 'A', sub: '' },
      { id: 'm2', date: '2026-02-02', amount: 200, kind: 'IN', category: 'B', sub: '' },
      { id: 'm3', date: '2026-02-03', amount: 300, kind: 'OUT', category: 'C', sub: '' },
    ]
    const w = mount(TxnTable, { props: { rows: manyTxns, currency: '฿', pageSize: 2 } })
    // On desktop, TableFrame tbody tr should have 2 rows
    expect(w.find('.hidden.md\\:block tbody').findAll('tr')).toHaveLength(2)
    expect(w.text()).toContain('Showing 1–2 of 3')
    const nextBtn = w.find('button[aria-label="Next page"]')
    await nextBtn.trigger('click')
    expect(w.find('.hidden.md\\:block tbody').findAll('tr')).toHaveLength(1)
    expect(w.text()).toContain('Showing 3–3 of 3')
  })
})
