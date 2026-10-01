import { describe, expect, it } from 'vitest'
import * as api from '@/mock/api'
import { login, logout } from '@/core/auth'
import { setup } from '@/mock/db'

describe('content studio (mock parity)', () => {
  it('lists pinned seed content and creates a draft', async () => {
    const rows = await api.listContent()
    expect(rows.length).toBeGreaterThanOrEqual(3)
    const created = await api.createContent({ title: 'Studio Test', kind: 'note' })
    expect(created.id).toMatch(/^c-/)
    expect(created.slug).toBe('studio-test')
    expect(created.status).toBe('draft')
    expect(created.author).toBe('Studio Owner')
  })

  it('enforces optimistic concurrency and stores revisions', async () => {
    const created = await api.createContent({ title: 'Version Test', body: 'one' })
    const updated = await api.updateContent(created.id, { version: created.version, body: 'two', note: 'Edit' })
    expect(updated.version).toBe(created.version + 1)
    expect(updated.revisions).toHaveLength(1)
    expect(updated.revisions[0].body).toBe('')

    await expect(api.updateContent(created.id, { version: created.version, body: 'stale' }))
      .rejects.toThrow(/version/)

    const full = await api.listContentRevisions(created.id)
    expect(full[0].body).toBe('one')

    const restored = await api.restoreContentRevision(created.id, full[0].revision, updated.version)
    expect(restored.body).toBe('one')
    expect(restored.status).toBe('draft')
  })

  it('publishes, serves publicly, and unpublishes', async () => {
    const created = await api.createContent({ title: 'Public Test', body: '# hi', excerpt: 'Hello' })
    const published = await api.publishContent(created.id, created.version)
    expect(published.status).toBe('published')
    expect(published.publishedAt).toBeTruthy()

    const publicList = await api.listPublicContent()
    expect(publicList.some((c) => c.id === created.id)).toBe(true)
    const publicItem = await api.getPublicContent(created.slug)
    expect(publicItem.body).toBe('# hi')

    const unpublished = await api.unpublishContent(created.id, published.version)
    expect(unpublished.status).toBe('draft')
    await expect(api.getPublicContent(created.slug)).rejects.toThrow()
  })

  it('validates content fields', async () => {
    await expect(api.createContent({ title: '   ' })).rejects.toThrow(/title/)
    await expect(api.createContent({ title: 'Bad slug', slug: 'Not A Slug' })).rejects.toThrow(/slug/)
    await expect(api.createContent({ title: 'Bad kind', kind: 'tweet' as never })).rejects.toThrow(/kind/)
  })

  it('persists an editable workspace name', async () => {
    const saved = await api.saveSetup({ workspaceName: '  Acme Studio  ' })
    expect(saved.workspaceName).toBe('Acme Studio')
    const read = await api.getSetup()
    expect(read.workspaceName).toBe('Acme Studio')

    await expect(api.saveSetup({ workspaceName: '   ' })).rejects.toThrow(/workspace/i)
    await expect(api.saveSetup({ workspaceName: 'x'.repeat(81) })).rejects.toThrow(/workspace/i)

    await api.saveSetup({ workspaceName: 'workspace' })
  })
})

describe('client-facing role + onboarding', () => {
  it('gives the Client role a studio path but not workspace settings', async () => {
    const roles = setup.roles.map((r) => r.name)
    expect(roles).toContain('Client')

    const client = await login('client@studio.local', 'demo1234')
    expect(client.role).toBe('Client')

    await api.saveSetup({ authRequired: true })
    const created = await api.createContent({ title: 'Client Piece', body: 'hello' })
    expect(created.author).toBe('Studio Client')
    const published = await api.publishContent(created.id, created.version)
    expect(published.status).toBe('published')

    await expect(api.saveSetup({ owner: 'Hacked' })).rejects.toThrow(/setup.write/)
    await expect(api.deleteContent(created.id)).rejects.toThrow(/content.delete/)

    // owner can delete and reset
    await logout()
    await login('owner@studio.local', 'demo1234')
    await api.deleteContent(created.id)
    await api.saveSetup({ authRequired: false })
    await logout()
  })

  it('lets an admin onboard a client with a generated password', async () => {
    await login('owner@studio.local', 'demo1234')
    const created = await api.createAccount({
      name: 'New Client',
      email: 'new-client@test.local',
      role: 'Client',
    })
    expect(created.account.role).toBe('Client')
    expect(created.temporaryPassword).toBeTruthy()

    await logout()
    const session = await login('new-client@test.local', created.temporaryPassword as string)
    expect(session.role).toBe('Client')
    await logout()

    await login('owner@studio.local', 'demo1234')
    await api.deleteAccount('new-client@test.local')
    await logout()
  })
})
