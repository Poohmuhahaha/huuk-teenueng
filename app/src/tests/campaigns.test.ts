// @vitest-environment happy-dom
// Campaign manager: setup, schedule window, linked content and permissions.
import { afterEach, describe, expect, it, vi } from 'vitest'
import { flushPromises, mount } from '@vue/test-utils'
import { QueryClient, VueQueryPlugin } from '@tanstack/vue-query'
import CampaignsPage from '@/pages/CampaignsPage.vue'
import router from '@/app/router'
import * as api from '@/mock/api'
import { setup } from '@/mock/db'
import { login, logout } from '@/core/auth'

async function reset(): Promise<void> {
  // The permission test turns auth on; sign back in as owner to switch it off.
  if (setup.authRequired) await login('owner@studio.local', 'demo1234').catch(() => undefined)
  await api.saveSetup({ authRequired: false }).catch(() => undefined)
  await logout().catch(() => undefined)
  for (const c of await api.listCampaigns().catch(() => [])) {
    if (c.id.startsWith('cmp-') && !['cmp-spring', 'cmp-evergreen'].includes(c.id)) {
      await api.deleteCampaign(c.id).catch(() => undefined)
    }
  }
}

afterEach(reset)

describe('campaigns (mock parity)', () => {
  it('seeds campaigns with a schedule window and linked content', async () => {
    const rows = await api.listCampaigns()
    expect(rows.length).toBeGreaterThanOrEqual(2)
    const spring = rows.find((c) => c.id === 'cmp-spring')!
    expect(spring.status).toBe('active')
    expect(spring.startDate).toBe('2026-03-01')
    expect(spring.endDate).toBe('2026-03-31')
    expect(spring.contentIds).toContain('c-march')
  })

  it('creates, schedules, updates and deletes a campaign', async () => {
    const created = await api.createCampaign({
      name: 'Winter Test', objective: 'Test the flow', status: 'draft',
      startDate: '2026-12-01', endDate: '2026-12-31',
      platforms: ['Instagram', 'Instagram', ''],
      budget: 1000, goalMetric: 'reach', goalTarget: 50000,
      contentIds: ['c-welcome'],
    })
    expect(created.platforms).toEqual(['Instagram'])
    expect(created.owner).toBe('Studio Owner')

    const updated = await api.updateCampaign(created.id, { status: 'active', endDate: '2027-01-15' })
    expect(updated.status).toBe('active')
    expect(updated.endDate).toBe('2027-01-15')

    await expect(api.updateCampaign(created.id, { endDate: '2026-11-01' }))
      .rejects.toThrow(/endDate/)
    await expect(api.updateCampaign(created.id, { status: 'archived' as never }))
      .rejects.toThrow(/status/)
    await expect(api.updateCampaign(created.id, { contentIds: ['c-nope'] }))
      .rejects.toThrow(/unknown content/)

    await api.deleteCampaign(created.id)
    await expect(api.getCampaign(created.id)).rejects.toThrow(/not found/)
  })

  it('requires the campaign permissions when auth is on', async () => {
    await api.saveSetup({ authRequired: true })
    await logout()
    await expect(api.createCampaign({ name: 'Anon' })).rejects.toThrow()

    // Viewer can read but not write.
    await login('owner@studio.local', 'demo1234')
    await api.addUser('Viewer Campaign', 'Viewer').catch(() => undefined)
    await logout()
    await api.register({ name: 'Viewer Campaign', email: 'viewer-camp@test.local', password: 'longpassword12' })
    await expect(api.createCampaign({ name: 'Viewer Campaign' })).rejects.toThrow(/campaigns.write/)
    await logout()

    // Editor can write.
    await login('editor@studio.local', 'demo1234')
    const ok = await api.createCampaign({ name: 'Editor Campaign' })
    expect(ok.id).toMatch(/^cmp-/)
  })
})

describe('campaigns page', () => {
  function mountPage() {
    const qc = new QueryClient({ defaultOptions: { queries: { retry: false } } })
    return mount(CampaignsPage, {
      global: { plugins: [router, [VueQueryPlugin, { queryClient: qc }]] },
    })
  }

  it('lists campaigns and opens the editor', async () => {
    await router.push('/campaigns')
    await router.isReady()
    const w = mountPage()
    await vi.waitFor(() => expect(w.findAll('.camp-item').length).toBeGreaterThanOrEqual(2), { timeout: 5000 })
    await w.findAll('.camp-item')[0].trigger('click')
    await vi.waitFor(() => expect(w.find('#camp-name').exists()).toBe(true), { timeout: 5000 })
    expect((w.find('#camp-name').element as HTMLInputElement).value).toBe('Spring Launch')
    w.unmount()
  }, 20000)

  it('creates a campaign and links content from the page', async () => {
    await router.push('/campaigns')
    await router.isReady()
    const w = mountPage()
    // Wait until setup has loaded so the button is enabled (canWrite).
    await vi.waitFor(() => {
      const btn = w.find('button.btn-primary')
      expect(btn.exists()).toBe(true)
      expect(btn.attributes('disabled')).toBeUndefined()
    }, { timeout: 8000 })
    await w.find('button.btn-primary').trigger('click')
    await vi.waitFor(() => expect(w.find('#camp-name').exists()).toBe(true), { timeout: 5000 })

    const scheduled = await api.listContent()
    const linked = w.findAll('.linkitem input[type="checkbox"]')
    expect(linked.length).toBe(scheduled.length)
    await linked[0].setValue(true)
    await w.find('#camp-name').setValue('Linked Campaign')
    const save = w.findAll('button.btn-primary').find((b) => b.text() === 'Save')
    await save?.trigger('click')
    await new Promise((r) => setTimeout(r, 900))
    await flushPromises()

    const rows = await api.listCampaigns()
    const saved = rows.find((c) => c.name === 'Linked Campaign')
    expect(saved).toBeTruthy()
    expect(saved?.contentIds).toHaveLength(1)
    w.unmount()
  }, 20000)
})
