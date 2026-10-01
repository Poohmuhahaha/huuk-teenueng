// Mock async API over the in-memory workbook store.
// Every fetcher returns a copy (like a real API); swap with HTTP later.
import {
  posts, setup, ideas, hashtagGroups, metrics, txns, brand, uid, connections, workspaces, PERMISSIONS,
  content, contentSummary, campaigns, live, ads, adsFlags,
} from './db'
import type {
  Post, SetupConfig, Idea, HashtagGroup, Metric, Txn, Brand, Status,
  PlatformConnection, PlatformId, User, Permission,
  Content, ContentRevision, ContentSummary, PublicContent, FontAsset, Campaign, WorkspaceSummary, LiveData,
  MockWorkspace, WorkspaceMembers,
  AdCampaign, AdsView,
} from './db'
import type {
  Api, OptionList, AuthResult, ImportResult, OauthMode, OauthStatus, OauthStart, OauthPending, AccountInfo,
  ContentDraft, ContentPatchInput, CreatedAccount,
} from '@/api/contract'
import { validProfileHandle } from '@/core/platforms'
import {
  base64ToBytes, checkImageBounds, imageDimensions, imageMime,
} from '@/core/images'
import type { BrandImageKind } from '@/core/images'

export type { OptionList, AuthResult, ImportResult, OauthMode, OauthStatus, OauthStart, OauthPending, AccountInfo }

const wait = (ms = 120): Promise<void> => new Promise((r) => setTimeout(r, ms))
const clone = <T>(v: T): T => JSON.parse(JSON.stringify(v)) as T

const MAX_TEXT = 200
const MAX_LONG_TEXT = 5000
const MAX_LIST_ITEMS = 100
const MAX_LIST_ITEM = 200

function checkLen(field: string, value: string, max: number): void {
  if (value.length > max) throw new Error(`${field} must be at most ${max} characters`)
}

function checkMonth(month: number): void {
  if (!Number.isInteger(month) || month < 1 || month > 12) throw new Error('month must be between 1 and 12')
}

function checkList(field: string, values: string[]): void {
  if (values.length > MAX_LIST_ITEMS) throw new Error(`${field} must have at most ${MAX_LIST_ITEMS} items`)
  for (const value of values) checkLen(field, value, MAX_LIST_ITEM)
}

function checkPost(post: Post): void {
  checkMonth(post.month)
  if (!post.topic.trim()) throw new Error('topic is required')
  checkLen('topic', post.topic, MAX_TEXT)
  checkLen('pillar', post.pillar, MAX_TEXT)
  checkLen('format', post.format, MAX_TEXT)
  checkLen('goal', post.goal, MAX_TEXT)
  checkLen('hook', post.hook, MAX_LONG_TEXT)
  checkLen('caption', post.caption, MAX_LONG_TEXT)
  checkLen('cta', post.cta, MAX_TEXT)
  checkLen('note', post.note, MAX_LONG_TEXT)
  checkList('hashtags', post.hashtags)
  checkList('platforms', post.platforms)
  if (setup.statuses.length && post.status && !setup.statuses.includes(post.status as Status)) {
    throw new Error(`unknown status '${post.status}'`)
  }
  for (const p of post.platforms) {
    if (setup.platforms.length && !setup.platforms.includes(p)) throw new Error(`unknown platform '${p}'`)
  }
}

export async function listPosts(month?: number): Promise<Post[]> {
  await wait()
  if (month !== undefined) checkMonth(month)
  return clone(month === undefined ? posts : posts.filter((p) => p.month === month))
}

export async function getSetup(): Promise<SetupConfig> {
  await wait(60)
  return clone(setup)
}

export async function saveSetup(patch: Partial<SetupConfig>): Promise<SetupConfig> {
  requirePerm('setup.write')
  await wait()
  if (patch.workspaceName !== undefined) {
    const name = patch.workspaceName.trim()
    if (!name) throw new Error('workspace name is required')
    checkLen('workspaceName', name, 80)
    patch.workspaceName = name
  }
  if (patch.roles) {
    const roles = patch.roles
    if (roles.length === 0) throw new Error('at least one role is required')
    if (roles.some((r) => !r.name.trim())) throw new Error('role name is required')
    const seen = new Set<string>()
    for (const r of roles) {
      if (seen.has(r.name)) throw new Error(`duplicate role name: ${r.name}`)
      seen.add(r.name)
    }
    for (const r of roles) {
      for (const perm of r.permissions) {
        if (!(PERMISSIONS as readonly string[]).includes(perm)) throw new Error(`unknown permission '${perm}'`)
      }
    }
    for (const u of setup.users) {
      if (!roles.some((r) => r.name === u.role)) throw new Error(`role is in use: ${u.role}`)
    }
  }
  Object.assign(setup, patch)
  return clone(setup)
}

export async function addOption(list: OptionList, item: string): Promise<SetupConfig> {
  requirePerm('setup.write')
  await wait()
  const value = item.trim()
  checkLen('option', value, MAX_LIST_ITEM)
  const target = setup[list] as unknown as string[]
  if (value && !target.includes(value)) target.push(value)
  return clone(setup)
}

// ---- users (US-011) ----

export async function addUser(name: string, role: string): Promise<SetupConfig> {
  requirePerm('users.manage')
  await wait()
  const n = name.trim()
  if (!n) throw new Error('name is required')
  checkLen('name', n, MAX_TEXT)
  if (setup.users.some((u) => u.name.toLowerCase() === n.toLowerCase())) throw new Error(`user '${n}' already exists`)
  const r = role.trim() || 'Editor'
  if (!setup.roles.some((role) => role.name === r)) throw new Error(`unknown role '${r}'`)
  setup.users.push({ name: n, role: r })
  return clone(setup)
}

export async function removeUser(name: string): Promise<SetupConfig> {
  requirePerm('users.manage')
  await wait()
  if (!setup.users.some((u) => u.name === name)) throw new Error('user not found')
  if (setup.users.length <= 1) throw new Error('cannot remove the last user')
  setup.users = setup.users.filter((u) => u.name !== name)
  return clone(setup)
}

// ---- roles (permission matrix) ----

export async function addRole(name: string, permissions: string[] = []): Promise<SetupConfig> {
  requirePerm('users.manage')
  await wait()
  const n = name.trim()
  if (!n) throw new Error('role name is required')
  if (setup.roles.some((r) => r.name === n)) throw new Error(`role '${n}' already exists`)
  setup.roles.push({ name: n, permissions: [...permissions] })
  return clone(setup)
}

export async function removeRole(name: string): Promise<SetupConfig> {
  requirePerm('users.manage')
  await wait()
  if (!setup.roles.some((r) => r.name === name)) throw new Error(`role '${name}' not found`)
  if (setup.roles.length <= 1) throw new Error('cannot remove the last role')
  if (setup.users.some((u) => u.role === name)) throw new Error(`role '${name}' is in use`)
  setup.roles = setup.roles.filter((r) => r.name !== name)
  return clone(setup)
}

// ---- auth (US-012) — email + password accounts with expiring sessions ----

interface Account {
  name: string
  email: string
  password: string
  role: string
  plan: string
}

interface Session {
  email: string
  expiresAt: number
}

const SESSION_TTL = 7 * 24 * 60 * 60 * 1000

// Seeded demo accounts. Plain-text passwords are fine in the wireframe mock.
const accounts: Account[] = [
  { name: 'Studio Owner', email: 'owner@studio.local', password: 'demo1234', role: 'Owner', plan: 'free' },
  { name: 'Editor Earn', email: 'editor@studio.local', password: 'demo1234', role: 'Editor', plan: 'free' },
  { name: 'Studio Client', email: 'client@studio.local', password: 'demo1234', role: 'Client', plan: 'free' },
]

const sessions = new Map<string, Session>()
let activeToken: string | null = null

const normEmail = (email: string): string => email.trim().toLowerCase()

// Role follows the Setup directory; an account whose directory entry was
// removed falls back to Viewer (mirrors the server's `user_view`).
function accountUser(a: Account): User {
  const dir = setup.users.find((u) => u.name === a.name)
  return { name: a.name, role: dir ? dir.role : 'Viewer', plan: a.plan }
}

function sessionUser(t: string | null): User | null {
  if (!t) return null
  const s = sessions.get(t)
  if (!s) return null
  if (s.expiresAt <= Date.now()) {
    sessions.delete(t)
    return null
  }
  const account = accounts.find((a) => a.email === s.email)
  return account ? accountUser(account) : null
}

/** The account behind the current session (mock equivalent of the server session). */
function activeAccount(): Account | undefined {
  const s = activeToken ? sessions.get(activeToken) : undefined
  return s ? accounts.find((a) => a.email === s.email) : undefined
}

function startSession(email: string): string {
  const t = uid('tok')
  sessions.set(t, { email, expiresAt: Date.now() + SESSION_TTL })
  activeToken = t
  return t
}

// Mirrors the server guard: demo mode is open; otherwise a session whose role
// holds the permission is required.
function requirePerm(perm: Permission): void {
  if (!setup.authRequired) return
  const user = sessionUser(activeToken)
  if (!user) throw new Error('401 unauthorized')
  const role = setup.roles.find((r) => r.name === user.role)
  if (!role || !role.permissions.includes(perm)) throw new Error(`403 forbidden: ${perm}`)
}

export async function register(v: { name: string; email: string; password: string }): Promise<AuthResult> {
  await wait(200)
  if (!setup.allowRegistration) throw new Error('registration is disabled')
  const name = v.name.trim()
  if (!name) throw new Error('name is required')
  const email = normEmail(v.email)
  if (!email.includes('@')) throw new Error('a valid email is required')
  if (v.password.length < 12) throw new Error('password must be at least 12 characters')
  if (email.length > 254) throw new Error('a valid email is required')
  if (accounts.some((a) => a.email === email)) throw new Error('email already registered')
  if (accounts.some((a) => a.name.toLowerCase() === name.toLowerCase())) throw new Error('name already taken')
  const dir = setup.users.find((u) => u.name.toLowerCase() === name.toLowerCase())
  // Self-serve signup: the workspace creator owns it (mirrors the server).
  const selfServeRole = setup.roles.some((r) => r.name === 'Owner') ? 'Owner' : 'Editor'
  const account: Account = { name, email, password: v.password, role: dir?.role ?? selfServeRole, plan: 'free' }
  accounts.push(account)
  if (!setup.users.some((u) => u.name.toLowerCase() === name.toLowerCase())) {
    setup.users.push({ name, role: selfServeRole })
  }
  return { token: startSession(email), user: clone(accountUser(account)) }
}

export async function login(email: string, password: string): Promise<AuthResult> {
  await wait(200)
  const account = accounts.find((a) => a.email === normEmail(email))
  if (!account || account.password !== password) throw new Error('invalid email or password')
  return { token: startSession(account.email), user: clone(accountUser(account)) }
}

export async function changePassword(oldPassword: string, newPassword: string): Promise<{ ok: boolean }> {
  await wait(120)
  const session = activeToken ? sessions.get(activeToken) : undefined
  const account = session ? accounts.find((a) => a.email === session.email) : undefined
  if (!account) throw new Error('401 unauthorized')
  if (account.password !== oldPassword) throw new Error('invalid password')
  if (newPassword.length < 12) throw new Error('password must be at least 12 characters')
  account.password = newPassword
  for (const [t, s] of sessions) if (t !== activeToken && s.email === account.email) sessions.delete(t)
  return { ok: true }
}

export async function updateProfile(name: string): Promise<{ user: User }> {
  await wait()
  if (!sessionUser(activeToken)) throw new Error('401 unauthorized')
  const n = name.trim()
  if (!n) throw new Error('name is required')
  const session = activeToken ? sessions.get(activeToken) : undefined
  const account = session ? accounts.find((a) => a.email === session.email) : undefined
  if (!account) throw new Error('401 unauthorized')
  if (accounts.some((a) => a.name === n && a !== account)) throw new Error('name already taken')
  const dir = setup.users.find((u) => u.name === account.name)
  if (dir) dir.name = n
  account.name = n
  return { user: clone(accountUser(account)) }
}

export async function setPlan(plan: string): Promise<{ user: User }> {
  const account = activeAccount()
  if (!account) throw new Error('401 unauthorized')
  const next = plan.trim().toLowerCase()
  if (!['free', 'pro', 'business'].includes(next)) throw new Error('unknown plan')
  account.plan = next
  return { user: clone(accountUser(account)) }
}

export async function me(token: string): Promise<{ user: User }> {
  await wait(60)
  const user = sessionUser(token)
  if (!user) throw new Error('invalid session')
  activeToken = token
  return { user: clone(user) }
}

export async function logout(token: string): Promise<{ ok: boolean }> {
  await wait(60)
  if (!sessions.has(token)) throw new Error('401 unauthorized')
  sessions.delete(token)
  if (activeToken === token) activeToken = null
  return { ok: true }
}

export async function listAccounts(): Promise<AccountInfo[]> {
  requirePerm('users.manage')
  await wait()
  const now = Date.now()
  return accounts
    .map((a) => ({
      name: a.name,
      email: a.email,
      sessions: [...sessions.values()].filter((s) => s.email === a.email && s.expiresAt > now).length,
    }))
    .sort((a, b) => a.name.localeCompare(b.name))
}

export async function deleteAccount(email: string): Promise<{ ok: boolean }> {
  requirePerm('users.manage')
  await wait()
  const target = normEmail(email)
  const account = accounts.find((a) => a.email === target)
  if (!account) throw new Error('account not found')
  accounts.splice(accounts.indexOf(account), 1)
  for (const [t, s] of sessions) {
    if (s.email === account.email) {
      sessions.delete(t)
      if (activeToken === t) activeToken = null
    }
  }
  // Roles live on the directory entry — drop it too (keep at least one).
  if (setup.users.length > 1) {
    const idx = setup.users.findIndex((u) => u.name.toLowerCase() === account.name.toLowerCase())
    if (idx >= 0) setup.users.splice(idx, 1)
  }
  return { ok: true }
}

export async function logoutAll(sessionToken?: string): Promise<{ ok: boolean }> {
  await wait(60)
  const key = sessionToken ?? activeToken
  const session = key ? sessions.get(key) : undefined
  if (!session) throw new Error('401 unauthorized')
  for (const [t, s] of sessions) if (s.email === session.email) sessions.delete(t)
  if (activeToken) activeToken = null
  return { ok: true }
}

// ---- post locking (US-014) ----

export async function lockPost(id: string, user: string): Promise<Post> {
  requirePerm('posts.lock')
  await wait()
  const actor = setup.authRequired ? sessionUser(activeToken)?.name : user
  const p = posts.find((x) => x.id === id)
  if (!p) throw new Error('post not found')
  if (p.lockedBy && p.lockedBy !== actor) throw new Error(`post is locked by ${p.lockedBy}`)
  p.lockedBy = actor
  return clone(p)
}

export async function unlockPost(id: string, user: string): Promise<Post> {
  requirePerm('posts.lock')
  await wait()
  const actor = setup.authRequired ? sessionUser(activeToken)?.name : user
  const p = posts.find((x) => x.id === id)
  if (!p) throw new Error('post not found')
  if (p.lockedBy && p.lockedBy !== actor) throw new Error(`post is locked by ${p.lockedBy}`)
  p.lockedBy = null
  return clone(p)
}

export async function updatePost(id: string, patch: Partial<Post> & { user?: string }): Promise<Post> {
  requirePerm('posts.write')
  await wait()
  const actor = setup.authRequired ? sessionUser(activeToken)?.name : patch.user
  const p = posts.find((x) => x.id === id)
  if (!p) throw new Error('post not found')
  if (p.lockedBy && p.lockedBy !== actor) throw new Error(`post is locked by ${p.lockedBy}`)
  const { user: _user, ...fields } = patch
  const candidate = { ...p, ...fields } as Post
  checkPost(candidate)
  Object.assign(p, fields)
  return clone(p)
}

export async function addPost(draft: Omit<Post, 'id'>): Promise<Post> {
  requirePerm('posts.write')
  await wait()
  const p: Post = { ...clone(draft), id: uid('p'), lockedBy: null }
  checkPost(p)
  posts.push(p)
  return clone(p)
}

export async function deletePost(id: string): Promise<{ ok: boolean }> {
  requirePerm('posts.write')
  await wait()
  const index = posts.findIndex((p) => p.id === id)
  if (index < 0) throw new Error('post not found')
  const holder = posts[index].lockedBy
  if (holder) {
    // Mirrors the server rule; in the mock the acting user is the owner.
    const actor = setup.owner || ''
    if (holder !== actor) throw new Error(`post is locked by ${holder}`)
  }
  posts.splice(index, 1)
  return { ok: true }
}

export async function listIdeas(): Promise<Idea[]> {
  await wait()
  return clone(ideas)
}

export async function addIdea(draft: Omit<Idea, 'id'>): Promise<Idea> {
  requirePerm('ideas.write')
  await wait()
  const topic = draft.topic.trim()
  if (!topic) throw new Error('topic is required')
  checkLen('topic', topic, MAX_TEXT)
  checkLen('format', draft.format, MAX_TEXT)
  checkLen('idea', draft.idea, MAX_LONG_TEXT)
  checkLen('link', draft.link, 2000)
  const it: Idea = { ...clone({ ...draft, topic }), id: uid('i') }
  ideas.push(it)
  return clone(it)
}

export async function toggleIdea(id: string): Promise<Idea> {
  requirePerm('ideas.write')
  await wait()
  const it = ideas.find((x) => x.id === id)
  if (!it) throw new Error('idea not found')
  it.done = !it.done
  return clone(it)
}

export async function promoteIdea(id: string, month: number): Promise<Post> {
  requirePerm('ideas.write')
  await wait()
  checkMonth(month)
  const it = ideas.find((x) => x.id === id)
  if (!it) throw new Error('idea not found')
  const p: Post = {
    id: uid('p'), month, topic: it.topic, pillar: setup.pillars[0] ?? 'Pillar I',
    format: it.format, goal: setup.goals[0] ?? '', date: null, time: '09:00',
    status: 'Start' as Status, hook: '', caption: it.idea, cta: '',
    hashtagGroup: hashtagGroups[0]?.title ?? '', hashtags: [],
    imageUrl: it.link, note: `promoted from ${it.id}`, done: false, platforms: [],
  }
  posts.push(p)
  it.done = true
  return clone(p)
}

export async function listTags(): Promise<HashtagGroup[]> {
  await wait()
  return clone(hashtagGroups)
}

export async function addTag(groupId: string, tag: string): Promise<HashtagGroup> {
  requirePerm('hashtags.write')
  await wait()
  const value = tag.trim().replace(/^#/, '')
  checkLen('tag', value, MAX_LIST_ITEM)
  const g = hashtagGroups.find((x) => x.id === groupId)
  if (!g) throw new Error('group not found')
  if (value && !g.tags.includes(value)) g.tags.push(value)
  return clone(g)
}

export async function listMetrics(): Promise<Metric[]> {
  await wait(60)
  return clone(metrics)
}

// ---- metrics import (US-008) — mirrors the server's deterministic import ----

function metricHash(postId: string, platform: string): number {
  const s = `${postId}${platform}`
  let h = 0
  for (let i = 0; i < s.length; i++) h = (h + s.charCodeAt(i)) % 100000
  return h
}

export async function importMetrics(platform: string, month: number): Promise<ImportResult> {
  requirePerm('metrics.import')
  await wait(400)
  const p = platform.trim()
  if (!p) throw new Error('platform is required')
  checkMonth(month)
  if (setup.platforms.length && !setup.platforms.includes(p)) throw new Error(`unknown platform '${p}'`)
  const affected: Metric[] = []
  for (const post of posts.filter((x) => x.month === month && x.platforms.includes(p))) {
    const h = metricHash(post.id, p)
    const views = 1000 + (h % 9000)
    const row: Metric = { postId: post.id, platform: p, likes: Math.floor(views / 12), views }
    const existing = metrics.find((m) => m.postId === post.id && m.platform === p)
    if (existing) Object.assign(existing, row)
    else metrics.push(row)
    affected.push(row)
  }
  return { imported: affected.length, metrics: clone(affected) }
}

export async function listTxns(): Promise<Txn[]> {
  await wait()
  return clone(txns)
}

export async function addTxn(draft: Omit<Txn, 'id'>): Promise<Txn> {
  requirePerm('finance.write')
  await wait()
  if (!Number.isFinite(draft.amount) || draft.amount <= 0 || draft.amount > 1e12) {
    throw new Error('amount must be a positive number')
  }
  if (draft.kind !== 'IN' && draft.kind !== 'OUT') throw new Error('kind must be IN or OUT')
  if (!draft.date) throw new Error('date is required')
  checkLen('category', draft.category, MAX_TEXT)
  checkLen('sub', draft.sub, MAX_TEXT)
  const t: Txn = { ...clone(draft), id: uid('t') }
  txns.push(t)
  return clone(t)
}

export async function getBrand(): Promise<Brand> {
  await wait(60)
  return clone(brand)
}

const fonts: FontAsset[] = []

const FONT_EXT: Record<string, string> = {
  woff2: 'woff2', woff: 'woff', ttf: 'truetype', otf: 'opentype',
}

// Brand images live as data URLs in the mock (the server stores real files).
export async function uploadBrandImage(
  name: string,
  data: string,
  kind: BrandImageKind,
): Promise<{ url: string; width: number; height: number }> {
  requirePerm('brand.write')
  await wait(300)
  if (!data) throw new Error('image file is empty')
  const mime = imageMime(name)
  if (!mime) throw new Error('unsupported image (use .png, .jpg, .webp or .gif)')
  const size = imageDimensions(base64ToBytes(data))
  if (!size) throw new Error('could not read the image dimensions')
  const boundsError = checkImageBounds(kind, size.width, size.height)
  if (boundsError) throw new Error(boundsError)
  return {
    url: `data:${mime};base64,${data.replace(/^data:[^,]+,/, '')}`,
    width: size.width,
    height: size.height,
  }
}

export async function deleteBrandImage(_name: string): Promise<{ ok: boolean }> {
  requirePerm('brand.write')
  await wait(120)
  return { ok: true }
}

export async function listFonts(): Promise<FontAsset[]> {
  await wait(40)
  return clone(fonts)
}

export async function uploadFont(name: string, data: string): Promise<FontAsset[]> {
  requirePerm('brand.write')
  await wait(120)
  const ext = name.split('.').pop()?.toLowerCase() ?? ''
  if (!FONT_EXT[ext]) throw new Error('unsupported font file (use .woff2, .woff, .ttf or .otf)')
  let size = 0
  try {
    size = atob(data).length
  } catch {
    throw new Error('invalid font payload')
  }
  if (!size) throw new Error('font file is empty')
  if (size > 2 * 1024 * 1024) throw new Error('font file exceeds 2 MB')
  const stem = name.replace(/\.[^.]+$/, '')
  const family = stem.replace(/[^a-zA-Z0-9 _-]/g, ' ').trim().replace(/\s+/g, ' ') || 'Imported Font'
  const slug = stem.toLowerCase().replace(/[^a-z0-9]+/g, '-').replace(/^-+|-+$/g, '') || 'font'
  let file = `${slug}.${ext}`
  let n = 2
  while (fonts.some((f) => f.name === file)) {
    file = `${slug}-${n}.${ext}`
    n += 1
  }
  fonts.push({ name: file, family, url: `/api/brand/fonts/${file}/file`, size, uploadedAt: Date.now() })
  fonts.sort((a, b) => a.family.toLowerCase().localeCompare(b.family.toLowerCase()))
  return clone(fonts)
}

export async function deleteFont(name: string): Promise<FontAsset[]> {
  requirePerm('brand.write')
  await wait(80)
  const index = fonts.findIndex((f) => f.name === name)
  if (index < 0) throw new Error('font not found')
  fonts.splice(index, 1)
  return clone(fonts)
}

export async function saveBrand(patch: Partial<Brand>): Promise<Brand> {
  requirePerm('brand.write')
  await wait()
  for (const key of ['channel', 'positioning', 'slogan', 'audience', 'voice'] as const) {
    const value = patch[key]
    if (typeof value === 'string') checkLen(key, value, MAX_TEXT)
  }
  for (const key of ['dos', 'donts', 'palette', 'fonts'] as const) {
    const value = patch[key]
    if (Array.isArray(value)) checkList(key, value)
  }
  Object.assign(brand, patch)
  return clone(brand)
}

// ---- marketing campaigns ----

const CAMPAIGN_STATUSES = ['draft', 'active', 'paused', 'completed'] as const
const CAMPAIGN_METRICS = ['views', 'likes', 'reach', 'posts'] as const

function campaignDate(raw: string | null): string | null {
  const value = (raw ?? '').trim()
  if (!value) return null
  if (!/^\d{4}-\d{2}-\d{2}$/.test(value) || Number.isNaN(Date.parse(value))) {
    throw new Error('dates must be YYYY-MM-DD')
  }
  return value
}

function validateCampaign(next: Campaign): void {
  if (!next.name.trim()) throw new Error('campaign name is required')
  checkLen('name', next.name, MAX_TEXT)
  checkLen('objective', next.objective, MAX_TEXT)
  checkLen('owner', next.owner, MAX_TEXT)
  checkLen('notes', next.notes, MAX_LONG_TEXT)
  if (!(CAMPAIGN_STATUSES as readonly string[]).includes(next.status)) {
    throw new Error('status must be draft, active, paused or completed')
  }
  if (!(CAMPAIGN_METRICS as readonly string[]).includes(next.goalMetric)) {
    throw new Error('goalMetric must be views, likes, reach or posts')
  }
  if (!Number.isFinite(next.budget) || next.budget < 0) throw new Error('budget must be a positive number')
  if (!Number.isFinite(next.goalTarget) || next.goalTarget < 0) throw new Error('goalTarget must be a positive number')
  const start = campaignDate(next.startDate)
  const end = campaignDate(next.endDate)
  if (start && end && start > end) throw new Error('endDate must be on or after startDate')
  checkList('platforms', next.platforms)
  checkList('pillars', next.pillars)
  checkList('hashtags', next.hashtags)
  for (const id of next.contentIds) {
    if (!content.some((c) => c.id === id)) throw new Error(`unknown content item: ${id}`)
  }
}

function mergeCampaign(target: Campaign, patch: Partial<Campaign>): Campaign {
  const next: Campaign = { ...target, ...patch }
  next.name = (patch.name ?? target.name).trim()
  next.objective = (patch.objective ?? target.objective).trim()
  next.owner = (patch.owner ?? target.owner).trim()
  next.notes = (patch.notes ?? target.notes).trim()
  next.status = (patch.status ?? target.status)
  next.goalMetric = (patch.goalMetric ?? target.goalMetric)
  if ('startDate' in patch) next.startDate = campaignDate(patch.startDate ?? null)
  if ('endDate' in patch) next.endDate = campaignDate(patch.endDate ?? null)
  for (const key of ['platforms', 'pillars', 'hashtags', 'contentIds'] as const) {
    if (patch[key]) next[key] = [...new Set(patch[key]!.map((v) => v.trim()).filter(Boolean))]
  }
  return next
}

export async function listCampaigns(): Promise<Campaign[]> {
  await wait(80)
  return clone(campaigns)
}

export async function getCampaign(id: string): Promise<Campaign> {
  await wait(60)
  const found = campaigns.find((c) => c.id === id)
  if (!found) throw new Error('campaign not found')
  return clone(found)
}

export async function createCampaign(patch: Partial<Campaign>): Promise<Campaign> {
  requirePerm('campaigns.write')
  await wait()
  const now = stamp()
  const next = mergeCampaign(
    {
      id: uid('cmp'), name: '', objective: '', status: 'draft', startDate: null, endDate: null,
      platforms: [], pillars: [], hashtags: [], budget: 0, goalMetric: 'views', goalTarget: 0,
      owner: sessionUser(activeToken)?.name ?? setup.owner, notes: '', contentIds: [],
      createdAt: now, updatedAt: now,
    },
    patch,
  )
  validateCampaign(next)
  campaigns.push(next)
  return clone(next)
}

export async function updateCampaign(id: string, patch: Partial<Campaign>): Promise<Campaign> {
  requirePerm('campaigns.write')
  await wait()
  const index = campaigns.findIndex((c) => c.id === id)
  if (index < 0) throw new Error('campaign not found')
  const next = mergeCampaign(campaigns[index], patch)
  validateCampaign(next)
  next.updatedAt = stamp()
  campaigns[index] = next
  return clone(next)
}

export async function deleteCampaign(id: string): Promise<{ ok: boolean }> {
  requirePerm('campaigns.delete')
  await wait()
  const index = campaigns.findIndex((c) => c.id === id)
  if (index < 0) throw new Error('campaign not found')
  campaigns.splice(index, 1)
  return { ok: true }
}

// ---- platform connections ----

// No backend in mock mode ⇒ no OAuth credentials anywhere: the login page runs its
// simulation. The HTTP client returns "redirect" when the server has credentials.
export async function oauthStatus(platform: PlatformId): Promise<OauthStatus> {
  await wait(40)
  return { platform, mode: 'mock', configured: false }
}

export async function startOauth(_platform: PlatformId): Promise<OauthStart> {
  await wait(40)
  requirePerm('platforms.manage')
  return { mode: 'mock' }
}

export async function oauthPending(_platform: PlatformId, _pick: string): Promise<OauthPending> {
  await wait(40)
  throw new Error('Page picking needs the real server — mock mode connects directly.')
}

export async function chooseOAuthPage(
  _platform: PlatformId,
  _pick: string,
  _external_id: string,
): Promise<PlatformConnection> {
  await wait(40)
  throw new Error('Page picking needs the real server — mock mode connects directly.')
}

function stamp(): string {
  return new Date().toISOString().slice(0, 16).replace('T', ' ')
}

function plusDays(days: number): string {
  const d = new Date()
  d.setDate(d.getDate() + days)
  return d.toISOString().slice(0, 10)
}

/** Email of the signed-in account (null in demo mode / before login). */
function sessionEmail(): string | null {
  return activeAccount()?.email ?? null
}

/** Email behind `setup.owner`, used when a workspace is created in demo mode. */
function demoOwnerEmail(): string {
  return accounts.find((a) => a.name === setup.owner)?.email ?? accounts[0]?.email ?? ''
}

/**
 * Mirrors the server's `summary().is_owner`: the signed-in account owns the
 * workspace when its email matches. Demo mode (no session) treats the acting
 * user as the owner, like `requirePerm` does.
 */
function workspaceIsOwner(ws: MockWorkspace): boolean {
  const email = sessionEmail()
  if (!email) return true
  return ws.owner !== '' && normEmail(ws.owner) === email
}

function workspaceSummary(ws: MockWorkspace): WorkspaceSummary {
  return {
    id: ws.id,
    name: ws.name,
    created: ws.created,
    connected: ws.connected,
    total: ws.total,
    isOwner: workspaceIsOwner(ws),
  }
}

export async function listWorkspaces(): Promise<WorkspaceSummary[]> {
  await wait(60)
  return workspaces.map((w) => workspaceSummary(w))
}

export async function createWorkspace(name: string): Promise<WorkspaceSummary> {
  requirePerm('platforms.manage')
  await wait(300)
  const trimmed = name.trim()
  if (!trimmed) throw new Error('workspace name is required')
  if (trimmed.length > 80) throw new Error('workspace name is too long (max 80)')
  const ws: MockWorkspace = {
    id: `ws-${Math.random().toString(36).slice(2, 8)}`,
    name: trimmed,
    created: new Date().toISOString().replace(/\.\d{3}Z$/, 'Z'),
    connected: 0,
    total: connections.length,
    owner: sessionEmail() ?? demoOwnerEmail(),
    members: [],
  }
  workspaces.push(ws)
  return workspaceSummary(ws)
}

export async function renameWorkspace(id: string, name: string): Promise<WorkspaceSummary> {
  requirePerm('platforms.manage')
  await wait(250)
  const trimmed = name.trim()
  if (!trimmed) throw new Error('workspace name is required')
  if (trimmed.length > 80) throw new Error('workspace name is too long (max 80)')
  const ws = workspaces.find((w) => w.id === id)
  if (!ws) throw new Error('workspace not found')
  ws.name = trimmed
  return workspaceSummary(ws)
}

export async function deleteWorkspace(id: string): Promise<{ deleted: string; fallback: string }> {
  requirePerm('platforms.manage')
  await wait(250)
  if (workspaces.length <= 1) throw new Error('the last workspace cannot be deleted — create another one first')
  const index = workspaces.findIndex((w) => w.id === id)
  if (index < 0) throw new Error('workspace not found')
  workspaces.splice(index, 1)
  return { deleted: id, fallback: workspaces[0].id }
}

// ---- workspace members (owner-only) ----

function memberWorkspace(id: string): MockWorkspace {
  const ws = workspaces.find((w) => w.id === id)
  if (!ws) throw new Error('workspace not found')
  return ws
}

/** Owner gate for the member endpoints (mirrors the server's `ensure_owner`). */
function ensureOwner(ws: MockWorkspace): void {
  const email = sessionEmail()
  if (email && ws.owner !== '' && normEmail(ws.owner) !== email) {
    throw new Error('403 forbidden: only the workspace owner can manage members')
  }
}

/** Display name behind an email: the account's name, else the local part. */
function memberName(email: string): string {
  const account = accounts.find((a) => a.email === normEmail(email))
  if (account) return account.name
  return normEmail(email).split('@')[0] || normEmail(email)
}

/** Directory role of a display name, `Viewer` when the user is gone. */
function memberRole(name: string): string {
  return setup.users.find((u) => u.name === name)?.role ?? 'Viewer'
}

function membersView(ws: MockWorkspace): WorkspaceMembers {
  return {
    owner: { email: ws.owner, name: memberName(ws.owner), role: 'Owner' },
    members: ws.members.map((email) => {
      const name = memberName(email)
      return { email: normEmail(email), name, role: memberRole(name) }
    }),
  }
}

function normalizeMemberEmail(raw: string): string {
  const email = normEmail(raw)
  if (!email.includes('@') || /\s/.test(email)) throw new Error('a valid email address is required')
  return email
}

export async function listWorkspaceMembers(id: string): Promise<WorkspaceMembers> {
  await wait(80)
  const ws = memberWorkspace(id)
  ensureOwner(ws)
  return clone(membersView(ws))
}

export async function addWorkspaceMember(id: string, email: string): Promise<WorkspaceMembers> {
  await wait()
  const ws = memberWorkspace(id)
  ensureOwner(ws)
  const target = normalizeMemberEmail(email)
  if (!accounts.some((a) => a.email === target)) throw new Error('no account with that email')
  if (normEmail(ws.owner) === target || ws.members.some((m) => normEmail(m) === target)) {
    throw new Error('that account already owns or belongs to this workspace')
  }
  ws.members.push(target)
  return clone(membersView(ws))
}

export async function removeWorkspaceMember(id: string, email: string): Promise<WorkspaceMembers> {
  await wait()
  const ws = memberWorkspace(id)
  ensureOwner(ws)
  const target = normEmail(email)
  const index = ws.members.findIndex((m) => normEmail(m) === target)
  if (index < 0) throw new Error('that account is not a member of this workspace')
  ws.members.splice(index, 1)
  return clone(membersView(ws))
}

export async function listPlatforms(): Promise<PlatformConnection[]> {
  await wait(60)
  return clone(connections)
}

export async function getLive(): Promise<LiveData> {
  await wait(60)
  return clone(live)
}

// ---- Meta Ads mirror + management ----

function adsView(): AdsView {
  return { ...clone(ads), canManage: adsFlags.manage }
}

function pushAdsAudit(action: string, target: string, detail: string): void {
  ads.audit.unshift({ at: stamp(), actor: 'you', action, target, detail })
  ads.audit.splice(50)
}

export async function getAds(): Promise<AdsView> {
  await wait(60)
  return adsView()
}

export async function syncAds(): Promise<AdsView> {
  requirePerm('platforms.manage')
  await wait(600)
  const meta = connections.find((c) => c.id === 'meta')
  if (!meta || meta.status !== 'connected') throw new Error('connect Meta first')
  if (meta.hasToken === false) {
    throw new Error('reconnect Meta to grant access — the stored token is missing')
  }
  ads.fetchedAt = Math.floor(Date.now() / 1000)
  ads.error = ''
  meta.lastSync = stamp()
  return adsView()
}

export async function setAdsManage(enabled: boolean): Promise<{ canManage: boolean }> {
  requirePerm('platforms.manage')
  await wait(120)
  adsFlags.manage = enabled
  return { canManage: adsFlags.manage }
}

function manageCampaign(id: string): AdCampaign {
  requirePerm('platforms.manage')
  if (!adsFlags.manage) throw new Error('turn on campaign management for this workspace first')
  const campaign = ads.campaigns.find((c) => c.id === id)
  if (!campaign) throw new Error('campaign not found in this workspace\'s mirror')
  return campaign
}

export async function setAdCampaignStatus(
  id: string,
  status: 'ACTIVE' | 'PAUSED' | 'ARCHIVED',
): Promise<{ ok: boolean }> {
  const campaign = manageCampaign(id)
  await wait(350)
  campaign.status = status
  campaign.effectiveStatus = status
  pushAdsAudit(`campaign ${status.toLowerCase()}`, id, campaign.name)
  return { ok: true }
}

export async function setAdCampaignBudget(
  id: string,
  budget: { dailyBudget?: number; lifetimeBudget?: number },
): Promise<{ ok: boolean }> {
  const campaign = manageCampaign(id)
  if (budget.dailyBudget === undefined && budget.lifetimeBudget === undefined) {
    throw new Error('provide dailyBudget or lifetimeBudget')
  }
  await wait(350)
  if (budget.dailyBudget !== undefined) campaign.dailyBudget = budget.dailyBudget
  if (budget.lifetimeBudget !== undefined) campaign.lifetimeBudget = budget.lifetimeBudget
  pushAdsAudit(
    'budget updated',
    id,
    [
      budget.dailyBudget !== undefined ? `daily ${budget.dailyBudget}` : '',
      budget.lifetimeBudget !== undefined ? `lifetime ${budget.lifetimeBudget}` : '',
    ].filter(Boolean).join(', '),
  )
  return { ok: true }
}

export async function duplicateAdCampaign(id: string): Promise<{ campaignId: string }> {
  const campaign = manageCampaign(id)
  await wait(500)
  const copyId = `cmp-ads-${Math.random().toString(36).slice(2, 8)}`
  ads.campaigns.unshift({
    ...clone(campaign),
    id: copyId,
    name: `${campaign.name} (copy)`,
    status: 'PAUSED',
    effectiveStatus: 'PAUSED',
    createdTime: new Date().toISOString(),
  })
  pushAdsAudit('campaign duplicated', id, `copy ${copyId} (paused)`)
  return { campaignId: copyId }
}

export async function createAdBoost(input: {
  name: string
  objective: string
  dailyBudget: number
  days: number
  countries: string[]
  storyId: string
}): Promise<{ campaignId: string }> {
  requirePerm('platforms.manage')
  if (!adsFlags.manage) throw new Error('turn on campaign management for this workspace first')
  const name = input.name.trim()
  if (!name) throw new Error('name is required')
  if (input.dailyBudget <= 0) throw new Error('dailyBudget must be greater than zero')
  if (!input.countries.length) throw new Error('pick at least one country to target')
  const account = ads.accounts[0]
  if (!account) throw new Error('sync Meta Ads first')
  const post = live.posts.find((p) => p.id === input.storyId)
  if (!post) throw new Error('pick a post from the live mirror')
  await wait(800)

  const campaignId = `cmp-ads-${Math.random().toString(36).slice(2, 8)}`
  const adsetId = `adset-${Math.random().toString(36).slice(2, 8)}`
  const adId = `ad-${Math.random().toString(36).slice(2, 8)}`
  const end = new Date(Date.now() + Math.min(Math.max(input.days, 1), 90) * 86400000)
    .toISOString().replace(/\.\d{3}Z$/, '+0000')
  ads.campaigns.unshift({
    id: campaignId, accountId: account.id, name,
    objective: input.objective, status: 'PAUSED', effectiveStatus: 'PAUSED',
    buyingType: 'AUCTION', dailyBudget: input.dailyBudget, lifetimeBudget: 0,
    budgetRemaining: 0, bidStrategy: 'LOWEST_COST_WITHOUT_CAP',
    specialAdCategories: ['NONE'],
    startTime: new Date().toISOString().replace(/\.\d{3}Z$/, '+0000'),
    stopTime: '', createdTime: new Date().toISOString(),
  })
  ads.adsets.unshift({
    id: adsetId, campaignId, name: `${name} — audience`,
    status: 'PAUSED', effectiveStatus: 'PAUSED',
    dailyBudget: 0, lifetimeBudget: 0,
    optimizationGoal: input.objective === 'OUTCOME_ENGAGEMENT' ? 'POST_ENGAGEMENT' : 'LINK_CLICKS',
    billingEvent: 'IMPRESSIONS', bidAmount: 0,
    startTime: '', endTime: end,
    targeting: input.countries.join(', '),
    promotedObject: `page ${connections.find((c) => c.id === 'meta')?.externalId ?? ''}`,
  })
  ads.ads.unshift({
    id: adId, adsetId, name: `${name} — ad`,
    status: 'PAUSED', effectiveStatus: 'PAUSED',
    creativeTitle: post.caption.slice(0, 60), creativeBody: post.caption,
    imageUrl: post.mediaUrl, thumbnailUrl: post.thumbnailUrl,
    storyId: post.id, previewUrl: post.permalink,
  })
  pushAdsAudit('boost created', campaignId, `ad set ${adsetId}, ad ${adId}, paused`)
  return { campaignId }
}

export async function connectPlatform(id: PlatformId, handle?: string): Promise<PlatformConnection> {
  requirePerm('platforms.manage')
  await wait(500)
  const c = connections.find((x) => x.id === id)
  if (!c) throw new Error('platform not found')
  const linked = (handle ?? '').trim()
  if (linked.length > 200) throw new Error('handle must be at most 200 characters')
  if (linked && !validProfileHandle(id, linked)) {
    throw new Error('enter a profile URL or @handle (not the platform home page)')
  }
  c.status = 'connected'
  c.hasToken = true
  c.handle =
    linked ||
    c.handle ||
    (id === 'meta' ? 'https://www.facebook.com/profile.php?id=100000000000001' : 'Studio Channel')
  c.externalId = c.externalId || `ext-${Math.random().toString(36).slice(2, 10)}`
  c.scopes = c.scopes.length ? c.scopes : ['basic']
  c.tokenType = id === 'youtube' || id === 'tiktok' ? 'refresh' : 'long-lived'
  c.expiresAt = plusDays(id === 'youtube' ? 0 : 60)
  c.lastSync = stamp()
  c.mediaCount = c.mediaCount || 0
  return clone(c)
}

export async function disconnectPlatform(id: PlatformId): Promise<PlatformConnection> {
  requirePerm('platforms.manage')
  await wait(250)
  const c = connections.find((x) => x.id === id)
  if (!c) throw new Error('platform not found')
  c.status = 'disconnected'
  c.tokenType = '—'
  c.expiresAt = null
  c.lastSync = null
  c.hasToken = false
  return clone(c)
}

export async function syncPlatform(id: PlatformId): Promise<PlatformConnection> {
  requirePerm('platforms.manage')
  await wait(600)
  const c = connections.find((x) => x.id === id)
  if (!c) throw new Error('platform not found')
  if (c.status !== 'connected') throw new Error('platform is not connected')
  c.lastSync = stamp()
  if (id === 'meta') {
    // Meta sync refreshes the live mirror (the server pulls the Graph API).
    live.fetchedAt = Math.floor(Date.now() / 1000)
    live.error = ''
    c.mediaCount = live.posts.length
  } else {
    c.mediaCount += 3
  }
  return clone(c)
}

// ---- CMS: content studio ----

const nowIso = (): string => new Date().toISOString().replace(/\.\d{3}Z$/, 'Z')

function promoteDue(): void {
  const now = Date.now()
  for (const c of content) {
    if (c.status !== 'scheduled' || !c.scheduledFor) continue
    if (new Date(c.scheduledFor).getTime() <= now) {
      c.status = 'published'
      c.publishedAt = nowIso()
      c.scheduledFor = null
      c.updatedAt = nowIso()
      c.version += 1
    }
  }
}

function slugify(input: string): string {
  const slug = input
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, '-')
    .replace(/^-+|-+$/g, '')
    .slice(0, 120)
  return slug || 'content'
}

function validSlug(slug: string): boolean {
  return /^[a-z0-9]+(?:-[a-z0-9]+)*$/.test(slug) && slug.length <= 120
}

function uniqueSlug(base: string, excludeId?: string): string {
  const taken = (slug: string): boolean => content.some((c) => c.id !== excludeId && c.slug === slug)
  if (!taken(base)) return base
  for (let n = 2; n < 10_000; n++) {
    const candidate = `${base}-${n}`
    if (!taken(candidate)) return candidate
  }
  return `${base}-${Math.random().toString(36).slice(2, 6)}`
}

function validateContent(c: Content): void {
  if (!c.title.trim()) throw new Error('title is required')
  checkLen('title', c.title, 200)
  if (!validSlug(c.slug)) throw new Error('slug must use lowercase letters, numbers and dashes')
  if (!['article', 'page', 'note'].includes(c.kind)) throw new Error('kind must be article, page or note')
  checkLen('body', c.body, 200_000)
  checkLen('excerpt', c.excerpt, 500)
  checkLen('heroImageUrl', c.heroImageUrl, 2000)
  checkLen('seoTitle', c.seoTitle, 200)
  checkLen('seoDescription', c.seoDescription, 500)
  if (c.tags.length > 20) throw new Error('tags must have at most 20 items')
  for (const tag of c.tags) checkLen('tag', tag, 50)
}

function checkContentVersion(c: Content, version: number | undefined): void {
  if (version === undefined) throw new Error('version is required')
  if (c.version !== version) {
    throw new Error(`content changed since version ${version} (current version is ${c.version})`)
  }
}

function pushRevision(c: Content, author: string, note: string): void {
  const next = c.revisions.reduce((max, r) => Math.max(max, r.revision), 0) + 1
  c.revisions.push({
    revision: next,
    savedAt: nowIso(),
    author,
    note,
    title: c.title,
    body: c.body,
    excerpt: c.excerpt,
    seoTitle: c.seoTitle,
    seoDescription: c.seoDescription,
    tags: [...c.tags],
    status: c.status,
  })
  if (c.revisions.length > 20) c.revisions.splice(0, c.revisions.length - 20)
}

function contentActor(fallback?: string): string {
  // Attribute to the signed-in account whenever there is one, even in demo
  // mode (mirrors the server's content_actor).
  const user = sessionUser(activeToken)
  return user?.name ?? (fallback?.trim() || setup.owner)
}

function contentView(c: Content): Content {
  return clone({ ...c, revisions: c.revisions.map((r) => ({ ...r, body: '' })) })
}

export async function listContent(params?: { status?: string; q?: string }): Promise<ContentSummary[]> {
  await wait()
  promoteDue()
  const needle = (params?.q ?? '').trim().toLowerCase()
  return content
    .filter((c) => !params?.status || c.status === params.status)
    .filter((c) =>
      !needle
      || c.title.toLowerCase().includes(needle)
      || c.excerpt.toLowerCase().includes(needle)
      || c.tags.some((t) => t.toLowerCase().includes(needle)),
    )
    .map((c) => clone(contentSummary(c)))
    .sort((a, b) => b.updatedAt.localeCompare(a.updatedAt))
}

export async function getContent(id: string): Promise<Content> {
  await wait(60)
  promoteDue()
  const c = content.find((x) => x.id === id)
  if (!c) throw new Error('content not found')
  return contentView(c)
}

export async function createContent(draft: ContentDraft): Promise<Content> {
  requirePerm('content.write')
  await wait()
  const stamp = nowIso()
  const item: Content = {
    id: uid('c'),
    title: draft.title,
    slug: draft.slug?.trim() ? draft.slug.trim() : slugify(draft.title),
    kind: draft.kind ?? 'article',
    status: draft.status === 'review' ? 'review' : 'draft',
    body: draft.body ?? '',
    excerpt: draft.excerpt ?? '',
    heroImageUrl: draft.heroImageUrl ?? '',
    tags: [...(draft.tags ?? [])],
    seoTitle: draft.seoTitle ?? '',
    seoDescription: draft.seoDescription ?? '',
    author: contentActor(draft.author),
    createdAt: stamp,
    updatedAt: stamp,
    publishedAt: null,
    scheduledFor: null,
    version: 1,
    revisions: [],
  }
  item.slug = uniqueSlug(item.slug)
  validateContent(item)
  content.push(item)
  return contentView(item)
}

export async function updateContent(id: string, patch: ContentPatchInput): Promise<Content> {
  requirePerm('content.write')
  await wait()
  const c = content.find((x) => x.id === id)
  if (!c) throw new Error('content not found')
  checkContentVersion(c, patch.version)
  if (patch.status && !['draft', 'review', 'archived'].includes(patch.status)) {
    throw new Error('status can only be set to draft, review or archived here')
  }
  const author = contentActor()
  const candidate: Content = {
    ...c,
    ...('title' in patch ? { title: patch.title! } : {}),
    ...('slug' in patch ? { slug: patch.slug! } : {}),
    ...('kind' in patch ? { kind: patch.kind! } : {}),
    ...('status' in patch ? { status: patch.status! } : {}),
    ...('body' in patch ? { body: patch.body! } : {}),
    ...('excerpt' in patch ? { excerpt: patch.excerpt! } : {}),
    ...('heroImageUrl' in patch ? { heroImageUrl: patch.heroImageUrl! } : {}),
    ...('tags' in patch ? { tags: [...patch.tags!] } : {}),
    ...('seoTitle' in patch ? { seoTitle: patch.seoTitle! } : {}),
    ...('seoDescription' in patch ? { seoDescription: patch.seoDescription! } : {}),
  }
  if (!candidate.slug.trim()) throw new Error('slug is required')
  candidate.slug = uniqueSlug(candidate.slug, id)
  validateContent(candidate)
  pushRevision(c, author, patch.note ?? 'Edit')
  Object.assign(c, {
    title: candidate.title,
    slug: candidate.slug,
    kind: candidate.kind,
    status: candidate.status,
    body: candidate.body,
    excerpt: candidate.excerpt,
    heroImageUrl: candidate.heroImageUrl,
    tags: candidate.tags,
    seoTitle: candidate.seoTitle,
    seoDescription: candidate.seoDescription,
    updatedAt: nowIso(),
    version: c.version + 1,
  })
  return contentView(c)
}

export async function publishContent(id: string, version: number): Promise<Content> {
  requirePerm('content.publish')
  await wait()
  const c = content.find((x) => x.id === id)
  if (!c) throw new Error('content not found')
  checkContentVersion(c, version)
  if (!c.title.trim()) throw new Error('title is required')
  pushRevision(c, contentActor(), 'Published')
  const stamp = nowIso()
  c.status = 'published'
  c.publishedAt = stamp
  c.scheduledFor = null
  c.updatedAt = stamp
  c.version += 1
  return contentView(c)
}

export async function unpublishContent(id: string, version: number): Promise<Content> {
  requirePerm('content.publish')
  await wait()
  const c = content.find((x) => x.id === id)
  if (!c) throw new Error('content not found')
  checkContentVersion(c, version)
  pushRevision(c, contentActor(), 'Unpublished')
  c.status = 'draft'
  c.scheduledFor = null
  c.updatedAt = nowIso()
  c.version += 1
  return contentView(c)
}

export async function scheduleContent(id: string, version: number, scheduledFor: string): Promise<Content> {
  requirePerm('content.publish')
  await wait()
  const when = new Date(scheduledFor)
  if (Number.isNaN(when.getTime())) throw new Error('scheduledFor must be an RFC 3339 timestamp')
  if (when.getTime() <= Date.now()) throw new Error('scheduledFor must be in the future')
  const c = content.find((x) => x.id === id)
  if (!c) throw new Error('content not found')
  checkContentVersion(c, version)
  pushRevision(c, contentActor(), 'Scheduled')
  c.status = 'scheduled'
  c.scheduledFor = when.toISOString().replace(/\.\d{3}Z$/, 'Z')
  c.updatedAt = nowIso()
  c.version += 1
  return contentView(c)
}

export async function deleteContent(id: string): Promise<{ ok: boolean }> {
  requirePerm('content.delete')
  await wait()
  const index = content.findIndex((x) => x.id === id)
  if (index < 0) throw new Error('content not found')
  content.splice(index, 1)
  return { ok: true }
}

export async function duplicateContent(id: string): Promise<Content> {
  requirePerm('content.write')
  await wait()
  const original = content.find((x) => x.id === id)
  if (!original) throw new Error('content not found')
  const stamp = nowIso()
  const copy: Content = {
    ...clone(original),
    id: uid('c'),
    title: `${original.title} (copy)`,
    slug: uniqueSlug(slugify(`${original.title} (copy)`)),
    status: 'draft',
    publishedAt: null,
    scheduledFor: null,
    version: 1,
    author: contentActor(),
    createdAt: stamp,
    updatedAt: stamp,
    revisions: [],
  }
  content.push(copy)
  return contentView(copy)
}

export async function listContentRevisions(id: string): Promise<ContentRevision[]> {
  await wait(60)
  const c = content.find((x) => x.id === id)
  if (!c) throw new Error('content not found')
  return clone([...c.revisions].sort((a, b) => b.revision - a.revision))
}

export async function getContentRevision(id: string, revision: number): Promise<ContentRevision> {
  await wait(60)
  const c = content.find((x) => x.id === id)
  if (!c) throw new Error('content not found')
  const found = c.revisions.find((r) => r.revision === revision)
  if (!found) throw new Error('revision not found')
  return clone(found)
}

export async function restoreContentRevision(id: string, revision: number, version: number): Promise<Content> {
  requirePerm('content.write')
  await wait()
  const c = content.find((x) => x.id === id)
  if (!c) throw new Error('content not found')
  checkContentVersion(c, version)
  const snapshot = c.revisions.find((r) => r.revision === revision)
  if (!snapshot) throw new Error('revision not found')
  pushRevision(c, contentActor(), `Restored revision ${snapshot.revision}`)
  c.title = snapshot.title
  c.body = snapshot.body
  c.excerpt = snapshot.excerpt
  c.seoTitle = snapshot.seoTitle
  c.seoDescription = snapshot.seoDescription
  c.tags = [...snapshot.tags]
  c.status = snapshot.status === 'review' ? 'review' : 'draft'
  c.scheduledFor = null
  c.updatedAt = nowIso()
  c.version += 1
  return contentView(c)
}

export async function listPublicContent(): Promise<ContentSummary[]> {
  await wait(60)
  promoteDue()
  return content
    .filter((c) => c.status === 'published')
    .map((c) => clone(contentSummary(c)))
    .sort((a, b) => (b.publishedAt ?? '').localeCompare(a.publishedAt ?? ''))
}

export async function getPublicContent(slug: string): Promise<PublicContent> {
  await wait(60)
  promoteDue()
  const c = content.find((x) => x.slug === slug && x.status === 'published')
  if (!c) throw new Error('content not found')
  return {
    id: c.id,
    title: c.title,
    slug: c.slug,
    kind: c.kind,
    body: c.body,
    excerpt: c.excerpt,
    heroImageUrl: c.heroImageUrl,
    tags: [...c.tags],
    author: c.author,
    publishedAt: c.publishedAt,
    updatedAt: c.updatedAt,
  }
}

// ---- admin onboarding ----

export async function createAccount(v: {
  name: string
  email: string
  role: string
  password?: string
}): Promise<CreatedAccount> {
  requirePerm('users.manage')
  await wait()
  const name = v.name.trim()
  if (!name) throw new Error('name is required')
  const email = normEmail(v.email)
  if (!email.includes('@')) throw new Error('a valid email is required')
  if (!setup.roles.some((r) => r.name === v.role)) throw new Error(`unknown role '${v.role}'`)
  if (accounts.some((a) => a.email === email)) throw new Error('email already registered')
  if (accounts.some((a) => a.name.toLowerCase() === name.toLowerCase())) throw new Error('name already taken')

  const generated = !v.password
  const password = v.password || `cp-${Math.random().toString(36).slice(2, 14)}`
  if (!generated && password.length < 12) throw new Error('password must be at least 12 characters')

  const dir = setup.users.find((u) => u.name.toLowerCase() === name.toLowerCase())
  if (dir) dir.role = v.role
  else setup.users.push({ name, role: v.role })
  accounts.push({ name, email, password, role: v.role, plan: 'free' })
  return {
    ok: true,
    account: { name, email, sessions: 0, role: v.role },
    temporaryPassword: generated ? password : null,
  }
}

// Compile-time proof that the mock implements the full contract.
const _api: Api = {
  listPosts, getSetup, saveSetup, addOption, addUser, removeUser, addRole, removeRole,
  register, login, changePassword, updateProfile, setPlan, me, logout, listAccounts, deleteAccount, logoutAll,
  listFonts, uploadFont, deleteFont, uploadBrandImage, deleteBrandImage,
  listCampaigns, getCampaign, createCampaign, updateCampaign, deleteCampaign,
  lockPost, unlockPost, updatePost, addPost, deletePost,
  listIdeas, addIdea, toggleIdea, promoteIdea,
  listTags, addTag,
  listMetrics, importMetrics,
  listTxns, addTxn,
  getBrand, saveBrand,
  listWorkspaces, createWorkspace, renameWorkspace, deleteWorkspace,
  listWorkspaceMembers, addWorkspaceMember, removeWorkspaceMember,
  listPlatforms, getLive, getAds, syncAds, setAdsManage, setAdCampaignStatus, setAdCampaignBudget,
  duplicateAdCampaign, createAdBoost,
  oauthStatus, startOauth, oauthPending, chooseOAuthPage, connectPlatform, disconnectPlatform, syncPlatform,
  createAccount,
  listContent, getContent, createContent, updateContent, publishContent, unpublishContent,
  scheduleContent, deleteContent, duplicateContent, listContentRevisions, getContentRevision,
  restoreContentRevision, listPublicContent, getPublicContent,
}
void _api
