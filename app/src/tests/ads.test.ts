// @vitest-environment happy-dom
// Meta Ads page: mirror rendering (campaigns, settings, insights) plus the
// opt-in-gated management actions (pause, budget, duplicate, boost).
import { describe, expect, it, beforeEach } from 'vitest'
import { flushPromises, mount } from '@vue/test-utils'
import { QueryClient, VueQueryPlugin } from '@tanstack/vue-query'
import AdsPage from '@/pages/AdsPage.vue'
import { ads, adsFlags, adsFetchedAt } from '@/mock/db'

const settle = (ms = 400): Promise<void> => new Promise((r) => setTimeout(r, ms))

function mountAds() {
  const qc = new QueryClient({ defaultOptions: { queries: { retry: false } } })
  return mount(AdsPage, {
    global: { plugins: [[VueQueryPlugin, { queryClient: qc }]] },
  })
}

function resetSeed(): void {
  adsFlags.manage = false
  ads.fetchedAt = adsFetchedAt
  ads.error = ''
  ads.audit = []
  ads.campaigns = [
    {
      id: 'cmp-ads-1', accountId: 'act_102400000000001',
      name: 'Spring Collection — Traffic', objective: 'OUTCOME_TRAFFIC',
      status: 'ACTIVE', effectiveStatus: 'ACTIVE', buyingType: 'AUCTION',
      dailyBudget: 50000, lifetimeBudget: 0, budgetRemaining: 32000,
      bidStrategy: 'LOWEST_COST_WITHOUT_CAP', specialAdCategories: ['NONE'],
      startTime: '2026-02-20T02:00:00+0000', stopTime: '', createdTime: '2026-02-18T04:00:00+0000',
    },
    {
      id: 'cmp-ads-2', accountId: 'act_102400000000001',
      name: 'Reels Engagement — Paused', objective: 'OUTCOME_ENGAGEMENT',
      status: 'PAUSED', effectiveStatus: 'PAUSED', buyingType: 'AUCTION',
      dailyBudget: 0, lifetimeBudget: 300000, budgetRemaining: 120000,
      bidStrategy: 'LOWEST_COST_WITH_BID_CAP', specialAdCategories: ['NONE'],
      startTime: '2026-02-10T02:00:00+0000', stopTime: '', createdTime: '2026-02-08T04:00:00+0000',
    },
  ]
  ads.adsets = [
    {
      id: 'adset-1', campaignId: 'cmp-ads-1', name: 'Bangkok 25-45',
      status: 'ACTIVE', effectiveStatus: 'ACTIVE',
      dailyBudget: 0, lifetimeBudget: 0,
      optimizationGoal: 'LINK_CLICKS', billingEvent: 'IMPRESSIONS', bidAmount: 0,
      startTime: '2026-02-20T02:00:00+0000', endTime: '',
      targeting: '25-45 · TH · 3 interests', promotedObject: 'page 102400000000001',
    },
  ]
  ads.ads = [
    {
      id: 'ad-1', adsetId: 'adset-1', name: 'Spring photo — link',
      status: 'ACTIVE', effectiveStatus: 'ACTIVE',
      creativeTitle: 'Spring is here', creativeBody: 'Shop the new collection.',
      imageUrl: '', thumbnailUrl: '', storyId: '102400000000001_9001', previewUrl: '',
    },
  ]
}

describe('AdsPage', () => {
  beforeEach(resetSeed)

  it('renders campaigns, budgets, insights and settings', async () => {
    const w = mountAds()
    await settle()
    await flushPromises()

    expect(w.text()).toContain('Meta Ads')
    const rows = w.findAll('.ads-tbl tbody tr')
    expect(rows.length).toBeGreaterThanOrEqual(2)
    expect(w.text()).toContain('Spring Collection — Traffic')
    expect(w.text()).toContain('Daily 500.00 THB')
    expect(w.text()).toContain('Lifetime 3,000.00 THB')
    expect(w.text()).toContain('1,240')
    expect(w.text()).toContain('2.95%')
    // Summary strip adds up the mirror.
    expect(w.text()).toContain('2,500.00 THB')

    // Expanding a campaign shows settings, ad sets and ads.
    await w.findAll('.ads-link')[0].trigger('click')
    await flushPromises()
    expect(w.text()).toContain('LOWEST_COST_WITHOUT_CAP')
    expect(w.text()).toContain('Bangkok 25-45')
    expect(w.text()).toContain('25-45 · TH · 3 interests')
    expect(w.text()).toContain('Spring is here')
    expect(w.text()).toContain('Read-only')
  })

  it('gates management actions behind the opt-in', async () => {
    const w = mountAds()
    await settle()
    await flushPromises()

    const toggle = w.find('.ads-toggle')
    expect(toggle.attributes('disabled')).toBeDefined()
    expect(toggle.attributes('title')).toContain('Turn on campaign management')

    await w.find('.ads-manage-toggle').trigger('click')
    await flushPromises()
    expect(w.text()).toContain('Allow Huuk to change campaigns')
    await w.find('.ads-dialog .btn-primary').trigger('click')
    await settle(300)
    await flushPromises()
    expect(w.text()).toContain('Management on')
    expect(adsFlags.manage).toBe(true)

    // Now pause works and is audited.
    await w.find('.ads-toggle').trigger('click')
    await settle(500)
    await flushPromises()
    expect(ads.campaigns[0].status).toBe('PAUSED')
    expect(ads.audit[0].action).toBe('campaign paused')
    expect(w.text()).toContain('campaign paused')
  })

  it('edits a budget with validation', async () => {
    adsFlags.manage = true
    const w = mountAds()
    await settle()
    await flushPromises()

    await w.findAll('.ads-link')[0].trigger('click')
    await flushPromises()
    const edit = w.findAll('button').find((b) => b.text() === 'Edit budget')!
    await edit.trigger('click')
    await flushPromises()

    const input = w.find('.ads-dialog input[type="number"]')
    await input.setValue('0')
    await w.find('.ads-dialog .btn-primary').trigger('click')
    await flushPromises()
    expect(w.text()).toContain('greater than zero')

    await input.setValue('12.50')
    await w.find('.ads-dialog .btn-primary').trigger('click')
    await settle(500)
    await flushPromises()
    expect(ads.campaigns[0].dailyBudget).toBe(1250)
    expect(ads.audit[0].action).toBe('budget updated')
  })

  it('duplicates a campaign as paused', async () => {
    adsFlags.manage = true
    const w = mountAds()
    await settle()
    await flushPromises()

    await w.findAll('.ads-link')[0].trigger('click')
    await flushPromises()
    const dup = w.findAll('button').find((b) => b.text() === 'Duplicate')!
    await dup.trigger('click')
    await settle(700)
    await flushPromises()

    expect(ads.campaigns).toHaveLength(3)
    expect(ads.campaigns[0].name).toContain('(copy)')
    expect(ads.campaigns[0].status).toBe('PAUSED')
    expect(ads.audit[0].action).toBe('campaign duplicated')
  })

  it('creates a paused boost for a live post and validates input', async () => {
    adsFlags.manage = true
    const w = mountAds()
    await settle()
    await flushPromises()

    await w.find('.ads-boost').trigger('click')
    await flushPromises()
    // The boost helper now hides behind an (i) button in the dialog title.
    expect(w.text()).not.toContain('Creates a paused campaign')
    await w.find('.ads-dialog .infotip').trigger('click')
    expect(w.text()).toContain('Creates a paused campaign')

    // Missing name is refused before any API call.
    await w.find('.ads-dialog .btn-primary').trigger('click')
    await flushPromises()
    expect(w.text()).toContain('Give the boost a name')
    expect(ads.campaigns).toHaveLength(2)

    await w.find('.ads-dialog input[maxlength="120"]').setValue('Songkran boost')
    await w.find('.ads-dialog .btn-primary').trigger('click')
    await settle(1100)
    await flushPromises()

    expect(ads.campaigns).toHaveLength(3)
    expect(ads.campaigns[0].name).toBe('Songkran boost')
    expect(ads.campaigns[0].status).toBe('PAUSED')
    expect(ads.ads[0].storyId).toBe('102400000000001_9001')
    expect(ads.audit[0].action).toBe('boost created')
  })
})
