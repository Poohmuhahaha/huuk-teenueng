// @vitest-environment happy-dom
// Landing page: public marketing entry at /landing — hero, six hubs,
// pricing tiers and demo accounts, with CTAs into the app.
import { describe, expect, it } from 'vitest'
import { mount } from '@vue/test-utils'
import { createMemoryHistory, createRouter } from 'vue-router'
import LandingPage from '@/pages/LandingPage.vue'

function mountLanding() {
  const router = createRouter({
    history: createMemoryHistory(),
    routes: [
      { path: '/', component: { template: '<div />' } },
      { path: '/landing', component: LandingPage },
      { path: '/login', component: { template: '<div />' } },
      { path: '/register', component: { template: '<div />' } },
      { path: '/plans', component: { template: '<div />' } },
      { path: '/read', component: { template: '<div />' } },
      { path: '/dashboard', component: { template: '<div />' } },
      { path: '/plan', component: { template: '<div />' } },
      { path: '/content', component: { template: '<div />' } },
      { path: '/promote', component: { template: '<div />' } },
      { path: '/branding', component: { template: '<div />' } },
    ],
  })
  return mount(LandingPage, { global: { plugins: [router] } })
}

describe('landing page', () => {
  it('renders hero with CTAs', () => {
    const w = mountLanding()
    expect(w.find('h1').text()).toMatch(/Plan content/i)
    const buttons = w.findAll('button').map((b) => b.text())
    expect(buttons.join(' ')).toMatch(/Start free/)
    expect(buttons.join(' ')).toMatch(/Open workspace/)
    expect(buttons.join(' ')).toMatch(/See packages/)
    w.unmount()
  })

  it('lists all five hubs', () => {
    const w = mountLanding()
    const heads = w.findAll('h3').map((h) => h.text())
    for (const hub of ['Home', 'Plan', 'Content', 'Promote', 'Settings']) {
      expect(heads).toContain(hub)
    }
    w.unmount()
  })

  it('shows pricing tiers and demo accounts', () => {
    const w = mountLanding()
    const text = w.text()
    for (const tier of ['Free', 'Pro', 'Business']) expect(text).toContain(tier)
    for (const demo of ['owner@studio.local', 'editor@studio.local', 'client@studio.local']) {
      expect(text).toContain(demo)
    }
    expect(text).toContain('demo1234')
    w.unmount()
  })

  it('links to login, plans and reader', () => {
    const w = mountLanding()
    const hrefs = w.findAll('a').map((a) => a.attributes('href'))
    expect(hrefs.some((h) => h?.includes('/login'))).toBe(true)
    expect(hrefs.some((h) => h?.includes('/plans'))).toBe(true)
    expect(hrefs.some((h) => h?.includes('/read'))).toBe(true)
    w.unmount()
  })
})
