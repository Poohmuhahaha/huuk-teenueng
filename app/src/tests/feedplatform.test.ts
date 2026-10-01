// @vitest-environment happy-dom
// Feed Review: the platform tabs actually change the preview (counts, canvas
// ratio, tiles) and a setup refetch must not reset a platform the user picked.
import { describe, expect, it, vi } from 'vitest'
import { flushPromises, mount } from '@vue/test-utils'
import { QueryClient, VueQueryPlugin } from '@tanstack/vue-query'
import FeedPage from '@/pages/FeedPage.vue'
import router from '@/app/router'
import { qk } from '@/core/queries'
import { activePlatforms } from '@/core/platforms'

const settle = (ms = 400): Promise<void> => new Promise((r) => setTimeout(r, ms))

function mountFeed() {
  const qc = new QueryClient({ defaultOptions: { queries: { retry: false } } })
  return {
    qc,
    w: mount(FeedPage, { global: { plugins: [router, [VueQueryPlugin, { queryClient: qc }]] } }),
  }
}

function tab(w: ReturnType<typeof mountFeed>['w'], name: string) {
  return w.findAll('.feed-tab').find((b) => b.text().startsWith(name))!
}

describe('Feed Review platform switching', () => {
  it('shows every setup platform with its planned-post count', async () => {
    activePlatforms.value = ['meta']
    const { w } = mountFeed()
    await settle()
    await flushPromises()

    const tabs = w.findAll('.feed-tab').map((b) => b.text())
    expect(tabs.some((label) => label.startsWith('Facebook'))).toBe(true)
    expect(tabs.some((label) => label.startsWith('Instagram'))).toBe(true)
    // Counts come from the demo plan (Instagram is on most posts).
    expect(w.find('.feed-tab.active').text()).toContain('Facebook')
    w.unmount()
  })

  it('switches the canvas and the tiles when another platform is chosen', async () => {
    activePlatforms.value = ['meta']
    const { w } = mountFeed()
    await settle()
    await flushPromises()

    expect(w.text()).toContain('1200 × 630') // Facebook link preview
    const facebookTiles = w.findAll('.cell').length

    await tab(w, 'TikTok').trigger('click')
    await settle()
    await flushPromises()
    expect(w.text()).toContain('1080 × 1920')
    expect(w.findAll('.cell').length).not.toBe(facebookTiles)
    expect(w.find('.feed-tab.active').text()).toContain('TikTok')

    // Narrowing the date range empties the feed and the tab counts follow.
    await w.find('#feed-upto').setValue('2020-01-01')
    await settle()
    await flushPromises()
    expect(w.text()).toContain('Nothing scheduled in range.')
    expect(w.find('.feed-tab.active .feed-tab-count').text()).toBe('0')
    w.unmount()
  })

  it('keeps a user-picked platform when the setup query refetches', async () => {
    activePlatforms.value = ['meta']
    const { qc, w } = mountFeed()
    await settle()
    await flushPromises()

    await tab(w, 'TikTok').trigger('click')
    await settle()
    await flushPromises()
    expect(w.find('.feed-tab.active').text()).toContain('TikTok')

    // Vue Query refetches setup on window focus; a new object must not reset it.
    await qc.invalidateQueries({ queryKey: qk.setup })
    await settle()
    await flushPromises()
    expect(w.find('.feed-tab.active').text()).toContain('TikTok')
    w.unmount()
  })

  it('follows the navbar switcher while the user has not picked anything', async () => {
    activePlatforms.value = ['tiktok']
    const { w } = mountFeed()
    await settle()
    await flushPromises()
    expect(w.find('.feed-tab.active').text()).toContain('TikTok')
    activePlatforms.value = ['meta']
    await settle()
    await flushPromises()
    expect(w.find('.feed-tab.active').text()).toContain('Facebook')
    w.unmount()
    vi.restoreAllMocks()
  })
})
