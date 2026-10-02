// @vitest-environment happy-dom
// SaaS onboarding: register -> plans -> continue with Free -> workspace.
import { afterEach, describe, expect, it, vi } from 'vitest'
import { flushPromises, mount } from '@vue/test-utils'
import { QueryClient, VueQueryPlugin } from '@tanstack/vue-query'
import router from '@/app/router'
import RegisterPage from '@/pages/RegisterPage.vue'
import PlansPage from '@/pages/PlansPage.vue'
import { currentUser, logout, register } from '@/core/auth'
import * as api from '@/mock/api'

function mountPage(component: unknown) {
  const qc = new QueryClient({ defaultOptions: { queries: { retry: false } } })
  return mount(component as never, {
    global: { plugins: [router, [VueQueryPlugin, { queryClient: qc }]] },
  })
}

afterEach(async () => {
  await logout().catch(() => undefined)
  await api.saveSetup({ allowRegistration: true }).catch(() => undefined)
})

describe('register page', () => {
  it('creates an account and continues to the plans page', async () => {
    await router.push('/register')
    await router.isReady()
    const w = mountPage(RegisterPage)
    await vi.waitFor(() => expect(w.find('#reg-name').exists()).toBe(true), { timeout: 5000 })

    await w.find('#reg-name').setValue('New Saas User')
    await w.find('#reg-email').setValue('saas@test.local')
    await w.find('#reg-password').setValue('longpassword12')
    await w.find('form').trigger('submit')

    await vi.waitFor(() => expect(currentUser.value?.name).toBe('New Saas User'), { timeout: 5000 })
    await vi.waitFor(() => expect(router.currentRoute.value.path).toBe('/plans'), { timeout: 5000 })
    expect(currentUser.value?.plan).toBe('free')
    w.unmount()
  })

  it('shows a closed notice when registration is disabled', async () => {
    await api.saveSetup({ allowRegistration: false })
    const w = mountPage(RegisterPage)
    await vi.waitFor(() => expect(w.text()).toContain('Registration is currently closed'), { timeout: 5000 })
    expect(w.find('form').exists()).toBe(false)
    w.unmount()
  })
})

describe('plans page', () => {
  it('continues with the free package into the workspace', async () => {
    await register({ name: 'Plan Picker', email: 'picker@test.local', password: 'longpassword12' })
    await router.push('/plans')
    await router.isReady()
    const w = mountPage(PlansPage)

    await w.find('.plan-card .btn-primary').trigger('click')
    // '/' redirects into the compact IA: the first-run setup lands on Settings › Brand.
    await vi.waitFor(() => expect(router.currentRoute.value.path).toBe('/settings'), { timeout: 5000 })
    expect(router.currentRoute.value.query.tab).toBe('brand')
    expect(router.currentRoute.value.query.onboarding).toBe('1')
    expect(currentUser.value?.plan).toBe('free')
    w.unmount()
  })

  it('sends signed-out visitors to the register page', async () => {
    await logout()
    await router.push('/plans')
    await router.isReady()
    const w = mountPage(PlansPage)

    await w.find('.plan-card .btn-primary').trigger('click')
    await flushPromises()
    await vi.waitFor(() => expect(router.currentRoute.value.path).toBe('/register'), { timeout: 5000 })
    w.unmount()
  })
})
