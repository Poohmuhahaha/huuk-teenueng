// @vitest-environment happy-dom
// Monthly plan set-up: adding a topic, editing the core fields, duplicating
// and deleting rows (mock API).
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { flushPromises, mount } from '@vue/test-utils'
import { QueryClient, VueQueryPlugin } from '@tanstack/vue-query'
import PlannerPage from '@/pages/PlannerPage.vue'
import router from '@/app/router'
import { posts } from '@/mock/db'

const settle = (ms = 350): Promise<void> => new Promise((r) => setTimeout(r, ms))

function mountPlanner() {
  const qc = new QueryClient({ defaultOptions: { queries: { retry: false } } })
  return mount(PlannerPage, {
    props: { month: '7' },
    global: { plugins: [router, [VueQueryPlugin, { queryClient: qc }]] },
  })
}

const julyRows = (): typeof posts => posts.filter((p) => p.month === 7)

/** Waits until permissions/setup loaded, so the plan actions are enabled. */
async function waitForAddButton(w: ReturnType<typeof mountPlanner>) {
  const button = w.find('.planner-empty .btn-primary')
  await vi.waitFor(() => {
    expect(button.attributes('disabled')).toBeUndefined()
  }, { timeout: 8000 })
  return button
}

describe('monthly plan set-up', () => {
  beforeEach(() => {
    for (let i = posts.length - 1; i >= 0; i -= 1) {
      if (posts[i].month === 7) posts.splice(i, 1)
    }
  })

  it('starts empty and adds a topic with sensible defaults', async () => {
    const w = mountPlanner()
    await settle()
    await flushPromises()

    expect(w.text()).toContain('No topics planned for this month yet')
    const add = await waitForAddButton(w)
    await add.trigger('click')
    await settle()
    await flushPromises()

    const rows = julyRows()
    expect(rows).toHaveLength(1)
    expect(rows[0].month).toBe(7)
    expect(rows[0].topic).toBe('Untitled topic')
    expect(rows[0].date?.startsWith('2026-07-')).toBe(true)
    expect(rows[0].status).toBeTruthy()
    expect(rows[0].platforms.length).toBeGreaterThan(0)

    // The new row is selected, so its editor opens for editing.
    expect(w.find('#post-topic').exists()).toBe(true)
    w.unmount()
  })

  it('edits the core plan fields and saves', async () => {
    const w = mountPlanner()
    await settle()
    await flushPromises()
    const add = await waitForAddButton(w)
    await add.trigger('click')
    await settle()
    await flushPromises()

    await w.find('#post-topic').setValue('Songkran teaser')
    await w.find('#post-date').setValue('2026-07-05')
    const save = w.findAll('button').find((b) => b.text().includes('Save row'))!
    await save.trigger('click')
    await settle()
    await flushPromises()

    expect(julyRows()[0].topic).toBe('Songkran teaser')
    expect(julyRows()[0].date).toBe('2026-07-05')
    w.unmount()
  })

  it('duplicates and deletes rows', async () => {
    const w = mountPlanner()
    await settle()
    await flushPromises()
    const add = await waitForAddButton(w)
    await add.trigger('click')
    await settle()
    await flushPromises()

    const duplicate = w.findAll('button').find((b) => b.text() === 'Duplicate')!
    await duplicate.trigger('click')
    await settle()
    await flushPromises()
    expect(julyRows()).toHaveLength(2)
    expect(julyRows().some((p) => p.topic.includes('(copy)'))).toBe(true)

    // window.confirm is not implemented in happy-dom; stub it for the delete.
    const originalConfirm = window.confirm
    window.confirm = () => true
    const remove = w.findAll('button').find((b) => b.text() === 'Delete')!
    await remove.trigger('click')
    await settle()
    await flushPromises()
    window.confirm = originalConfirm

    expect(julyRows()).toHaveLength(1)
    w.unmount()
  })
})
