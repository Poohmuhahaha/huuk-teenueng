// @vitest-environment happy-dom
import { describe, expect, it } from 'vitest'
import { flushPromises, mount } from '@vue/test-utils'
import { QueryClient, VueQueryPlugin } from '@tanstack/vue-query'
import PlatformLogin from '@/components/overlays/PlatformLogin.vue'
import type { PlatformId } from '@/core/platforms'
import * as api from '@/mock/api'
import { connections } from '@/mock/db'

const settle = (ms = 300): Promise<void> => new Promise((r) => setTimeout(r, ms))

function mountLogin(platform: PlatformId) {
  const qc = new QueryClient({ defaultOptions: { queries: { retry: false } } })
  return mount(PlatformLogin, {
    props: { platform },
    global: { plugins: [[VueQueryPlugin, { queryClient: qc }]] },
  })
}

describe('PlatformLogin manual connect', () => {
  it('links a public profile when OAuth is not configured', async () => {
    const w = mountLogin('youtube')
    await settle(400)
    await flushPromises()
    await w.find('#plogin-handle').setValue('@my.channel')
    await w.find('button.btn-primary').trigger('click')
    await settle(700)
    await flushPromises()

    const conn = (await api.listPlatforms()).find((p) => p.id === 'youtube')
    expect(conn?.status).toBe('connected')
    expect(conn?.handle).toBe('@my.channel')
    await api.disconnectPlatform('youtube')
    w.unmount()
  })

  it('rejects the platform home page as a profile', async () => {
    const w = mountLogin('tiktok')
    await settle(400)
    await flushPromises()
    await w.find('#plogin-handle').setValue('https://www.tiktok.com')
    await w.find('button.btn-primary').trigger('click')
    await settle(300)
    await flushPromises()
    expect(w.find('.autherr').text()).toMatch(/profile link/i)
    const conn = (await api.listPlatforms()).find((p) => p.id === 'tiktok')
    expect(conn?.handle).not.toBe('https://www.tiktok.com')
    w.unmount()
  })

  it('requires a profile before connecting', async () => {
    const w = mountLogin('tiktok')
    await settle(400)
    await flushPromises()
    expect(w.find('button.btn-primary').attributes('disabled')).toBeDefined()
    w.unmount()
  })
})

describe('PlatformLogin exit affordances (dead-end regression)', () => {
  it('closes when the popup backdrop is clicked', async () => {
    const w = mountLogin('tiktok')
    await settle(400)
    await w.find('.plogin').trigger('click')
    expect(w.emitted('close')).toBeTruthy()
    w.unmount()
  })

  it('offers a close button on the login step', async () => {
    const w = mountLogin('youtube')
    await settle()
    await flushPromises()
    const close = w.find('.plogin-close')
    expect(close.exists()).toBe(true)
    await close.trigger('click')
    expect(w.emitted('close')).toBeTruthy()
  })

  it('offers a close button on the connected/maintenance view', async () => {
    const w = mountLogin('meta')
    await settle()
    await flushPromises()
    expect(w.text()).toContain('Connected')
    const close = w.find('.plogin-close')
    expect(close.exists()).toBe(true)
    await close.trigger('click')
    expect(w.emitted('close')).toBeTruthy()
  })

  it('flags a connection whose token is missing', async () => {
    const meta = connections.find((c) => c.id === 'meta')!
    meta.hasToken = false
    const w = mountLogin('meta')
    await settle()
    await flushPromises()
    expect(w.text()).toMatch(/needs a fresh login/i)
    meta.hasToken = true
    w.unmount()
  })

  it('gives the account URL a full-width row so it cannot overflow', async () => {
    const w = mountLogin('meta')
    await settle()
    await flushPromises()
    const full = w.find('.plogin-full')
    expect(full.exists()).toBe(true)
    expect(full.text()).toContain('facebook.com')
    w.unmount()
  })
})
