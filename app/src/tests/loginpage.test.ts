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
      { path: '/:pathMatch(.*)*', component: { template: '<div />' } },
    ],
  })
  return [router, [VueQueryPlugin, { queryClient: qc }]] as const
}

async function reset(): Promise<void> {
  await logout().catch(() => undefined)
  await api.saveSetup({ authRequired: false, allowRegistration: true }).catch(() => undefined)
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
})

describe('app login gate', () => {
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
