// Compact IA: the app is 6 hub screens (Home, Plan, Content, Promote, Analyze,
// Settings). Related features live as tabs inside a hub; legacy deep links
// redirect into the owning hub + tab.
// @vitest-environment happy-dom
import { describe, expect, it, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import { QueryClient, VueQueryPlugin } from '@tanstack/vue-query'
import { createMemoryHistory, createRouter } from 'vue-router'
import { screens, screenIndexOf } from '@/core/screens'
import appRouter from '@/app/router'
import AppShell from '@/components/layout/AppShell.vue'
import DashboardPage from '@/pages/DashboardPage.vue'
import PlanPage from '@/pages/PlanPage.vue'
import ContentPage from '@/pages/ContentPage.vue'
import PromotePage from '@/pages/PromotePage.vue'
import AnalyzePage from '@/pages/AnalyzePage.vue'
import SettingsPage from '@/pages/SettingsPage.vue'
import { login, logout } from '@/core/auth'

describe('hub registry', () => {
  it('lists the six hub screens', () => {
    const byTo = new Map(screens.map((s) => [s.to, s]))
    expect(byTo.get('/dashboard')?.component).toBe(DashboardPage)
    expect(byTo.get('/plan')?.component).toBe(PlanPage)
    expect(byTo.get('/content')?.component).toBe(ContentPage)
    expect(byTo.get('/promote')?.component).toBe(PromotePage)
    expect(byTo.get('/analyze')?.component).toBe(AnalyzePage)
    expect(byTo.get('/settings')?.component).toBe(SettingsPage)
    expect(screens).toHaveLength(6)
  })

  it('resolves each hub path to its card', () => {
    expect(screenIndexOf('/dashboard')).toBe(screens.findIndex((s) => s.to === '/dashboard'))
    expect(screenIndexOf('/plan')).toBe(screens.findIndex((s) => s.to === '/plan'))
    expect(screenIndexOf('/content')).toBe(screens.findIndex((s) => s.to === '/content'))
    expect(screenIndexOf('/promote')).toBe(screens.findIndex((s) => s.to === '/promote'))
    expect(screenIndexOf('/analyze')).toBe(screens.findIndex((s) => s.to === '/analyze'))
    expect(screenIndexOf('/settings')).toBe(screens.findIndex((s) => s.to === '/settings'))
  })

  it('lands / on Home', () => {
    expect(screens[0].to).toBe('/dashboard')
  })

  it('redirects legacy deep links into the owning hub + tab', async () => {
    for (const [path, target, tab] of [
      ['/brand', '/settings', 'brand'],
      ['/members', '/settings', 'members'],
      ['/calendar', '/plan', 'calendar'],
      ['/hashtags', '/plan', 'hashtags'],
      ['/feed', '/content', 'feed'],
      ['/studio', '/content', 'studio'],
      ['/live', '/dashboard', undefined],
      ['/campaigns', '/promote', 'campaigns'],
      ['/ads', '/promote', 'ads'],
      ['/performance', '/analyze', 'performance'],
      ['/finance', '/analyze', 'finance'],
    ] as const) {
      await appRouter.push(path)
      expect(appRouter.currentRoute.value.path).toBe(target)
      if (tab) expect(appRouter.currentRoute.value.query.tab).toBe(tab)
    }
    await appRouter.push('/studio/abc')
    expect(appRouter.currentRoute.value.path).toBe('/content')
    expect(appRouter.currentRoute.value.query).toMatchObject({ tab: 'studio', id: 'abc' })
    await appRouter.push('/campaigns/cmp-1')
    expect(appRouter.currentRoute.value.path).toBe('/promote')
    expect(appRouter.currentRoute.value.query).toMatchObject({ tab: 'campaigns', id: 'cmp-1' })
  })
})

describe('compact navbar', () => {
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

  it('renders one flat link per hub', async () => {
    await login('owner@studio.local', 'demo1234')
    const w = mountShell()
    try {
      const labels = w.findAll('.navlink').map((l) => l.text())
      for (const name of ['Home', 'Plan', 'Content', 'Promote', 'Analyze', 'Settings']) {
        expect(labels).toContain(name)
      }
      expect(w.findAll('.navdrop')).toHaveLength(0)
    } finally {
      w.unmount()
      await logout().catch(() => undefined)
    }
  })
})

describe('avatar toggle (connect + workspace)', () => {
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

  it('shows the connect section and the workspace switcher inside the toggle', async () => {
    await login('owner@studio.local', 'demo1234')
    const w = mountShell()
    try {
      await w.find('.avatar').trigger('click')
      await vi.waitFor(() => {
        expect(w.find('.avatar-menu').exists()).toBe(true)
        expect(w.find('.pop-connect').exists()).toBe(true)
      }, { timeout: 8000 })
      // One row per platform, each opening the connection dialog…
      expect(w.findAll('.conn-row')).toHaveLength(3)
      // …and the workspace switcher right below it.
      expect(w.find('.avatar-menu .ws-trigger').exists()).toBe(true)

      await w.findAll('.conn-btn')[0].trigger('click')
      await vi.waitFor(() => {
        expect(w.find('.plogin').exists()).toBe(true)
      }, { timeout: 8000 })
    } finally {
      w.unmount()
      await logout().catch(() => undefined)
    }
  })
})
