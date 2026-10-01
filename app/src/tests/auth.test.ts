// @vitest-environment happy-dom
import { afterEach, describe, expect, it, vi } from 'vitest'
import * as api from '@/mock/api'
import { currentUser, login, logout, register, restore, updateProfile } from '@/core/auth'
import { setToken, token } from '@/core/token'

describe('register (US-012)', () => {
  it('rejects an empty name, a malformed email, and a short password', async () => {
    await expect(api.register({ name: '   ', email: 'valid@studio.local', password: 'password1234' })).rejects.toThrow(/name/)
    await expect(api.register({ name: 'Valid Name', email: 'no-at-sign', password: 'password1234' })).rejects.toThrow(/email/)
    await expect(api.register({ name: 'Valid Name', email: 'valid@studio.local', password: 'short' })).rejects.toThrow(/12/)
  })

  it('rejects a duplicate email', async () => {
    await expect(api.register({ name: 'Dup One', email: 'dup@studio.local', password: 'password1234' })).resolves.toBeTruthy()
    await expect(api.register({ name: 'Dup Two', email: 'dup@studio.local', password: 'password1234' })).rejects.toThrow(/already/)
  })

  it('rejects a name already used by another account', async () => {
    await expect(
      api.register({ name: '  Studio Owner  ', email: 'second-owner@studio.local', password: 'password1234' }),
    ).rejects.toThrow('name already taken')
  })

  it('defaults to Owner and follows the Setup directory role when the name exists', async () => {
    const fresh = await api.register({ name: 'Fresh Viewer', email: 'fresh@studio.local', password: 'password1234' })
    expect(fresh.user.role).toBe('Owner')
    expect(fresh.user.plan).toBe('free')

    await api.addUser('Dir Editor', 'Editor')
    const known = await api.register({ name: 'Dir Editor', email: 'direditor@studio.local', password: 'password1234' })
    expect(known.user.role).toBe('Editor')
  })
})

describe('login (US-012)', () => {
  it('uses one message for an unknown email and a wrong password', async () => {
    const missing = await api.login('nobody@studio.local', 'demo1234').catch((e: Error) => e.message)
    const wrong = await api.login('owner@studio.local', 'wrong-password').catch((e: Error) => e.message)
    expect(missing).toBe('invalid email or password')
    expect(wrong).toBe('invalid email or password')
  })

  it('returns the session user with its directory role', async () => {
    const r = await api.login('editor@studio.local', 'demo1234')
    expect(r.token).toBeTruthy()
    expect(r.user).toMatchObject({ name: 'Editor Earn', role: 'Editor', plan: 'free' })
  })
})

describe('change-password (US-012)', () => {
  it('rejects a wrong old password and a short new password', async () => {
    await api.register({ name: 'Pw One', email: 'pwone@studio.local', password: 'password1234' })
    await expect(api.changePassword('wrong-old', 'anotherpass12')).rejects.toThrow()
    await expect(api.changePassword('password1234', 'short')).rejects.toThrow(/12/)
  })

  it('changes the password, revokes other sessions, and rejects the old password', async () => {
    await api.register({ name: 'Pw Two', email: 'pwtwo@studio.local', password: 'password1234' })
    const first = await api.login('pwtwo@studio.local', 'password1234')
    const second = await api.login('pwtwo@studio.local', 'password1234')
    await api.me(second.token)
    await api.changePassword('password1234', 'newpassword12')
    await expect(api.me(first.token)).rejects.toThrow()
    await expect(api.me(second.token)).resolves.toBeTruthy()
    await expect(api.login('pwtwo@studio.local', 'password1234')).rejects.toThrow('invalid email or password')
    await expect(api.login('pwtwo@studio.local', 'newpassword12')).resolves.toBeTruthy()
  })
})

describe('logoutAll (US-012)', () => {
  it('revokes every session of the account and leaves other accounts alone', async () => {
    const first = await api.register({ name: 'All Out', email: 'allout@studio.local', password: 'password1234' })
    const second = await api.login('allout@studio.local', 'password1234')
    const other = await api.login('owner@studio.local', 'demo1234')
    await api.me(second.token)

    await expect(api.logoutAll()).resolves.toEqual({ ok: true })
    await expect(api.logoutAll()).rejects.toThrow('401 unauthorized')
    await expect(api.me(first.token)).rejects.toThrow()
    await expect(api.me(second.token)).rejects.toThrow()
    await expect(api.me(other.token)).resolves.toBeTruthy()
  })
})

describe('account administration (US-012)', () => {
  afterEach(() => vi.useRealTimers())

  it('lists accounts with live session counts and deletes an account with its sessions', async () => {
    const created = await api.register({ name: 'Temp Admin', email: 'tempadmin@studio.local', password: 'password1234' })
    const second = await api.login('tempadmin@studio.local', 'password1234')
    const listed = await api.listAccounts()
    expect(listed.find((a) => a.email === 'tempadmin@studio.local')).toEqual({
      name: 'Temp Admin',
      email: 'tempadmin@studio.local',
      sessions: 2,
    })
    expect(listed.map((a) => a.name)).toEqual([...listed.map((a) => a.name)].sort())

    await api.me(created.token)
    await api.deleteAccount('tempadmin@studio.local')
    await expect(api.me(created.token)).rejects.toThrow()
    await expect(api.me(second.token)).rejects.toThrow()
    await expect(api.deleteAccount('tempadmin@studio.local')).rejects.toThrow('account not found')

    const after = await api.listAccounts()
    expect(after.some((a) => a.email === 'tempadmin@studio.local')).toBe(false)
    expect(after.some((a) => a.email === 'owner@studio.local')).toBe(true)
  })

  it('counts only live sessions', async () => {
    await api.register({ name: 'Stale Session', email: 'stale@studio.local', password: 'password1234' })
    vi.useFakeTimers({ toFake: ['Date'] })
    vi.setSystemTime(Date.now() + 8 * 24 * 60 * 60 * 1000)
    const listed = await api.listAccounts()
    expect(listed.find((a) => a.email === 'stale@studio.local')?.sessions).toBe(0)
    vi.useRealTimers()
  })
})

describe('token persistence (US-012)', () => {
  afterEach(() => vi.useRealTimers())

  it('setToken writes to localStorage and clears it', () => {
    setToken('tok-123')
    expect(token.value).toBe('tok-123')
    expect(localStorage.getItem('cp.token')).toBe('tok-123')
    setToken(null)
    expect(token.value).toBeNull()
    expect(localStorage.getItem('cp.token')).toBeNull()
  })

  it('restore() rehydrates a valid session', async () => {
    const r = await api.login('owner@studio.local', 'demo1234')
    setToken(r.token)
    currentUser.value = null
    await restore()
    expect(currentUser.value).toMatchObject({ name: 'Studio Owner', role: 'Owner', plan: 'free' })
  })

  it('restore() clears a bad token', async () => {
    setToken('bogus-token')
    await restore()
    expect(token.value).toBeNull()
    expect(currentUser.value).toBeNull()
    expect(localStorage.getItem('cp.token')).toBeNull()
  })

  it('me() rejects once the session expires', async () => {
    vi.useFakeTimers({ toFake: ['Date'] })
    const r = await api.login('editor@studio.local', 'demo1234')
    vi.setSystemTime(Date.now() + 8 * 24 * 60 * 60 * 1000)
    await expect(api.me(r.token)).rejects.toThrow()
  })

  it('auth.register/login/logout drive the shared session state', async () => {
    const created = await register({ name: 'Auth Reg', email: 'authreg@studio.local', password: 'password1234' })
    expect(created.name).toBe('Auth Reg')
    expect(currentUser.value?.name).toBe('Auth Reg')
    expect(token.value).toBeTruthy()
    expect(localStorage.getItem('cp.token')).toBe(token.value)

    await logout()
    expect(currentUser.value).toBeNull()
    expect(token.value).toBeNull()
    expect(localStorage.getItem('cp.token')).toBeNull()

    const u = await login('owner@studio.local', 'demo1234')
    expect(u.name).toBe('Studio Owner')
    expect(currentUser.value?.role).toBe('Owner')
    await logout()
  })
})

describe('updateProfile (US-012)', () => {
  it('rejects without an active session', async () => {
    await api.logoutAll().catch(() => undefined)
    await logout()
    await expect(api.updateProfile('No Session')).rejects.toThrow('401 unauthorized')
  })

  it('renames the account, keeps the directory role, and updates the signed-in user', async () => {
    await api.addUser('Profile One', 'Editor')
    await register({ name: 'Profile One', email: 'profile1@studio.local', password: 'password1234' })
    expect(currentUser.value).toMatchObject({ name: 'Profile One', role: 'Editor' })

    const updated = await updateProfile('Profile Renamed')
    expect(updated).toMatchObject({ name: 'Profile Renamed', role: 'Editor' })
    expect(currentUser.value).toMatchObject({ name: 'Profile Renamed', role: 'Editor' })

    const dir = await api.getSetup()
    expect(dir.users.find((u) => u.name === 'Profile Renamed')?.role).toBe('Editor')
    await expect(api.login('profile1@studio.local', 'password1234'))
      .resolves.toMatchObject({ user: { name: 'Profile Renamed', role: 'Editor' } })
  })

  it('rejects a name already taken by another account', async () => {
    await login('owner@studio.local', 'demo1234')
    await expect(updateProfile('Editor Earn')).rejects.toThrow('name already taken')
    expect(currentUser.value?.name).toBe('Studio Owner')
  })
})
