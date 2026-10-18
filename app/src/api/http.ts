// HTTP client for the Rust backend (`server/`).
// Implements the same `Api` contract as `src/mock/api.ts`, so the app can switch
// at runtime via VITE_API_URL (see src/api/index.ts).
import { z } from 'zod'
import type {
  Post, SetupConfig, Idea, HashtagGroup, Metric, Txn, Brand,
  PlatformConnection, PlatformId, User, WorkspaceSummary, WorkspaceMembers,
  Content, ContentRevision, ContentSummary, PublicContent, FontAsset, Campaign, LiveData, AdsView,
} from '@/mock/db'
import type { BrandImageKind } from '@/core/images'
import type {
  Api, OptionList, AuthResult, ImportResult, OauthStatus, OauthStart, OauthPending, AccountInfo,
  ContentDraft, ContentPatchInput, CreatedAccount,
} from './contract'
import { activeWorkspaceId } from '@/core/workspace'
import {
  ApiError,
  BASE,
  CREDENTIALS,
  customFetch,
  normalizeBase,
  parseApiError,
  refreshSession,
  validatePayload,
} from './client'
import {
  BrandSchema,
  CampaignSchema,
  ContentSchema,
  ContentSummarySchema,
  FontAssetSchema,
  HashtagGroupSchema,
  IdeaSchema,
  MetricSchema,
  PlatformConnectionSchema,
  PostSchema,
  SetupConfigSchema,
  TxnSchema,
} from './schemas'

export { ApiError, BASE, customFetch, normalizeBase, validatePayload }

/** True when an error means the session is invalid/forbidden. */
export function isAuthError(error: unknown): boolean {
  if (error instanceof ApiError) return error.status === 401 || error.status === 403
  const message = error instanceof Error ? error.message : String(error)
  return /401|403|invalid session|invalid token|unauthorized/i.test(message)
}

const DEFAULT_TIMEOUT_MS = 15_000

interface RequestOptions<T = unknown> {
  /** Keep the session even on 401 (login, wrong old password). */
  keepSessionOn401?: boolean
  /** Internal: a request already replayed after a silent refresh never loops. */
  retried?: boolean
  /** Optional Zod schema returning T directly. */
  schema?: z.ZodType<T>
  /** Optional Zod runtime validator when T uses narrower client union types. */
  validate?: (data: unknown) => unknown
}

async function req<T>(path: string, init: RequestInit = {}, opts: RequestOptions<T> = {}): Promise<T> {
  const { headers, ...rest } = init
  const contentType: Record<string, string> = rest.body !== undefined ? { 'Content-Type': 'application/json' } : {}
  const workspace: Record<string, string> = activeWorkspaceId.value
    ? { 'X-Workspace-Id': activeWorkspaceId.value }
    : {}

  let res: Response
  try {
    res = await fetch(`${BASE}${path}`, {
      ...rest,
      credentials: CREDENTIALS,
      signal: rest.signal ?? AbortSignal.timeout(DEFAULT_TIMEOUT_MS),
      headers: { ...contentType, ...workspace, ...(headers as Record<string, string> | undefined) },
    })
  } catch (error) {
    const name = error instanceof DOMException ? error.name : ''
    if (name === 'TimeoutError' || name === 'AbortError') {
      throw new ApiError(0, 'The server took too long to respond. Please try again.', 'TIMEOUT')
    }
    throw new ApiError(
      0,
      'Network error — check your connection and that the API is reachable.',
      'NETWORK_ERROR',
    )
  }

  if (!res.ok) {
    const apiError = await parseApiError(res)
    const canRefresh =
      res.status === 401 &&
      !opts.keepSessionOn401 &&
      !opts.retried &&
      path !== '/api/auth/refresh'
    if (canRefresh) {
      try {
        await refreshSession()
      } catch {
        // Refresh failed (no live session) — surface the original error; the
        // queue was already cleared and the UI session dropped.
        throw apiError
      }
      return req<T>(path, init, { ...opts, retried: true })
    }
    throw apiError
  }

  if (res.status === 204) return undefined as T
  const text = await res.text()
  if (!text) return undefined as T

  let parsed: unknown
  try {
    parsed = JSON.parse(text)
  } catch {
    throw new ApiError(res.status, 'Invalid JSON response from server', 'INVALID_JSON')
  }

  if (opts.schema) {
    return opts.schema.parse(parsed)
  }
  if (opts.validate) {
    opts.validate(parsed)
  }
  return parsed as T
}

const send = <T>(
  path: string,
  method: string,
  body?: unknown,
  opts: RequestOptions<T> = {},
): Promise<T> =>
  req<T>(path, { method, body: body === undefined ? undefined : JSON.stringify(body) }, opts)

export const listPosts = (month?: number, year?: number): Promise<Post[]> => {
  const q = [
    month === undefined ? '' : `month=${month}`,
    year === undefined ? '' : `year=${year}`,
  ].filter(Boolean).join('&')
  return req(q ? `/api/posts?${q}` : '/api/posts', {}, {
    validate: (d) => z.array(PostSchema).parse(d),
  })
}

export const getSetup = (): Promise<SetupConfig> =>
  req('/api/setup', {}, { validate: (d) => SetupConfigSchema.parse(d) })

export const saveSetup = (patch: Partial<SetupConfig>): Promise<SetupConfig> =>
  send('/api/setup', 'PATCH', patch, { validate: (d) => SetupConfigSchema.parse(d) })

export const addOption = (list: OptionList, item: string): Promise<SetupConfig> =>
  send('/api/setup/options', 'POST', { list, item }, { validate: (d) => SetupConfigSchema.parse(d) })

export const addUser = (name: string, role: string): Promise<SetupConfig> =>
  send('/api/setup/users', 'POST', { name, role }, { validate: (d) => SetupConfigSchema.parse(d) })

export const removeUser = (name: string): Promise<SetupConfig> =>
  send(`/api/setup/users/${encodeURIComponent(name)}`, 'DELETE', undefined, {
    validate: (d) => SetupConfigSchema.parse(d),
  })

export const addRole = (name: string, permissions: string[] = []): Promise<SetupConfig> =>
  send('/api/setup/roles', 'POST', { name, permissions }, {
    validate: (d) => SetupConfigSchema.parse(d),
  })

export const removeRole = (name: string): Promise<SetupConfig> =>
  send(`/api/setup/roles/${encodeURIComponent(name)}`, 'DELETE', undefined, {
    validate: (d) => SetupConfigSchema.parse(d),
  })

export const register = (v: { name: string; email: string; password: string }): Promise<AuthResult> =>
  send('/api/auth/register', 'POST', v, { keepSessionOn401: true })

export const login = (email: string, password: string): Promise<AuthResult> =>
  send('/api/auth/login', 'POST', { email, password }, { keepSessionOn401: true })

export const changePassword = (oldPassword: string, newPassword: string): Promise<{ ok: boolean }> =>
  send('/api/auth/change-password', 'POST', { oldPassword, newPassword }, { keepSessionOn401: true })

export const updateProfile = (name: string): Promise<{ user: User }> =>
  send('/api/auth/profile', 'PATCH', { name })

export const setPlan = (plan: string): Promise<{ user: User }> =>
  send('/api/auth/plan', 'POST', { plan })

// Cookie-authenticated: the token parameter exists only for the mock API.
export const me = (): Promise<{ user: User }> => req('/api/auth/me')

export const logout = (): Promise<{ ok: boolean }> =>
  send('/api/auth/logout', 'POST', undefined, { keepSessionOn401: true })

export const listAccounts = (): Promise<AccountInfo[]> => req('/api/auth/accounts')

export const deleteAccount = (email: string): Promise<{ ok: boolean }> =>
  send(`/api/auth/accounts/${encodeURIComponent(email)}`, 'DELETE')

export const logoutAll = (): Promise<{ ok: boolean }> =>
  send('/api/auth/logout-all', 'POST', undefined, { keepSessionOn401: true })

export const lockPost = (id: string, user: string): Promise<Post> =>
  send(`/api/posts/${id}/lock`, 'POST', { user })

export const unlockPost = (id: string, user: string): Promise<Post> =>
  send(`/api/posts/${id}/unlock`, 'POST', { user })

export const updatePost = (id: string, patch: Partial<Post> & { user?: string }): Promise<Post> =>
  send(`/api/posts/${id}`, 'PATCH', patch)

export const deletePost = (id: string): Promise<{ ok: boolean }> =>
  send(`/api/posts/${encodeURIComponent(id)}`, 'DELETE')

export const addPost = (draft: Omit<Post, 'id'>): Promise<Post> =>
  send('/api/posts', 'POST', draft)

export const listIdeas = (): Promise<Idea[]> =>
  req('/api/ideas', {}, { schema: z.array(IdeaSchema) })

export const addIdea = (draft: Omit<Idea, 'id'>): Promise<Idea> =>
  send('/api/ideas', 'POST', draft, { schema: IdeaSchema })

export const toggleIdea = (id: string): Promise<Idea> =>
  send(`/api/ideas/${id}/toggle`, 'POST', undefined, { schema: IdeaSchema })

export const promoteIdea = (id: string, month: number, year?: number): Promise<Post> =>
  send(`/api/ideas/${id}/promote?month=${month}${year === undefined ? '' : `&year=${year}`}`, 'POST', undefined, {
    validate: (d) => PostSchema.parse(d),
  })

export const listTags = (): Promise<HashtagGroup[]> =>
  req('/api/tags', {}, { schema: z.array(HashtagGroupSchema) })

export const addTag = (groupId: string, tag: string): Promise<HashtagGroup> =>
  send(`/api/tags/${groupId}`, 'POST', { tag }, { schema: HashtagGroupSchema })

export const listMetrics = (): Promise<Metric[]> =>
  req('/api/metrics', {}, { schema: z.array(MetricSchema) })

export const importMetrics = (platform: string, month: number, year?: number): Promise<ImportResult> =>
  send('/api/metrics/import', 'POST', { platform, month, ...(year === undefined ? {} : { year }) })

export const listTxns = (): Promise<Txn[]> =>
  req('/api/txns', {}, { validate: (d) => z.array(TxnSchema).parse(d) })

export const addTxn = (draft: Omit<Txn, 'id'>): Promise<Txn> =>
  send('/api/txns', 'POST', draft, { validate: (d) => TxnSchema.parse(d) })

export const getBrand = (): Promise<Brand> =>
  req('/api/brand', {}, { schema: BrandSchema })

export const listCampaigns = (): Promise<Campaign[]> =>
  req('/api/campaigns', {}, { validate: (d) => z.array(CampaignSchema).parse(d) })

export const getCampaign = (id: string): Promise<Campaign> =>
  req(`/api/campaigns/${encodeURIComponent(id)}`, {}, { validate: (d) => CampaignSchema.parse(d) })

export const createCampaign = (patch: Partial<Campaign>): Promise<Campaign> =>
  send('/api/campaigns', 'POST', patch, { validate: (d) => CampaignSchema.parse(d) })

export const updateCampaign = (id: string, patch: Partial<Campaign>): Promise<Campaign> =>
  send(`/api/campaigns/${encodeURIComponent(id)}`, 'PATCH', patch, { validate: (d) => CampaignSchema.parse(d) })

export const deleteCampaign = (id: string): Promise<{ ok: boolean }> =>
  send(`/api/campaigns/${encodeURIComponent(id)}`, 'DELETE')

export const listFonts = (): Promise<FontAsset[]> =>
  req('/api/brand/fonts', {}, { schema: z.array(FontAssetSchema) })

export const uploadFont = (name: string, data: string): Promise<FontAsset[]> =>
  send('/api/brand/fonts', 'POST', { name, data }, { schema: z.array(FontAssetSchema) })

export const deleteFont = (name: string): Promise<FontAsset[]> =>
  send(`/api/brand/fonts/${encodeURIComponent(name)}`, 'DELETE', undefined, { schema: z.array(FontAssetSchema) })

export const uploadBrandImage = (
  name: string,
  data: string,
  kind: BrandImageKind,
): Promise<{ url: string; width: number; height: number }> =>
  send('/api/brand/images', 'POST', { name, data, kind })

export const deleteBrandImage = (name: string): Promise<{ ok: boolean }> =>
  send(`/api/brand/images/${encodeURIComponent(name)}`, 'DELETE')

export const saveBrand = (patch: Partial<Brand>): Promise<Brand> =>
  send('/api/brand', 'PATCH', patch, { schema: BrandSchema })

export const listPlatforms = (): Promise<PlatformConnection[]> =>
  req('/api/platforms', {}, { validate: (d) => z.array(PlatformConnectionSchema).parse(d) })

export const getLive = (): Promise<LiveData> => req('/api/live')

export const getAds = (): Promise<AdsView> => req('/api/ads')

export const syncAds = (): Promise<AdsView> => send('/api/ads/sync', 'POST', {})

export const setAdsManage = (enabled: boolean): Promise<{ canManage: boolean }> =>
  send('/api/ads/manage', 'POST', { enabled })

export const setAdCampaignStatus = (
  id: string,
  status: 'ACTIVE' | 'PAUSED' | 'ARCHIVED',
): Promise<{ ok: boolean }> =>
  send(`/api/ads/campaigns/${encodeURIComponent(id)}/status`, 'POST', { status })

export const setAdCampaignBudget = (
  id: string,
  budget: { dailyBudget?: number; lifetimeBudget?: number },
): Promise<{ ok: boolean }> =>
  send(`/api/ads/campaigns/${encodeURIComponent(id)}/budget`, 'POST', budget)

export const duplicateAdCampaign = (id: string): Promise<{ campaignId: string }> =>
  send(`/api/ads/campaigns/${encodeURIComponent(id)}/duplicate`, 'POST', {})

export const createAdBoost = (input: {
  name: string
  objective: string
  dailyBudget: number
  days: number
  countries: string[]
  storyId: string
}): Promise<{ campaignId: string }> => send('/api/ads/boost', 'POST', input)

export const listWorkspaces = (): Promise<WorkspaceSummary[]> => req('/api/workspaces')

export const createWorkspace = (name: string): Promise<WorkspaceSummary> =>
  send('/api/workspaces', 'POST', { name })

export const renameWorkspace = (id: string, name: string): Promise<WorkspaceSummary> =>
  send(`/api/workspaces/${encodeURIComponent(id)}`, 'PATCH', { name })

export const deleteWorkspace = (id: string): Promise<{ deleted: string; fallback: string }> =>
  send(`/api/workspaces/${encodeURIComponent(id)}`, 'DELETE')

export const listWorkspaceMembers = (id: string): Promise<WorkspaceMembers> =>
  req(`/api/workspaces/${encodeURIComponent(id)}/members`)

export const addWorkspaceMember = (id: string, email: string): Promise<WorkspaceMembers> =>
  send(`/api/workspaces/${encodeURIComponent(id)}/members`, 'POST', { email })

export const removeWorkspaceMember = (id: string, email: string): Promise<WorkspaceMembers> =>
  send(`/api/workspaces/${encodeURIComponent(id)}/members/${encodeURIComponent(email)}`, 'DELETE')

export const oauthStatus = (platform: PlatformId): Promise<OauthStatus> =>
  req(`/api/oauth/${platform}`)

export const startOauth = (platform: PlatformId): Promise<OauthStart> =>
  req(`/api/oauth/${platform}/start`)

export const oauthPending = (platform: PlatformId, pick: string): Promise<OauthPending> =>
  req(`/api/oauth/${platform}/pending?pick=${encodeURIComponent(pick)}`)

export const chooseOAuthPage = (platform: PlatformId, pick: string, external_id: string): Promise<PlatformConnection> =>
  send(`/api/oauth/${platform}/choose`, 'POST', { pick, external_id })

export const connectPlatform = (id: PlatformId, handle?: string): Promise<PlatformConnection> =>
  send(`/api/platforms/${id}/connect`, 'POST', { handle: handle ?? '' })

export const disconnectPlatform = (id: PlatformId): Promise<PlatformConnection> =>
  send(`/api/platforms/${id}/disconnect`, 'POST')

export const syncPlatform = (id: PlatformId): Promise<PlatformConnection> =>
  send(`/api/platforms/${id}/sync`, 'POST')

export const createAccount = (v: {
  name: string
  email: string
  role: string
  password?: string
}): Promise<CreatedAccount> => send('/api/auth/accounts', 'POST', v)

export const listContent = (params?: { status?: string; q?: string }): Promise<ContentSummary[]> => {
  const query = new URLSearchParams()
  if (params?.status) query.set('status', params.status)
  if (params?.q) query.set('q', params.q)
  const suffix = query.toString()
  return req(`/api/content${suffix ? `?${suffix}` : ''}`, {}, {
    validate: (d) => z.array(ContentSummarySchema).parse(d),
  })
}

export const getContent = (id: string): Promise<Content> =>
  req(`/api/content/${encodeURIComponent(id)}`, {}, {
    validate: (d) => ContentSchema.parse(d),
  })

export const createContent = (draft: ContentDraft): Promise<Content> =>
  send('/api/content', 'POST', draft)

export const updateContent = (id: string, patch: ContentPatchInput): Promise<Content> =>
  send(`/api/content/${encodeURIComponent(id)}`, 'PATCH', patch)

export const publishContent = (id: string, version: number): Promise<Content> =>
  send(`/api/content/${encodeURIComponent(id)}/publish`, 'POST', { version })

export const unpublishContent = (id: string, version: number): Promise<Content> =>
  send(`/api/content/${encodeURIComponent(id)}/unpublish`, 'POST', { version })

export const scheduleContent = (id: string, version: number, scheduledFor: string): Promise<Content> =>
  send(`/api/content/${encodeURIComponent(id)}/schedule`, 'POST', { version, scheduledFor })

export const deleteContent = (id: string): Promise<{ ok: boolean }> =>
  send(`/api/content/${encodeURIComponent(id)}`, 'DELETE')

export const duplicateContent = (id: string): Promise<Content> =>
  send(`/api/content/${encodeURIComponent(id)}/duplicate`, 'POST')

export const listContentRevisions = (id: string): Promise<ContentRevision[]> =>
  req(`/api/content/${encodeURIComponent(id)}/revisions`)

export const getContentRevision = (id: string, revision: number): Promise<ContentRevision> =>
  req(`/api/content/${encodeURIComponent(id)}/revisions/${revision}`)

export const restoreContentRevision = (id: string, revision: number, version: number): Promise<Content> =>
  send(`/api/content/${encodeURIComponent(id)}/revisions/${revision}/restore`, 'POST', { version })

export const listPublicContent = (): Promise<ContentSummary[]> => req('/api/public/content')

export const getPublicContent = (slug: string): Promise<PublicContent> =>
  req(`/api/public/content/${encodeURIComponent(slug)}`)

// Compile-time proof that this module implements the full contract.
const _api: Api = {
  listPosts, getSetup, saveSetup, addOption, addUser, removeUser, addRole, removeRole,
  register, login, changePassword, updateProfile, setPlan, me, logout, listAccounts, deleteAccount, logoutAll,
  lockPost, unlockPost, updatePost, addPost, deletePost,
  listIdeas, addIdea, toggleIdea, promoteIdea,
  listTags, addTag,
  listMetrics, importMetrics,
  listTxns, addTxn,
  getBrand, saveBrand, listFonts, uploadFont, deleteFont, uploadBrandImage, deleteBrandImage,
  listCampaigns, getCampaign, createCampaign, updateCampaign, deleteCampaign,
  listPlatforms, getLive, getAds, syncAds, setAdsManage, setAdCampaignStatus, setAdCampaignBudget,
  duplicateAdCampaign, createAdBoost,
  oauthStatus, startOauth, oauthPending, chooseOAuthPage, connectPlatform, disconnectPlatform, syncPlatform,
  listWorkspaces, createWorkspace, renameWorkspace, deleteWorkspace,
  listWorkspaceMembers, addWorkspaceMember, removeWorkspaceMember,
  createAccount,
  listContent, getContent, createContent, updateContent, publishContent, unpublishContent,
  scheduleContent, deleteContent, duplicateContent, listContentRevisions, getContentRevision,
  restoreContentRevision, listPublicContent, getPublicContent,
}
void _api
