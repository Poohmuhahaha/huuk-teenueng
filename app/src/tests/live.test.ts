// @vitest-environment happy-dom
// Live mirror section: real posts + engagement pulled from the connected
// platforms, with an on-demand sync (mock API here).
import { describe, expect, it } from 'vitest'
import { flushPromises, mount } from '@vue/test-utils'
import { QueryClient, VueQueryPlugin } from '@tanstack/vue-query'
import LiveSection from '@/components/live/LiveSection.vue'
import LivePage from '@/pages/LivePage.vue'
import * as api from '@/mock/api'
import { connections, live, liveFetchedAt } from '@/mock/db'

const settle = (ms = 300): Promise<void> => new Promise((r) => setTimeout(r, ms))

function mountLive(props: { variant?: 'cards' | 'grid' | 'table'; limit?: number } = {}) {
  const qc = new QueryClient({ defaultOptions: { queries: { retry: false } } })
  return mount(LiveSection, {
    props,
    global: { plugins: [[VueQueryPlugin, { queryClient: qc }]] },
  })
}

function restoreSeed(): void {
  live.posts = [
    {
      id: '102400000000001_9001', platform: 'meta', kind: 'photo',
      caption: 'Behind the scenes of the spring shoot.', mediaUrl: '', thumbnailUrl: '',
      permalink: 'https://www.facebook.com/102400000000001_9001',
      createdAt: '2026-03-01T06:00:00+0000', likes: 214, comments: 18, shares: 6, views: 0,
      reactions: { like: 186, love: 24, wow: 4 }, reach: 0, saves: 0,
      clicks: 62, linkClicks: 21, videoLength: 0, videoAvgWatchTime: 0,
      attachmentTitle: '', linkUrl: '',
      fetchedAt: liveFetchedAt,
    },
    {
      id: '102400000000001_9002', platform: 'meta', kind: 'video',
      caption: '60-second studio tour.', mediaUrl: '', thumbnailUrl: '',
      permalink: 'https://www.facebook.com/102400000000001_9002',
      createdAt: '2026-02-26T09:30:00+0000', likes: 132, comments: 9, shares: 4, views: 5400,
      reactions: { like: 118, love: 11 }, reach: 0, saves: 0,
      clicks: 48, linkClicks: 12, videoLength: 60, videoAvgWatchTime: 18.4,
      attachmentTitle: '60-second studio tour', linkUrl: '',
      fetchedAt: liveFetchedAt,
    },
    {
      id: '17841400000000001_7001', platform: 'instagram', kind: 'reel',
      caption: 'Slow living, honest reviews.', mediaUrl: '', thumbnailUrl: '',
      permalink: 'https://www.instagram.com/reel/7001/',
      createdAt: '2026-02-24T12:00:00+0000', likes: 96, comments: 7, shares: 0, views: 3100,
      reactions: {}, reach: 2600, saves: 31, clicks: 0, linkClicks: 0,
      videoLength: 22, videoAvgWatchTime: 9.1,
      attachmentTitle: '', linkUrl: '',
      fetchedAt: liveFetchedAt,
    },
  ]
  live.fetchedAt = liveFetchedAt
  live.error = ''
}

describe('LivePage', () => {
  it('lists the whole mirror without a row limit', async () => {
    restoreSeed()
    const qc = new QueryClient({ defaultOptions: { queries: { retry: false } } })
    const w = mount(LivePage, { global: { plugins: [[VueQueryPlugin, { queryClient: qc }]] } })
    await settle()
    await flushPromises()
    expect(w.text()).toContain('Live content')
    expect(w.findAll('.livecard')).toHaveLength(3)
    w.unmount()
  })
})

describe('LiveSection', () => {
  it('renders mirrored posts with engagement and links to the real post', async () => {
    restoreSeed()
    const w = mountLive()
    await settle()
    await flushPromises()

    expect(w.text()).toContain('Live from Meta')
    const cards = w.findAll('.livecard')
    expect(cards).toHaveLength(3)
    expect(cards[0].attributes('href')).toBe('https://www.facebook.com/102400000000001_9001')
    // The complete per-post data: reaction breakdown, clicks, watch time.
    expect(cards[0].text()).toContain('Like 186')
    expect(cards[0].text()).toContain('Clicks 62')
    expect(cards[0].text()).toContain('Link clicks 21')
    expect(cards[1].text()).toContain('Avg watch 0:18')
    expect(w.text()).toContain('engagements')
    expect(w.text()).toContain('page views')
    expect(cards[0].attributes('rel')).toContain('noopener')
    const text = w.text()
    expect(text).toContain('Studio Channel')
    expect(text).toContain('@studio.channel')
    expect(text).toContain('214')
    expect(text).toContain('Instagram · reel')
    w.unmount()
  })

  it('limits the number of posts when a limit is given', async () => {
    restoreSeed()
    const w = mountLive({ limit: 2 })
    await settle()
    await flushPromises()
    expect(w.findAll('.livecard')).toHaveLength(2)
    w.unmount()
  })

  it('renders the table variant with the engagement columns', async () => {
    restoreSeed()
    const w = mountLive({ variant: 'table' })
    await settle()
    await flushPromises()
    expect(w.find('table').exists()).toBe(true)
    const text = w.text()
    for (const header of ['Likes', 'Comments', 'Shares', 'Views']) expect(text).toContain(header)
    expect(w.findAll('tbody tr')).toHaveLength(3)
    w.unmount()
  })

  it('syncs the mirror on demand', async () => {
    restoreSeed()
    const w = mountLive()
    await settle()
    await flushPromises()
    expect(live.fetchedAt).toBe(liveFetchedAt)

    await w.find('.live-sync').trigger('click')
    await settle(900)
    await flushPromises()

    expect(live.fetchedAt).toBeGreaterThan(liveFetchedAt)
    expect(w.text()).toContain('just now')
    w.unmount()
  })

  it('explains how to get content when nothing was fetched yet', async () => {
    restoreSeed()
    live.posts = []
    live.fetchedAt = null
    const connected = mountLive()
    await settle()
    await flushPromises()
    expect(connected.text()).toMatch(/connected — press sync now/i)
    connected.unmount()

    const meta = connections.find((c) => c.id === 'meta')!
    meta.status = 'disconnected'
    const w = mountLive()
    await settle()
    await flushPromises()
    expect(w.text()).toMatch(/connect meta in the social bar/i)
    meta.status = 'connected'
    restoreSeed()
    w.unmount()
  })

  it('offers a one-click reconnect when the token is missing', async () => {
    restoreSeed()
    const meta = connections.find((c) => c.id === 'meta')!
    meta.hasToken = false
    live.posts = []
    live.fetchedAt = null
    const w = mountLive()
    await settle()
    await flushPromises()

    expect(w.text()).toMatch(/needs a fresh login/i)
    const button = w.find('.live-empty button')
    expect(button.exists()).toBe(true)
    expect(button.text()).toContain('Reconnect Meta')

    // In mock mode there are no credentials, so the action explains that.
    await button.trigger('click')
    await settle(150)
    await flushPromises()
    expect(w.find('.autherr').text()).toMatch(/not configured/i)

    meta.hasToken = true
    restoreSeed()
    w.unmount()
  })

  it('shows the last refresh failure without hiding the posts', async () => {
    restoreSeed()
    live.error = 'graph error: session has expired'
    const w = mountLive()
    await settle()
    await flushPromises()
    expect(w.text()).toContain('Last refresh failed')
    expect(w.text()).toContain('session has expired')
    expect(w.findAll('.livecard')).toHaveLength(3)
    restoreSeed()
    w.unmount()
  })

  it('goes through the API contract in mock mode', async () => {
    const data = await api.getLive()
    expect(data.posts.length).toBeGreaterThan(0)
    expect(data.accounts.some((a) => a.platform === 'meta')).toBe(true)
  })
})
