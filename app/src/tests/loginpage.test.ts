// @vitest-environment happy-dom
// Production gate: with AUTH_REQUIRED the app must show a login page first.
import { afterEach, describe, expect, it, vi } from 'vitest'
import { flushPromises, mount } from '@vue/test-utils'
import { QueryClient, VueQueryPlugin } from '@tanstack/vue-query'
import { createMemoryHistory, createRouter } from 'vue-router'
import App from '@/app/App.vue'
import LoginPage from '@/pages/LoginPage.vue'
import * as api from '@/mock/api'
import { currentUser, logout } from '@/core/auth'

function makePlugins() {
  const qc = new QueryClient({ defaultOptions: { queries: { retry: false } } })
  const router = createRouter({
    history: createMemoryHistory(),
    routes: [
      { path: '/', component: { template: '<div />' } },
      { path: '/branding', component: { template: '<div />' } },
      { path: '/login', component: { template: '<div />' } },
      { path: '/:pathMatch(.*)*', component: { template: '<div />' } },
    ],
  })
  return [router, [VueQueryPlugin, { queryClient: qc }]] as const
}

async function reset(): Promise<void> {
  sessionStorage.clear()
  await api.login('owner@studio.local', 'demo1234').catch(() => undefined)
  await api.saveSetup({ authRequired: false, allowRegistration: true }).catch(() => undefined)
  await logout().catch(() => undefined)
}

afterEach(reset)

describe('login page', () => {
  it('reports bad credentials and signs in with valid ones', async () => {
    const w = mount(LoginPage, { global: { plugins: makePlugins() as never } })
    await flushPromises()

    await w.find('#lp-email').setValue('owner@studio.local')
    await w.find('#lp-password').setValue('wrong-password')
    await w.find('form').trigger('submit')
    await vi.waitFor(() => expect(w.find('.autherr').exists()).toBe(true), { timeout: 5000 })
    expect(w.find('.autherr').text()).toMatch(/invalid/i)
    expect(currentUser.value).toBeNull()

    await w.find('#lp-email').setValue('owner@studio.local')
    await w.find('#lp-password').setValue('demo1234')
    await w.find('form').trigger('submit')
    await vi.waitFor(() => expect(currentUser.value?.name).toBe('Studio Owner'), { timeout: 5000 })
    w.unmount()
  })

  it('hides the registration toggle when the server disallows it', async () => {
    await api.saveSetup({ allowRegistration: false })
    const w = mount(LoginPage, { global: { plugins: makePlugins() as never } })
    await vi.waitFor(() => expect(w.find('.lp-switch').exists()).toBe(false), { timeout: 5000 })
    w.unmount()
  })

  it('ignores a redirect query and returns to the remembered app path', async () => {
    const [router, vueQuery] = makePlugins()
    await router.push('/login?redirect=//evil.example')
    await router.isReady()

    const w = mount(LoginPage, { global: { plugins: [router, vueQuery] as never } })
    await flushPromises()

    await w.find('#lp-email').setValue('owner@studio.local')
    await w.find('#lp-password').setValue('demo1234')
    await w.find('form').trigger('submit')
    await vi.waitFor(() => expect(router.currentRoute.value.path).toBe('/'), { timeout: 5000 })

    w.unmount()
  })
})

describe('app login gate', () => {
  it('remembers the real path and returns there after login', async () => {
    await api.saveSetup({ authRequired: true })
    await logout()

    const [router, vueQuery] = makePlugins()
    await router.push('/branding')
    await router.isReady()

    const w = mount(App, { global: { plugins: [router, vueQuery] as never } })
    await vi.waitFor(() => expect(router.currentRoute.value.path).toBe('/login'), { timeout: 5000 })
    expect(router.currentRoute.value.query).toEqual({})

    await w.find('#lp-email').setValue('owner@studio.local')
    await w.find('#lp-password').setValue('demo1234')
    await w.find('form').trigger('submit')
    await vi.waitFor(() => expect(router.currentRoute.value.path).toBe('/branding'), { timeout: 5000 })

    w.unmount()
  })

  it('shows the login page before the shell when auth is required', async () => {
    await api.saveSetup({ authRequired: true })
    await logout()

    const w = mount(App, { global: { plugins: makePlugins() as never } })
    await vi.waitFor(() => expect(w.find('.loginpage').exists()).toBe(true), { timeout: 5000 })
    expect(w.find('.app').exists()).toBe(false)

    await w.find('#lp-email').setValue('owner@studio.local')
    await w.find('#lp-password').setValue('demo1234')
    await w.find('form').trigger('submit')
    await flushPromises()
    await vi.waitFor(() => expect(w.find('.app').exists()).toBe(true), { timeout: 5000 })

    w.unmount()
  })
})
