// Deck card registry: Live, Campaigns, Ads, Members and Content (Studio) are
// cards in the slide deck, not standalone pages.
// @vitest-environment happy-dom
import { describe, expect, it, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import { QueryClient, VueQueryPlugin } from '@tanstack/vue-query'
import { createMemoryHistory, createRouter } from 'vue-router'
import { screens, screenIndexOf } from '@/core/screens'
import appRouter from '@/app/router'
import AppShell from '@/components/layout/AppShell.vue'
import LivePage from '@/pages/LivePage.vue'
import CampaignsPage from '@/pages/CampaignsPage.vue'
import AdsPage from '@/pages/AdsPage.vue'
import MembersPage from '@/pages/MembersPage.vue'
import StudioPage from '@/pages/StudioPage.vue'
import { login, logout } from '@/core/auth'

describe('deck card registry', () => {
  it('lists Live, Campaigns, Ads, Members and Content as cards', () => {
    const byTo = new Map(screens.map((s) => [s.to, s]))
    expect(byTo.get('/live')?.component).toBe(LivePage)
    expect(byTo.get('/campaigns')?.component).toBe(CampaignsPage)
    expect(byTo.get('/ads')?.component).toBe(AdsPage)
    expect(byTo.get('/members')?.component).toBe(MembersPage)
    expect(byTo.get('/studio')?.component).toBe(StudioPage)
  })

  it('resolves detail URLs to their cards', () => {
    const campaigns = screens.findIndex((s) => s.to === '/campaigns')
    const studio = screens.findIndex((s) => s.to === '/studio')
    expect(screenIndexOf('/campaigns')).toBe(campaigns)
    expect(screenIndexOf('/campaigns/cmp-1')).toBe(campaigns)
    expect(screenIndexOf('/studio/abc')).toBe(studio)
    expect(screenIndexOf('/live')).toBe(screens.findIndex((s) => s.to === '/live'))
  })

  it('keeps Brand first so / lands on it', () => {
    expect(screens[0].to).toBe('/brand')
  })

  it('routes the former standalone pages through the deck', () => {
    for (const [path, page] of [
      ['/live', LivePage],
      ['/campaigns', CampaignsPage],
      ['/campaigns/cmp-1', CampaignsPage],
      ['/ads', AdsPage],
      ['/members', MembersPage],
      ['/studio', StudioPage],
      ['/studio/abc', StudioPage],
    ] as const) {
      const route = appRouter.resolve(path)
      expect(route.meta.standalone).toBeFalsy()
      expect(route.matched[0]?.components?.default).toBe(page)
    }
  })
})

describe('navbar groups', () => {
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

  it('groups links into four dropdowns and closes on navigation', async () => {
    const w = mountShell()
    try {
      const labels = w.findAll('.navdrop').map((d) => d.text())
      expect(labels).toHaveLength(4)
      for (const name of ['Plan', 'Create', 'Promote', 'Analyze']) {
        expect(labels.some((t) => t.includes(name))).toBe(true)
      }

      await w.findAll('.navdrop')[2].trigger('click')
      await vi.waitFor(() => expect(w.find('.navmenu').exists()).toBe(true), { timeout: 5000 })
      expect(w.find('.navmenu').findAll('.navmenu-link').map((l) => l.text())).toEqual(
        ['Live', 'Campaigns', 'Ads'],
      )

      // Opening another group switches the menu.
      await w.findAll('.navdrop')[0].trigger('click')
      expect(w.find('.navmenu').findAll('.navmenu-link').map((l) => l.text())).toEqual(
        ['Brand', 'Monthly', 'Calendar', 'Ideas', 'Hashtags'],
      )

      // Navigating closes the menu via the route watcher.
      await w.findAll('.navmenu-link')[0].trigger('click')
      await vi.waitFor(() => expect(w.find('.navmenu').exists()).toBe(false), { timeout: 5000 })
    } finally {
      w.unmount()
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
