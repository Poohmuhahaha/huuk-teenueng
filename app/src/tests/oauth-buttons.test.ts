// @vitest-environment happy-dom
import { afterEach, describe, expect, it, vi } from 'vitest'
import { flushPromises, mount } from '@vue/test-utils'
import OAuthButtons from '@/components/auth/OAuthButtons.vue'

function statusResponse(configured: boolean): Response {
  return new Response(JSON.stringify({ configured }), {
    status: 200,
    headers: { 'Content-Type': 'application/json' },
  })
}

afterEach(() => {
  vi.unstubAllGlobals()
})

describe('OAuthButtons', () => {
  it('renders Google and GitHub when both providers are configured', async () => {
    vi.stubGlobal('fetch', vi.fn(async () => statusResponse(true)))
    const w = mount(OAuthButtons)
    await flushPromises()

    expect(w.find('a[href="/api/auth/oauth/google/start"]').text()).toContain('Google')
    expect(w.find('a[href="/api/auth/oauth/github/start"]').text()).toContain('GitHub')
  })

  it('hides providers that are not configured', async () => {
    vi.stubGlobal(
      'fetch',
      vi.fn(async (url: string) => statusResponse(String(url).includes('/github/')))
    )
    const w = mount(OAuthButtons)
    await flushPromises()

    expect(w.find('a[href="/api/auth/oauth/google/start"]').exists()).toBe(false)
    expect(w.find('a[href="/api/auth/oauth/github/start"]').exists()).toBe(true)
  })

  it('renders nothing when no provider is configured', async () => {
    vi.stubGlobal('fetch', vi.fn(async () => statusResponse(false)))
    const w = mount(OAuthButtons)
    await flushPromises()

    expect(w.find('a').exists()).toBe(false)
  })
})
