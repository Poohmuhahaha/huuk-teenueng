// @vitest-environment happy-dom
// The navbar workspace switcher: list, switch, rename, create.
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { flushPromises, mount } from '@vue/test-utils'
import { QueryClient, VueQueryPlugin } from '@tanstack/vue-query'
import { createMemoryHistory, createRouter } from 'vue-router'
import AppShell from '@/components/layout/AppShell.vue'
import { activeWorkspaceId, setActiveWorkspace } from '@/core/workspace'
import { workspaces as dbWorkspaces } from '@/mock/db'

function mountShell() {
  const qc = new QueryClient({ defaultOptions: { queries: { retry: false } } })
  const router = createRouter({
    history: createMemoryHistory(),
    routes: [
      { path: '/', component: { template: '<div />' } },
      { path: '/:pathMatch(.*)*', component: { template: '<div />' } },
    ],
  })
  return mount(AppShell, {
    global: { plugins: [router, [VueQueryPlugin, { queryClient: qc }]] },
    slots: { default: '<div />' },
  })
}

async function openMenu(w: ReturnType<typeof mountShell>) {
  // The switcher lives in the avatar toggle, next to the connect section.
  await w.find('.avatar').trigger('click')
  await vi.waitFor(() => {
    expect(w.find('.ws-trigger').exists()).toBe(true)
    expect(w.find('.ws-trigger').text()).toContain('workspace')
  }, { timeout: 5000 })
  await w.find('.ws-trigger').trigger('click')
  expect(w.find('.ws-menu').exists()).toBe(true)
  // The management actions appear once setup (permissions) has loaded.
  await vi.waitFor(() => {
    expect(w.find('.ws-form').exists()).toBe(true)
  }, { timeout: 5000 })
}

beforeEach(() => {
  // Reset the shared mock store between tests.
  dbWorkspaces.splice(1)
  dbWorkspaces[0].name = 'workspace'
  setActiveWorkspace(null)
})

describe('workspace switcher', () => {
  it('lists workspaces and marks the active one', async () => {
    const w = mountShell()
    await openMenu(w)

    const items = w.findAll('.ws-item')
    expect(items).toHaveLength(1)
    expect(items[0].text()).toContain('workspace')
    expect(items[0].classes()).toContain('active')

    w.unmount()
  })

  it('renames the active workspace from the menu', async () => {
    const w = mountShell()
    await openMenu(w)

    await w.findAll('.ws-actions .menubtn')[0].trigger('click')
    const input = w.find('.ws-actions input')
    expect(input.exists()).toBe(true)
    await input.setValue('Acme Studio')
    await input.trigger('keydown', { key: 'Enter' })
    await flushPromises()

    await vi.waitFor(() => {
      expect(dbWorkspaces[0].name).toBe('Acme Studio')
      expect(w.find('.ws-trigger').text()).toContain('Acme Studio')
    }, { timeout: 5000 })

    w.unmount()
  })

  it('creates a workspace and makes it active', async () => {
    const w = mountShell()
    await openMenu(w)

    await w.find('.ws-form .field').setValue('Client B')
    await w.find('.ws-form').trigger('submit')
    await flushPromises()

    await vi.waitFor(() => {
      expect(dbWorkspaces).toHaveLength(2)
      expect(dbWorkspaces[1].name).toBe('Client B')
      expect(w.find('.ws-trigger').text()).toContain('Client B')
    }, { timeout: 5000 })
    expect(activeWorkspaceId.value).toBe(dbWorkspaces[1].id)

    w.unmount()
  })

  it('switches back to another workspace', async () => {
    const w = mountShell()
    await openMenu(w)

    await w.find('.ws-form .field').setValue('Client B')
    await w.find('.ws-form').trigger('submit')
    await flushPromises()
    await vi.waitFor(() => {
      expect(w.find('.ws-trigger').text()).toContain('Client B')
    }, { timeout: 5000 })

    await w.find('.ws-trigger').trigger('click')
    const first = w.findAll('.ws-item')[0]
    expect(first.text()).toContain('workspace')
    await first.trigger('click')
    await flushPromises()

    expect(activeWorkspaceId.value).toBe('ws-default')
    await vi.waitFor(() => {
      expect(w.find('.ws-trigger').text()).toContain('workspace')
    }, { timeout: 5000 })

    w.unmount()
  })
})
