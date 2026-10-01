// The data-source contract shared by the in-memory mock (`src/mock/api.ts`)
// and the HTTP client (`src/api/http.ts`). `src/api/index.ts` picks one and the
// rest of the app is source-agnostic.
import type {
  Post, SetupConfig, Idea, HashtagGroup, Metric, Txn, Brand,
  PlatformConnection, PlatformId, User,
  Content, ContentKind, ContentRevision, ContentStatus, ContentSummary, PublicContent,
  FontAsset, Campaign, WorkspaceSummary, WorkspaceMembers, LiveData, AdsView,
} from '@/mock/db'
import type { BrandImageKind } from '@/core/images'

export type OptionList = 'pillars' | 'formats' | 'goals' | 'statuses' | 'platforms'

export interface AuthResult {
  token: string
  user: User
}

export interface ImportResult {
  imported: number
  metrics: Metric[]
}

export type OauthMode = 'mock' | 'redirect'

export interface OauthStatus {
  platform: PlatformId
  mode: OauthMode
  configured: boolean
  authorizeHost?: string
}

export interface OauthStart {
  mode: OauthMode
  url?: string
}

export interface OauthPick {
  handle: string
  external_id: string
}

export interface OauthPending {
  platform: PlatformId
  accounts: OauthPick[]
}

export interface AccountInfo {
  name: string
  email: string
  sessions: number
}

/** Result of an admin-created account; the temporary password is shown once. */
export interface CreatedAccount {
  ok: boolean
  account: AccountInfo & { role: string }
  temporaryPassword: string | null
}

/** Fields accepted when creating a content item. */
export interface ContentDraft {
  title: string
  slug?: string
  kind?: ContentKind
  status?: ContentStatus
  body?: string
  excerpt?: string
  heroImageUrl?: string
  tags?: string[]
  seoTitle?: string
  seoDescription?: string
  author?: string
}

/** Editable fields + the concurrency version for a content update. */
export interface ContentPatchInput {
  version: number
  title?: string
  slug?: string
  kind?: ContentKind
  status?: ContentStatus
  body?: string
  excerpt?: string
  heroImageUrl?: string
  tags?: string[]
  seoTitle?: string
  seoDescription?: string
  note?: string
}

export interface Api {
  listPosts(month?: number): Promise<Post[]>
  getSetup(): Promise<SetupConfig>
  saveSetup(patch: Partial<SetupConfig>): Promise<SetupConfig>
  addOption(list: OptionList, item: string): Promise<SetupConfig>
  addUser(name: string, role: string): Promise<SetupConfig>
  removeUser(name: string): Promise<SetupConfig>
  addRole(name: string, permissions?: string[]): Promise<SetupConfig>
  removeRole(name: string): Promise<SetupConfig>

  register(v: { name: string; email: string; password: string }): Promise<AuthResult>
  login(email: string, password: string): Promise<AuthResult>
  changePassword(oldPassword: string, newPassword: string): Promise<{ ok: boolean }>
  updateProfile(name: string): Promise<{ user: User }>
  setPlan(plan: string): Promise<{ user: User }>
  me(token: string): Promise<{ user: User }>
  logout(token: string): Promise<{ ok: boolean }>
  listAccounts(): Promise<AccountInfo[]>
  deleteAccount(email: string): Promise<{ ok: boolean }>
  logoutAll(token?: string): Promise<{ ok: boolean }>

  lockPost(id: string, user: string): Promise<Post>
  unlockPost(id: string, user: string): Promise<Post>
  updatePost(id: string, patch: Partial<Post> & { user?: string }): Promise<Post>
  addPost(draft: Omit<Post, 'id'>): Promise<Post>
  deletePost(id: string): Promise<{ ok: boolean }>

  listIdeas(): Promise<Idea[]>
  addIdea(draft: Omit<Idea, 'id'>): Promise<Idea>
  toggleIdea(id: string): Promise<Idea>
  promoteIdea(id: string, month: number): Promise<Post>

  listTags(): Promise<HashtagGroup[]>
  addTag(groupId: string, tag: string): Promise<HashtagGroup>

  listMetrics(): Promise<Metric[]>
  importMetrics(platform: string, month: number): Promise<ImportResult>

  listTxns(): Promise<Txn[]>
  addTxn(draft: Omit<Txn, 'id'>): Promise<Txn>

  getBrand(): Promise<Brand>
  saveBrand(patch: Partial<Brand>): Promise<Brand>
  listCampaigns(): Promise<Campaign[]>
  getCampaign(id: string): Promise<Campaign>
  createCampaign(patch: Partial<Campaign>): Promise<Campaign>
  updateCampaign(id: string, patch: Partial<Campaign>): Promise<Campaign>
  deleteCampaign(id: string): Promise<{ ok: boolean }>

  listFonts(): Promise<FontAsset[]>
  /** `data` is the base64 payload (no `data:` URL prefix). */
  uploadFont(name: string, data: string): Promise<FontAsset[]>
  deleteFont(name: string): Promise<FontAsset[]>
  /** Uploads a logo/moodboard image; returns the URL to store in the brand. */
  uploadBrandImage(
    name: string,
    data: string,
    kind: BrandImageKind,
  ): Promise<{ url: string; width: number; height: number }>
  deleteBrandImage(name: string): Promise<{ ok: boolean }>

  listWorkspaces(): Promise<WorkspaceSummary[]>
  createWorkspace(name: string): Promise<WorkspaceSummary>
  renameWorkspace(id: string, name: string): Promise<WorkspaceSummary>
  deleteWorkspace(id: string): Promise<{ deleted: string; fallback: string }>
  listWorkspaceMembers(id: string): Promise<WorkspaceMembers>
  addWorkspaceMember(id: string, email: string): Promise<WorkspaceMembers>
  removeWorkspaceMember(id: string, email: string): Promise<WorkspaceMembers>

  listPlatforms(): Promise<PlatformConnection[]>
  /** Mirror of the connected platforms' real posts (refreshed by sync). */
  getLive(): Promise<LiveData>
  /** Meta Ads mirror: accounts, campaigns, ad sets, ads, settings + insights. */
  getAds(): Promise<AdsView>
  syncAds(): Promise<AdsView>
  /** Explicit opt-in for campaign management (pause/budget/duplicate/boost). */
  setAdsManage(enabled: boolean): Promise<{ canManage: boolean }>
  setAdCampaignStatus(id: string, status: 'ACTIVE' | 'PAUSED' | 'ARCHIVED'): Promise<{ ok: boolean }>
  setAdCampaignBudget(
    id: string,
    budget: { dailyBudget?: number; lifetimeBudget?: number },
  ): Promise<{ ok: boolean }>
  duplicateAdCampaign(id: string): Promise<{ campaignId: string }>
  createAdBoost(input: {
    name: string
    objective: string
    dailyBudget: number
    days: number
    countries: string[]
    storyId: string
  }): Promise<{ campaignId: string }>
  oauthStatus(platform: PlatformId): Promise<OauthStatus>
  startOauth(platform: PlatformId): Promise<OauthStart>
  oauthPending(platform: PlatformId, pick: string): Promise<OauthPending>
  chooseOAuthPage(platform: PlatformId, pick: string, external_id: string): Promise<PlatformConnection>
  connectPlatform(id: PlatformId, handle?: string): Promise<PlatformConnection>
  disconnectPlatform(id: PlatformId): Promise<PlatformConnection>
  syncPlatform(id: PlatformId): Promise<PlatformConnection>

  createAccount(v: { name: string; email: string; role: string; password?: string }): Promise<CreatedAccount>

  listContent(params?: { status?: string; q?: string }): Promise<ContentSummary[]>
  getContent(id: string): Promise<Content>
  createContent(draft: ContentDraft): Promise<Content>
  updateContent(id: string, patch: ContentPatchInput): Promise<Content>
  publishContent(id: string, version: number): Promise<Content>
  unpublishContent(id: string, version: number): Promise<Content>
  scheduleContent(id: string, version: number, scheduledFor: string): Promise<Content>
  deleteContent(id: string): Promise<{ ok: boolean }>
  duplicateContent(id: string): Promise<Content>
  listContentRevisions(id: string): Promise<ContentRevision[]>
  getContentRevision(id: string, revision: number): Promise<ContentRevision>
  restoreContentRevision(id: string, revision: number, version: number): Promise<Content>
  listPublicContent(): Promise<ContentSummary[]>
  getPublicContent(slug: string): Promise<PublicContent>
}
