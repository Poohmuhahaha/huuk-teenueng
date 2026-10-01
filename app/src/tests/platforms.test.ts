// @vitest-environment happy-dom
import { beforeEach, describe, expect, it } from 'vitest'
import { flushPromises, mount } from '@vue/test-utils'
import { VueQueryPlugin, QueryClient } from '@tanstack/vue-query'
import SocialMediaBar from '@/components/ui/SocialMediaBar.vue'
import { activePlatform, activePlatforms, PLATFORMS, profileUrl, togglePlatform, validProfileHandle } from '@/core/platforms'
import * as api from '@/mock/api'

function settle(ms = 250): Promise<void> {
  return new Promise((r) => setTimeout(r, ms))
}

function mountBar() {
  const qc = new QueryClient({ defaultOptions: { queries: { retry: false } } })
  return mount(SocialMediaBar, {
    global: { plugins: [[VueQueryPlugin, { queryClient: qc }]] },
  })
}

beforeEach(() => {
  activePlatforms.value = ['meta']
})

describe('SocialMediaBar (wireframe)', () => {
  it('links a connected platform to its real profile', async () => {
    // The seed connects Meta with the Page's profile URL as handle.
    const w = mountBar()
    await settle()
    await flushPromises()
    const link = w.find('a.socialname')
    expect(link.exists()).toBe(true)
    expect(link.attributes('href')).toBe('https://www.facebook.com/profile.php?id=102400000000001')
    expect(link.attributes('target')).toBe('_blank')
    expect(link.attributes('rel')).toContain('noopener')
  })

  it('renders the label and a checkbox for every platform', async () => {
    const w = mountBar()
    await settle()
    await flushPromises()
    expect(w.text()).toContain('Social Media')
    expect(w.findAll('input[type="checkbox"]')).toHaveLength(3)
    const text = w.text()
    for (const name of ['Meta', 'YouTube', 'TikTok']) expect(text).toContain(name)
    // Instagram has no separate connection — the Meta login covers it.
    expect(PLATFORMS.map((p) => p.id)).toEqual(['meta', 'youtube', 'tiktok'])
  })

  it('checks by default and toggles platforms on/off', async () => {
    const w = mountBar()
    await settle()
    await flushPromises()
    const boxes = w.findAll('input[type="checkbox"]')
    expect((boxes[0].element as HTMLInputElement).checked).toBe(true)

    await boxes[1].setValue(true) // YouTube
    expect(activePlatforms.value).toContain('youtube')
    expect(activePlatform.value).toBe('meta') // primary = first in registry order

    await boxes[0].setValue(false) // uncheck Meta
    expect(activePlatforms.value).not.toContain('meta')
    expect(activePlatform.value).toBe('youtube') // primary = first remaining in registry order
  })

  it('never unchecks the last remaining platform', async () => {
    activePlatforms.value = ['tiktok']
    const w = mountBar()
    await settle()
    await flushPromises()
    // platform order is meta, youtube, tiktok
    const box = w.findAll('input[type="checkbox"]')[2]
    await box.setValue(false)
    expect(activePlatforms.value).toEqual(['tiktok'])
    // the DOM checkbox must snap back too (controlled-input desync regression)
    expect((box.element as HTMLInputElement).checked).toBe(true)
  })

  it('keeps registry order when re-checking a platform', () => {
    activePlatforms.value = ['youtube']
    togglePlatform('meta')
    expect(activePlatforms.value).toEqual(['meta', 'youtube'])
    expect(activePlatform.value).toBe('meta')
  })
})

describe('platform mock api', () => {
  it('reports mock OAuth mode when no credentials are configured', async () => {
    const s = await api.oauthStatus('meta')
    expect(s.mode).toBe('mock')
    expect(s.configured).toBe(false)
    const start = await api.startOauth('meta')
    expect(start.mode).toBe('mock')
  })

  it('only links real profiles, never platform home pages', () => {
    expect(profileUrl('instagram', 'https://www.instagram.com')).toBeNull()
    expect(profileUrl('instagram', 'https://www.instagram.com/')).toBeNull()
    expect(profileUrl('instagram', 'https://instagram.com/my.brand')).toBe('https://instagram.com/my.brand')
    expect(profileUrl('tiktok', 'https://www.instagram.com/someone')).toBeNull()
    expect(profileUrl('youtube', '@my.channel')).toBe('https://youtube.com/@my.channel')
    expect(profileUrl('meta', 'https://www.facebook.com/profile.php?id=61592348575800')).toBe(
      'https://www.facebook.com/profile.php?id=61592348575800',
    )
    expect(profileUrl('meta', 'Studio Channel')).toBeNull()

    expect(validProfileHandle('meta', 'https://www.facebook.com')).toBe(false)
    expect(validProfileHandle('meta', 'https://www.facebook.com/profile.php?id=61592348575800')).toBe(true)
    expect(validProfileHandle('meta', '@yourpage')).toBe(true)
    expect(validProfileHandle('meta', 'not a handle')).toBe(false)
  })

  it('connects with a real profile handle and links to it', async () => {
    const c = await api.connectPlatform('youtube', '@my.channel')
    expect(c.status).toBe('connected')
    expect(c.handle).toBe('@my.channel')
    expect(profileUrl('youtube', c.handle)).toBe('https://youtube.com/@my.channel')
    await api.disconnectPlatform('youtube')
  })

  it('connects and disconnects a platform', async () => {
    const c = await api.connectPlatform('youtube')
    expect(c.status).toBe('connected')
    expect(c.tokenType).toBe('refresh')
    const d = await api.disconnectPlatform('youtube')
    expect(d.status).toBe('disconnected')
    expect(d.expiresAt).toBeNull()
  })

  it('sync bumps lastSync and media count', async () => {
    await api.connectPlatform('tiktok')
    const before = (await api.listPlatforms()).find((p) => p.id === 'tiktok')!
    const after = await api.syncPlatform('tiktok')
    expect(after.mediaCount).toBe(before.mediaCount + 3)
    expect(after.lastSync).toBeTruthy()
    await api.disconnectPlatform('tiktok')
  })
})
