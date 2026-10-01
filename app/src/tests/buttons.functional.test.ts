// @vitest-environment happy-dom
// Regression tests for issues found by the button audit:
// - MasterTable row clicks were swallowed by `@click.stop` on every cell
// - ContentEditor toolbar had no coverage at all
import { describe, expect, it, vi } from 'vitest'
import { flushPromises, mount } from '@vue/test-utils'
import { QueryClient, VueQueryPlugin } from '@tanstack/vue-query'
import { createMemoryHistory, createRouter } from 'vue-router'
import MasterTable from '@/components/tables/MasterTable.vue'
import ContentEditor from '@/components/editor/ContentEditor.vue'
import { posts } from '@/mock/db'

describe('MasterTable row activation', () => {
  it('emits select when a data cell is clicked', async () => {
    const w = mount(MasterTable, { props: { rows: posts.slice(0, 3) } })
    await flushPromises()
    const cells = w.findAll('tbody td')
    expect(cells.length).toBeGreaterThan(1)
    await cells[1].trigger('click')
    expect(w.emitted('select')).toHaveLength(1)
  })

  it('does not emit select when the checkbox cell is clicked', async () => {
    const w = mount(MasterTable, { props: { rows: posts.slice(0, 3) } })
    await flushPromises()
    const checkbox = w.find('tbody input[type="checkbox"]')
    await checkbox.trigger('click')
    expect(w.emitted('select')).toBeUndefined()
  })

  it('emits select on Enter from a focused row', async () => {
    const w = mount(MasterTable, { props: { rows: posts.slice(0, 3) } })
    await flushPromises()
    const row = w.find('tbody tr')
    expect(row.attributes('tabindex')).toBe('0')
    await row.trigger('keydown', { key: 'Enter' })
    expect(w.emitted('select')).toHaveLength(1)
  })
})

describe('ContentEditor toolbar', () => {
  function mountEditor() {
    const router = createRouter({
      history: createMemoryHistory(),
      routes: [
        { path: '/', component: { template: '<div />' } },
        { path: '/read/:slug', component: { template: '<div />' } },
      ],
    })
    const qc = new QueryClient({ defaultOptions: { queries: { retry: false } } })
    return mount(ContentEditor, {
      props: { id: 'c-bts' },
      global: { plugins: [router, [VueQueryPlugin, { queryClient: qc }]] },
    })
  }

  it('wraps the selection with the bold button', async () => {
    const w = mountEditor()
    await vi.waitFor(() => expect(w.find('#studio-body').exists()).toBe(true), { timeout: 5000 })
    const bold = w.findAll('.toolbtn')[0]
    await vi.waitFor(() => expect((bold.element as HTMLButtonElement).disabled).toBe(false), { timeout: 5000 })
    const ta = w.find('#studio-body').element as HTMLTextAreaElement
    ta.value = 'hello world'
    ta.dispatchEvent(new Event('input'))
    await flushPromises()
    ta.setSelectionRange(0, 5)
    await bold.trigger('click')
    await flushPromises()
    expect(ta.value).toBe('**hello** world')
  })

  it('prefixes the current line with the H2 button', async () => {
    const w = mountEditor()
    await vi.waitFor(() => expect(w.find('#studio-body').exists()).toBe(true), { timeout: 5000 })
    const h2 = w.findAll('.toolbtn')[2]
    await vi.waitFor(() => expect((h2.element as HTMLButtonElement).disabled).toBe(false), { timeout: 5000 })
    const ta = w.find('#studio-body').element as HTMLTextAreaElement
    ta.value = 'first\nsecond'
    ta.dispatchEvent(new Event('input'))
    await flushPromises()
    ta.setSelectionRange(6, 6)
    await h2.trigger('click')
    await flushPromises()
    expect(ta.value).toBe('first\n## second')
  })
})
