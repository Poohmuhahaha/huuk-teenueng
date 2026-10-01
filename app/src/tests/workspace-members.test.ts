// @vitest-environment happy-dom
// Owner-only Workspace members page: form gating, add/remove, and mock errors.
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import { QueryClient, VueQueryPlugin } from '@tanstack/vue-query'
import { createMemoryHistory, createRouter } from 'vue-router'
import MembersPage from '@/pages/MembersPage.vue'
import AppShell from '@/components/layout/AppShell.vue'
import appRouter from '@/app/router'
import * as api from '@/mock/api'
import { setup, workspaces as dbWorkspaces } from '@/mock/db'
import { setActiveWorkspace } from '@/core/workspace'
import { clearSession } from '@/core/session'

const SEED_MEMBERS = ['editor@studio.local', 'client@studio.local']

function mountPage() {
  const qc = new QueryClient({ defaultOptions: { queries: { retry: false } } })
  return mount(MembersPage, {
    global: { plugins: [[VueQueryPlugin, { queryClient: qc }]] },
  })
}

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

function navLabels(w: ReturnType<typeof mountShell>): string[] {
  return w.findAll('.navlink').map((l) => l.text())
}

beforeEach(async () => {
  await api.logoutAll().catch(() => undefined)
  clearSession()
  setup.authRequired = false
  dbWorkspaces.splice(1)
  dbWorkspaces[0].name = 'workspace'
  dbWorkspaces[0].owner = 'thontrapoowadol@example.com'
  dbWorkspaces[0].members = [...SEED_MEMBERS]
  setActiveWorkspace(null)
  vi.stubGlobal('confirm', () => true)
})

afterEach(() => {
  vi.unstubAllGlobals()
})

describe('workspace members page', () => {
  it('lets the owner see, add, and remove members', async () => {
    const w = mountPage()
    await vi.waitFor(() => {
      expect(w.find('.members-add').exists()).toBe(true)
      expect(w.findAll('.member-row')).toHaveLength(3)
    }, { timeout: 8000 })
    expect(w.text()).toContain('thontrapoowadol@example.com')
    expect(w.text()).toContain('editor@studio.local')
    expect(w.findAll('.member-row')[0].text()).toContain('Owner')

    await w.find('.members-add input').setValue('owner@studio.local')
    await w.find('.members-add').trigger('submit')
    await vi.waitFor(async () => {
      const view = await api.listWorkspaceMembers('ws-default')
      expect(view.members.map((m) => m.email)).toContain('owner@studio.local')
    }, { timeout: 8000 })
    await vi.waitFor(() => {
      expect(w.find('.oknote').text()).toContain('Member added')
    }, { timeout: 8000 })

    const editorRow = w.findAll('.member-row').find((row) => row.text().includes('editor@studio.local'))
    expect(editorRow).toBeTruthy()
    await editorRow!.find('.member-remove').trigger('click')
    await vi.waitFor(async () => {
      const view = await api.listWorkspaceMembers('ws-default')
      expect(view.members.map((m) => m.email)).not.toContain('editor@studio.local')
    }, { timeout: 8000 })
    w.unmount()

    const shell = mountShell()
    await vi.waitFor(() => {
      expect(navLabels(shell)).toContain('Members')
    }, { timeout: 8000 })
    shell.unmount()
  })

  it('shows only the owner notice to a non-owner and hides the nav link', async () => {
    await api.login('editor@studio.local', 'demo1234')
    setActiveWorkspace('ws-default')

    const w = mountPage()
    await vi.waitFor(() => {
      expect(w.text()).toContain('Only the workspace owner can manage members')
    }, { timeout: 8000 })
    expect(w.find('.members-add').exists()).toBe(false)
    expect(w.find('.member-row').exists()).toBe(false)
    w.unmount()

    const shell = mountShell()
    await shell.find('.avatar').trigger('click')
    await vi.waitFor(() => {
      expect(shell.find('.ws-trigger').exists()).toBe(true)
    }, { timeout: 8000 })
    expect(navLabels(shell)).not.toContain('Members')
    shell.unmount()
  })

  it('surfaces mock errors in an alert', async () => {
    const w = mountPage()
    await vi.waitFor(() => {
      expect(w.find('.members-add').exists()).toBe(true)
    }, { timeout: 8000 })

    await w.find('.members-add input').setValue('client@studio.local')
    await w.find('.members-add').trigger('submit')
    await vi.waitFor(() => {
      const alert = w.find('.autherr')
      expect(alert.exists()).toBe(true)
      expect(alert.text()).toContain('already owns or belongs')
    }, { timeout: 8000 })

    await w.find('.members-add input').setValue('not-an-email')
    await w.find('.members-add').trigger('submit')
    await vi.waitFor(() => {
      expect(w.find('.autherr').text()).toContain('valid email')
    }, { timeout: 8000 })
    w.unmount()
  })
})

describe('members route registration', () => {
  it('registers /members as a deck card (App renders the deck, not a standalone view)', () => {
    const route = appRouter.resolve('/members')
    expect(route.meta.standalone).toBeFalsy()
    expect(route.matched[0]?.components?.default).toBe(MembersPage)
  })
})
