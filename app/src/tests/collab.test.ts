import { describe, expect, it } from 'vitest'
import * as api from '@/mock/api'
import { setup } from '@/mock/db'

describe('auth (US-012)', () => {
  it('logs in an existing account, resolves me, and logs out', async () => {
    await expect(api.login('nobody@studio.local', 'demo1234')).rejects.toThrow('invalid email or password')
    const r = await api.login('owner@studio.local', 'demo1234')
    expect(r.token).toBeTruthy()
    expect(r.user.name).toBe('Studio Owner')
    expect(r.user.role).toBe('Owner')

    const who = await api.me(r.token)
    expect(who.user.name).toBe('Studio Owner')

    await api.logout(r.token)
    await expect(api.me(r.token)).rejects.toThrow()
  })
})

describe('users (US-011)', () => {
  it('adds a user (role defaults to Editor) and rejects duplicates', async () => {
    const s1 = await api.addUser('Temp Editor', '')
    expect(s1.users.find((u) => u.name === 'Temp Editor')?.role).toBe('Editor')
    await expect(api.addUser('Temp Editor', 'Editor')).rejects.toThrow()
    const s2 = await api.removeUser('Temp Editor')
    expect(s2.users.some((u) => u.name === 'Temp Editor')).toBe(false)
  })

  it('never removes the last remaining user', async () => {
    // Snapshot and restore: later tests rely on the seeded directory roles.
    const original = setup.users.map((u) => ({ ...u }))
    try {
      for (const name of setup.users.slice(0, -1).map((u) => u.name)) await api.removeUser(name)
      expect(setup.users).toHaveLength(1)
      await expect(api.removeUser(setup.users[0].name)).rejects.toThrow(/last user/)
    } finally {
      setup.users = original
    }
  })
})

describe('post locking (US-014)', () => {
  it('lets the lock holder edit and blocks everyone else', async () => {
    await api.lockPost('p-desk', 'User A')
    await expect(api.updatePost('p-desk', { topic: 'x', user: 'User B' })).rejects.toThrow(/locked/)
    const kept = await api.updatePost('p-desk', { topic: 'Desk Setup Tour', user: 'User A' })
    expect(kept.topic).toBe('Desk Setup Tour')
    await expect(api.unlockPost('p-desk', 'User B')).rejects.toThrow(/locked/)
    await api.unlockPost('p-desk', 'User A')
    const freed = await api.updatePost('p-desk', { topic: 'Desk Setup Tour', user: 'User B' })
    expect(freed.topic).toBe('Desk Setup Tour')
  })

  it('respects a lock seeded by another user', async () => {
    const locked = (await api.listPosts(2)).find((p) => p.lockedBy)
    expect(locked).toBeTruthy()
    await expect(api.updatePost(locked!.id, { topic: 'nope', user: 'User B' })).rejects.toThrow(locked!.lockedBy!)
  })

  it('creates new posts unlocked regardless of client input', async () => {
    const created = await api.addPost({
      month: 2, topic: 'Fresh Draft', pillar: 'Pillar I', format: 'Short-video', goal: '10K subscribe',
      date: null, time: '09:00', status: 'Start', hook: '', caption: '', cta: '',
      hashtagGroup: 'Reach', hashtags: [], imageUrl: '', note: '', done: false, platforms: [],
      lockedBy: 'Ghost',
    })
    expect(created.lockedBy).toBeNull()
  })
})

describe('session-derived locks (US-014)', () => {
  it('uses the session name when auth is required and ignores client-supplied users', async () => {
    await api.login('editor@studio.local', 'demo1234')
    await api.saveSetup({ authRequired: true })
    const locked = await api.lockPost('p-desk', 'Somebody Else')
    expect(locked.lockedBy).toBe('Editor Earn')

    await api.login('owner@studio.local', 'demo1234')
    await expect(api.updatePost('p-desk', { topic: 'nope', user: 'Editor Earn' })).rejects.toThrow(/locked by Editor Earn/)
    await expect(api.unlockPost('p-desk', 'Editor Earn')).rejects.toThrow(/locked by Editor Earn/)

    await api.login('editor@studio.local', 'demo1234')
    const freed = await api.unlockPost('p-desk', 'Not The Editor')
    expect(freed.lockedBy ?? null).toBeNull()

    await api.login('owner@studio.local', 'demo1234')
    await api.saveSetup({ authRequired: false })
  })
})

describe('metrics import (US-008)', () => {
  it('imports deterministic rows for a platform + month', async () => {
    await expect(api.importMetrics('  ', 2)).rejects.toThrow()
    const r1 = await api.importMetrics('Instagram', 2)
    expect(r1.imported).toBeGreaterThan(0)
    for (const m of r1.metrics) {
      expect(m.platform).toBe('Instagram')
      expect(m.views).toBeGreaterThanOrEqual(1000)
      expect(m.likes).toBe(Math.floor(m.views / 12))
    }
    const r2 = await api.importMetrics('Instagram', 2)
    expect(r2.metrics).toEqual(r1.metrics)
  })
})
