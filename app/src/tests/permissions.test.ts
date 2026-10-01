// @vitest-environment happy-dom
import { afterAll, describe, expect, it } from 'vitest'
import { flushPromises, mount } from '@vue/test-utils'
import { QueryClient, VueQueryPlugin } from '@tanstack/vue-query'
import { defineComponent, h } from 'vue'
import * as api from '@/mock/api'
import { PERMISSIONS, setup } from '@/mock/db'
import type { Role } from '@/mock/db'
import { qk, usePermission } from '@/core/queries'
import { login, logout } from '@/core/auth'

function seedRoles(): Role[] {
  return [
    { name: 'Owner', permissions: [...PERMISSIONS] },
    {
      name: 'Editor',
      permissions: [
        'posts.write', 'posts.lock', 'metrics.import', 'ideas.write', 'hashtags.write',
        'content.write', 'content.publish',
      ],
    },
    { name: 'Viewer', permissions: [] },
    // The seeded demo directory user "Studio Client" holds this role.
    { name: 'Client', permissions: ['content.write', 'content.publish'] },
  ]
}

function settle(ms = 250): Promise<void> {
  return new Promise((r) => setTimeout(r, ms))
}

async function resetAuth(): Promise<void> {
  if (setup.authRequired) await login('owner@studio.local', 'demo1234').catch(() => undefined)
  await api.saveSetup({ authRequired: false, roles: seedRoles() }).catch(() => undefined)
  await logout()
}

// Viewer Vic (added to the directory in the first test) needs a real account;
// registering a name outside the directory would also yield the Viewer role.
async function ensureViewer(): Promise<void> {
  await api.register({ name: 'Viewer Vic', email: 'viewer@studio.local', password: 'demo12345678' }).catch(() => undefined)
}

describe('permissions (authRequired + roles)', () => {
  it('allows anonymous mutations in demo mode', async () => {
    await resetAuth()
    expect(setup.authRequired).toBe(false)
    const post = await api.updatePost('p-desk', { topic: 'Desk Setup Tour' })
    expect(post.topic).toBe('Desk Setup Tour')
    await expect(api.listPosts(2)).resolves.toBeTruthy()
    await api.addUser('Viewer Vic', 'Viewer')
  })

  it('requires a session and a permitted role once authRequired is on', async () => {
    await api.saveSetup({ authRequired: true })
    await expect(api.listPosts(2)).resolves.toBeTruthy()
    await expect(api.updatePost('p-desk', { topic: 'nope' })).rejects.toThrow(/401/)
    await login('editor@studio.local', 'demo1234')
    const post = await api.updatePost('p-desk', { topic: 'Desk Setup Tour' })
    expect(post.topic).toBe('Desk Setup Tour')
    await expect(api.saveSetup({ owner: 'Nope' })).rejects.toThrow(/403 forbidden: setup.write/)
    await logout()
    await ensureViewer()
    await login('viewer@studio.local', 'demo12345678')
    await expect(api.importMetrics('Instagram', 2)).rejects.toThrow(/403 forbidden: metrics.import/)
  })

  it('adds, validates, and removes roles', async () => {
    await logout()
    await login('owner@studio.local', 'demo1234')
    const added = await api.addRole('Growth', ['posts.write', 'metrics.import'])
    expect(added.roles.find((r) => r.name === 'Growth')?.permissions).toEqual(['posts.write', 'metrics.import'])
    await expect(api.addRole('Growth')).rejects.toThrow(/already exists/)
    await expect(api.addRole('   ')).rejects.toThrow(/required/)
    await expect(api.removeRole('Editor')).rejects.toThrow(/in use/)
    await expect(api.removeRole('Ghost')).rejects.toThrow(/not found/)
    const removed = await api.removeRole('Growth')
    expect(removed.roles.some((r) => r.name === 'Growth')).toBe(false)
  })

  it('validates a saved role matrix before applying it', async () => {
    await login('owner@studio.local', 'demo1234')
    const current = setup.roles.map((r) => ({ name: r.name, permissions: [...r.permissions] }))
    await expect(api.saveSetup({ roles: [] })).rejects.toThrow('at least one role is required')
    await expect(api.saveSetup({ roles: [...current, { name: 'Owner', permissions: [] }] }))
      .rejects.toThrow('duplicate role name: Owner')
    await expect(api.saveSetup({ roles: current.map((r) => (r.name === 'Viewer' ? { ...r, name: '  ' } : r)) }))
      .rejects.toThrow('role name is required')
    await expect(api.saveSetup({ roles: current.map((r) => (r.name === 'Viewer' ? { ...r, permissions: ['nope.perm'] } : r)) }))
      .rejects.toThrow("unknown permission 'nope.perm'")
    await expect(api.saveSetup({ roles: current.filter((r) => r.name !== 'Editor') }))
      .rejects.toThrow('role is in use: Editor')
    expect(setup.roles).toEqual(current)
  })

  it('applies a saved permission matrix', async () => {
    await login('owner@studio.local', 'demo1234')
    const roles = setup.roles.map((r) =>
      r.name === 'Viewer'
        ? { name: r.name, permissions: ['metrics.import'] }
        : { name: r.name, permissions: [...r.permissions] },
    )
    await api.saveSetup({ roles })
    await logout()
    await ensureViewer()
    await login('viewer@studio.local', 'demo12345678')
    await expect(api.importMetrics('Instagram', 2)).resolves.toBeTruthy()
    await expect(api.updatePost('p-desk', { topic: 'nope' })).rejects.toThrow(/403 forbidden: posts.write/)
  })

  afterAll(async () => {
    await resetAuth()
  })
})

describe('usePermission()', () => {
  it('is open in demo mode and follows the session role when required', async () => {
    await resetAuth()
    const qc = new QueryClient({ defaultOptions: { queries: { retry: false } } })
    const Probe = defineComponent({
      setup() {
        const { can, isLoggedIn, authRequired } = usePermission()
        return () =>
          h('div', [
            h('span', { id: 'write' }, String(can('posts.write'))),
            h('span', { id: 'import' }, String(can('metrics.import'))),
            h('span', { id: 'required' }, String(authRequired.value)),
            h('span', { id: 'logged' }, String(isLoggedIn.value)),
          ])
      },
    })
    const w = mount(Probe, { global: { plugins: [[VueQueryPlugin, { queryClient: qc }]] } })
    await settle()
    await flushPromises()
    expect(w.get('#write').text()).toBe('true')
    expect(w.get('#required').text()).toBe('false')
    expect(w.get('#logged').text()).toBe('false')

    await api.saveSetup({ authRequired: true })
    qc.setQueryData(qk.setup, await api.getSetup())
    await flushPromises()
    expect(w.get('#write').text()).toBe('false')
    expect(w.get('#required').text()).toBe('true')

    await login('editor@studio.local', 'demo1234')
    await flushPromises()
    expect(w.get('#write').text()).toBe('true')
    expect(w.get('#import').text()).toBe('true')
    expect(w.get('#logged').text()).toBe('true')

    await resetAuth()
  })
})

afterAll(async () => {
  await resetAuth()
})
