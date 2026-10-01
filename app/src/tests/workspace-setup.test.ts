// @vitest-environment happy-dom
// First-run onboarding: a signed-in account with no workspace creates one.
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { flushPromises, mount } from '@vue/test-utils'
import { QueryClient, VueQueryPlugin } from '@tanstack/vue-query'
import { createMemoryHistory, createRouter } from 'vue-router'
import WorkspaceSetupPage from '@/pages/WorkspaceSetupPage.vue'
import { activeWorkspaceId, setActiveWorkspace } from '@/core/workspace'
import { workspaces as dbWorkspaces } from '@/mock/db'

function mountPage() {
  const qc = new QueryClient({ defaultOptions: { queries: { retry: false } } })
  const router = createRouter({
    history: createMemoryHistory(),
    routes: [
      { path: '/', component: { template: '<div />' } },
      { path: '/welcome', component: WorkspaceSetupPage },
      { path: '/:pathMatch(.*)*', component: { template: '<div />' } },
    ],
  })
  const w = mount(WorkspaceSetupPage, {
    global: { plugins: [router, [VueQueryPlugin, { queryClient: qc }]] },
  })
  return { w, router }
}

beforeEach(() => {
  dbWorkspaces.splice(1)
  dbWorkspaces[0].name = 'workspace'
  setActiveWorkspace(null)
})

describe('WorkspaceSetupPage', () => {
  it('creates the first workspace and activates it', async () => {
    const { w, router } = mountPage()

    const button = w.find('button[type="submit"]')
    expect(button.attributes('disabled')).toBeDefined()

    await w.find('#ws-onboarding-name').setValue('Teenueng')
    expect(w.find('button[type="submit"]').attributes('disabled')).toBeUndefined()
    await w.find('form').trigger('submit')
    await flushPromises()

    await vi.waitFor(() => {
      expect(dbWorkspaces).toHaveLength(2)
      expect(dbWorkspaces[1].name).toBe('Teenueng')
    }, { timeout: 5000 })
    expect(activeWorkspaceId.value).toBe(dbWorkspaces[1].id)
    await vi.waitFor(() => {
      expect(router.currentRoute.value.path).toBe('/')
    }, { timeout: 5000 })

    w.unmount()
  })

  it('surfaces API errors without creating anything', async () => {
    const { w } = mountPage()
    await w.find('#ws-onboarding-name').setValue('   ')
    await w.find('form').trigger('submit')
    await flushPromises()
    expect(dbWorkspaces).toHaveLength(1)
    expect(w.find('.autherr').exists()).toBe(false)

    await w.find('#ws-onboarding-name').setValue('x'.repeat(81))
    await w.find('form').trigger('submit')
    await flushPromises()
    await vi.waitFor(() => {
      expect(w.find('.autherr').text()).toContain('too long')
    }, { timeout: 5000 })
    expect(dbWorkspaces).toHaveLength(1)

    w.unmount()
  })
})
