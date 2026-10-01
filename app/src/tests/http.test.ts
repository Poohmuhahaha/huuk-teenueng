// @vitest-environment happy-dom
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { ApiError, isAuthError, normalizeBase } from '@/api/http'
import * as api from '@/api/http'
import { currentUser, startSession } from '@/core/session'
import { setToken, token } from '@/core/token'
import { setActiveWorkspace } from '@/core/workspace'

function jsonResponse(body: unknown, status = 200): Response {
  return new Response(JSON.stringify(body), {
    status,
    headers: { 'Content-Type': 'application/json' },
  })
}

describe('normalizeBase', () => {
  it('handles empty, root, trailing slashes, and /api suffixes', () => {
    expect(normalizeBase(undefined)).toBe('')
    expect(normalizeBase('')).toBe('')
    expect(normalizeBase('/')).toBe('')
    expect(normalizeBase('http://localhost:8787/')).toBe('http://localhost:8787')
    expect(normalizeBase('https://api.example.com/api')).toBe('https://api.example.com')
    expect(normalizeBase('https://api.example.com/api/')).toBe('https://api.example.com')
    expect(normalizeBase('api.example.com')).toBe('/api.example.com')
  })
})

describe('HTTP client', () => {
  let fetchMock: ReturnType<typeof vi.fn>

  beforeEach(() => {
    fetchMock = vi.fn()
    vi.stubGlobal('fetch', fetchMock)
    setToken(null)
    currentUser.value = null
  })

  afterEach(() => {
    vi.unstubAllGlobals()
    setToken(null)
    currentUser.value = null
  })

  it('surfaces the server error message, not the raw body', async () => {
    fetchMock.mockResolvedValue(jsonResponse({ error: 'invalid email or password' }, 401))
    const error = await api.login('a@b.co', 'wrong').catch((e: unknown) => e)
    expect(error).toBeInstanceOf(ApiError)
    expect((error as ApiError).status).toBe(401)
    expect((error as ApiError).message).toBe('invalid email or password')
    // A failed login must not disturb an existing session.
    expect(isAuthError(error)).toBe(true)
  })

  it('does not send an Authorization header or Content-Type on plain GETs', async () => {
    fetchMock.mockResolvedValue(jsonResponse([]))
    await api.listPosts(2)
    const [url, init] = fetchMock.mock.calls[0] as [string, RequestInit]
    expect(url).toContain('/api/posts?month=2')
    const headers = init.headers as Record<string, string>
    expect(headers.Authorization).toBeUndefined()
    expect(headers['Content-Type']).toBeUndefined()
  })

  it('does not send Content-Type on authenticated GETs', async () => {
    fetchMock.mockResolvedValue(jsonResponse([]))
    await api.listAccounts()
    const [, init] = fetchMock.mock.calls[0] as [string, RequestInit]
    const headers = init.headers as Record<string, string>
    expect(headers['Content-Type']).toBeUndefined()
  })

  it('clears the session when an authenticated request returns 401', async () => {
    startSession('session-token', { name: 'Studio Owner', role: 'Owner' })
    fetchMock.mockResolvedValue(jsonResponse({ error: 'invalid token' }, 401))
    await expect(api.listAccounts()).rejects.toThrow('invalid token')
    expect(token.value).toBeNull()
    expect(currentUser.value).toBeNull()
  })

  it('handles empty (204) responses', async () => {
    fetchMock.mockResolvedValue(new Response(null, { status: 204 }))
    await expect(api.deleteAccount('x@y.co')).resolves.toBeUndefined()
  })

  it('fetches OAuth pick candidates with an encoded pick id', async () => {
    fetchMock.mockResolvedValue(jsonResponse({ platform: 'meta', accounts: [] }))
    await api.oauthPending('meta', 'pick a/b')
    const [url] = fetchMock.mock.calls[0] as [string, RequestInit]
    expect(url).toContain('/api/oauth/meta/pending?pick=pick%20a%2Fb')
  })

  it('posts the OAuth page choice with pick + external_id', async () => {
    fetchMock.mockResolvedValue(jsonResponse({ platform: 'meta', connection: null }))
    await api.chooseOAuthPage('meta', 'pick-1', '999')
    const [url, init] = fetchMock.mock.calls[0] as [string, RequestInit]
    expect(url).toContain('/api/oauth/meta/choose')
    expect(init.method).toBe('POST')
    expect(JSON.parse(init.body as string)).toEqual({ pick: 'pick-1', external_id: '999' })
  })

  it('sends the active workspace on every request', async () => {
    fetchMock.mockImplementation(() => Promise.resolve(jsonResponse([])))
    await api.listPlatforms()
    let [, init] = fetchMock.mock.calls[0] as [string, RequestInit]
    expect((init.headers as Record<string, string>)['X-Workspace-Id']).toBeUndefined()

    setActiveWorkspace('ws-client-b')
    await api.listPlatforms()
    ;[, init] = fetchMock.mock.calls[1] as [string, RequestInit]
    expect((init.headers as Record<string, string>)['X-Workspace-Id']).toBe('ws-client-b')
    setActiveWorkspace(null)
  })

  it('creates, renames, and deletes workspaces over the API', async () => {
    fetchMock.mockImplementation(() =>
      Promise.resolve(jsonResponse({ id: 'ws-1', name: 'Client B', created: '', connected: 0, total: 4 })),
    )
    await api.createWorkspace('Client B')
    let [url, init] = fetchMock.mock.calls[0] as [string, RequestInit]
    expect(url).toContain('/api/workspaces')
    expect(JSON.parse(init.body as string)).toEqual({ name: 'Client B' })

    await api.renameWorkspace('ws-1', 'Client B2')
    ;[url, init] = fetchMock.mock.calls[1] as [string, RequestInit]
    expect(url).toContain('/api/workspaces/ws-1')
    expect(init.method).toBe('PATCH')

    fetchMock.mockResolvedValue(jsonResponse({ deleted: 'ws-1', fallback: 'ws-default' }))
    await api.deleteWorkspace('ws-1')
    ;[url, init] = fetchMock.mock.calls[2] as [string, RequestInit]
    expect(url).toContain('/api/workspaces/ws-1')
    expect(init.method).toBe('DELETE')
  })
})
