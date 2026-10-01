// Domain model transcribed from the workbook (01 §4, §7).
// Sheets 01–12 share one schema; Set up drives every dropdown.

export type Status = 'Start' | 'Design' | 'Dev' | 'Done'
export type TxnKind = 'IN' | 'OUT'

export type Post = {
  id: string
  month: number // 1–12
  topic: string
  pillar: string
  format: string
  goal: string
  date: string | null // YYYY-MM-DD
  time: string
  status: Status
  hook: string
  caption: string
  cta: string
  hashtagGroup: string
  hashtags: string[]
  imageUrl: string
  note: string
  done: boolean
  platforms: string[]
  lockedBy?: string | null // US-014: another user is editing this row
}

export interface User {
  name: string
  role: string
  /** SaaS package for the signed-in account ("free" | "pro" | "business"). */
  plan?: string
}

export const PERMISSIONS = [
  'setup.write',
  'users.manage',
  'posts.write',
  'posts.lock',
  'metrics.import',
  'finance.write',
  'brand.write',
  'ideas.write',
  'hashtags.write',
  'platforms.manage',
  'content.write',
  'content.publish',
  'content.delete',
  'campaigns.write',
  'campaigns.delete',
] as const

export type Permission = (typeof PERMISSIONS)[number]

export interface Role {
  name: string
  permissions: string[]
}

export interface SetupConfig {
  language: 'EN' | 'TH'
  year: number
  owner: string
  workspaceName: string
  users: User[]
  pillars: string[]
  formats: string[]
  goals: string[]
  statuses: Status[]
  platforms: string[]
  showEditableColors: boolean
  authRequired: boolean
  allowRegistration: boolean
  roles: Role[]
}

export interface Idea {
  id: string
  topic: string
  format: string
  idea: string
  link: string
  done: boolean
}

export interface HashtagGroup {
  id: string
  title: string
  tags: string[]
}

export interface Metric {
  postId: string
  platform: string
  likes: number
  views: number
}

export type Txn = {
  id: string
  date: string
  amount: number
  kind: TxnKind
  category: string
  sub: string
}

export interface FontAsset {
  name: string
  family: string
  url: string
  size: number
  uploadedAt: number
}

export interface Brand {
  channel: string
  positioning: string
  slogan: string
  audience: string
  voice: string
  dos: string[]
  donts: string[]
  palette: string[]
  fonts: string[]
  /** Logo image URLs (main, secondary, social — blanks allowed). */
  logos: string[]
  /** Moodboard image URLs. */
  moodboard: string[]
  /** Corner radius in px for the whole site (0–24). */
  radius: number
  /** Accent fill strength in percent for solid surfaces (5–100). */
  fillOpacity: number
  /** Card/panel border width in px (0–3). */
  strokeWidth: number
  /** Shadow depth: none | soft | strong. */
  shadow: string
}

export type CampaignStatus = 'draft' | 'active' | 'paused' | 'completed'
export type CampaignMetric = 'views' | 'likes' | 'reach' | 'posts'

export interface Campaign {
  id: string
  name: string
  objective: string
  status: CampaignStatus
  startDate: string | null
  endDate: string | null
  platforms: string[]
  pillars: string[]
  hashtags: string[]
  budget: number
  goalMetric: CampaignMetric
  goalTarget: number
  owner: string
  notes: string
  contentIds: string[]
  createdAt: string
  updatedAt: string
}

export interface PlatformGoal {
  platform: string
  start: number
  goal: number
}

export const MONTHS: number[] = Array.from({ length: 12 }, (_, i) => i + 1)
export const MONTH_NAMES = [
  'January', 'February', 'March', 'April', 'May', 'June',
  'July', 'August', 'September', 'October', 'November', 'December',
]

export function uid(prefix: string): string {
  return `${prefix}-${Math.random().toString(36).slice(2, 8)}`
}

export function plus7(dateStr: string): string {
  const d = new Date(dateStr + 'T00:00:00')
  d.setDate(d.getDate() + 7)
  return d.toISOString().slice(0, 10)
}

export function weekOfMonth(dateStr: string): number {
  return Math.ceil(new Date(dateStr + 'T00:00:00').getDate() / 7)
}

export function fmtNum(n: number): string {
  return n.toLocaleString('en-US')
}

// ---- seed (mirrors Set up ranges + a February of sample posts) ----
export const setup: SetupConfig = {
  language: 'EN',
  year: 2026,
  owner: 'Studio Owner',
  workspaceName: 'workspace',
  users: [
    { name: 'Studio Owner', role: 'Owner' },
    { name: 'Editor Earn', role: 'Editor' },
    // Demo client account so the Content Studio path can be tried locally
    // (client@studio.local / demo1234), mirroring DEMO_MODE on the server.
    { name: 'Studio Client', role: 'Client' },
  ],
  pillars: ['Pillar I', 'Pillar II', 'Pillar III'],
  formats: ['Long-video', 'Short-video'],
  goals: ['10K subscribe'],
  statuses: ['Start', 'Design', 'Dev', 'Done'],
  platforms: ['Facebook', 'Instagram', 'TikTok', 'Youtube'],
  showEditableColors: true,
  authRequired: false,
  allowRegistration: true,
  roles: [
    { name: 'Owner', permissions: [...PERMISSIONS] },
    {
      name: 'Editor',
      permissions: [
        'posts.write', 'posts.lock', 'metrics.import', 'ideas.write', 'hashtags.write',
        'content.write', 'content.publish', 'campaigns.write',
      ],
    },
    { name: 'Viewer', permissions: [] },
    // Client-facing role: Content Studio only (write + publish).
    { name: 'Client', permissions: ['content.write', 'content.publish'] },
  ],
}

export const posts: Post[] = [
  {
    id: 'p-morning', month: 2, topic: 'Morning Vlog', pillar: 'Pillar II',
    format: 'Short-video', goal: '10K subscribe', date: '2026-02-03', time: '08:00',
    status: 'Done', hook: 'POV: slow mornings', caption: 'A slow morning routine before the city wakes.',
    cta: 'Save for later', hashtagGroup: 'Reach', hashtags: ['morning', 'vlog', 'slow'],
    imageUrl: '', note: '', done: true, platforms: ['Instagram', 'TikTok'],
  },
  {
    id: 'p-night', month: 2, topic: 'Self-Care Night Routine', pillar: 'Pillar I',
    format: 'Long-video', goal: '10K subscribe', date: '2026-02-05', time: '20:00',
    status: 'Design', hook: 'Unwind with me', caption: 'Full night routine, no cuts.',
    cta: 'Subscribe', hashtagGroup: 'Niche', hashtags: ['selfcare', 'night'],
    imageUrl: '', note: 'needs thumbnail', done: false, platforms: ['Youtube'],
    lockedBy: 'Editor Earn',
  },
  {
    id: 'p-desk', month: 2, topic: 'Desk Setup Tour', pillar: 'Pillar III',
    format: 'Short-video', goal: '10K subscribe', date: '2026-02-06', time: '12:00',
    status: 'Start', hook: 'My 2026 desk', caption: 'Everything on my desk and why.',
    cta: 'Comment yours', hashtagGroup: 'Reach', hashtags: ['desksetup'],
    imageUrl: '', note: '', done: false, platforms: ['TikTok', 'Instagram'],
  },
  {
    id: 'p-budget', month: 2, topic: 'Budget Breakfast', pillar: 'Pillar I',
    format: 'Short-video', goal: '10K subscribe', date: '2026-02-10', time: '07:30',
    status: 'Dev', hook: 'Eat for under $2', caption: 'Three cheap breakfasts that slap.',
    cta: 'Share this', hashtagGroup: 'Niche', hashtags: ['budget', 'food'],
    imageUrl: '', note: '', done: false, platforms: ['Facebook', 'Instagram'],
  },
  {
    id: 'p-feed', month: 2, topic: 'Feed Aesthetic Tips', pillar: 'Pillar II',
    format: 'Long-video', goal: '10K subscribe', date: '2026-02-12', time: '18:00',
    status: 'Start', hook: 'Fix your grid', caption: 'Five rules for a coherent feed.',
    cta: 'Follow', hashtagGroup: 'Branded', hashtags: ['feed', 'aesthetic'],
    imageUrl: '', note: '', done: false, platforms: ['Instagram'],
  },
  {
    id: 'p-march', month: 3, topic: 'March Teaser', pillar: 'Pillar I',
    format: 'Short-video', goal: '10K subscribe', date: '2026-03-02', time: '09:00',
    status: 'Start', hook: 'Coming soon', caption: 'What March looks like.',
    cta: 'Stay tuned', hashtagGroup: 'Event', hashtags: ['march'],
    imageUrl: '', note: '', done: false, platforms: ['TikTok'],
  },
]

export const campaigns: Campaign[] = [
  {
    id: 'cmp-spring', name: 'Spring Launch',
    objective: 'Launch the spring collection with a 4-week content sprint.',
    status: 'active', startDate: '2026-03-01', endDate: '2026-03-31',
    platforms: ['Instagram', 'TikTok'], pillars: ['Pillar I', 'Pillar II'],
    hashtags: ['spring', 'launch'], budget: 15000, goalMetric: 'views', goalTarget: 250000,
    owner: 'Studio Owner', notes: 'Two posts per week plus one story series.',
    contentIds: ['c-march'], createdAt: '2026-02-20 09:00', updatedAt: '2026-02-20 09:00',
  },
  {
    id: 'cmp-evergreen', name: 'Evergreen Growth',
    objective: 'Keep the back catalogue working with monthly refreshes.',
    status: 'draft', startDate: '2026-04-01', endDate: '2026-06-30',
    platforms: ['Youtube'], pillars: ['Pillar III'], hashtags: ['evergreen'],
    budget: 0, goalMetric: 'likes', goalTarget: 5000,
    owner: 'Studio Owner', notes: '', contentIds: [],
    createdAt: '2026-03-01 10:00', updatedAt: '2026-03-01 10:00',
  },
]

export const ideas: Idea[] = [
  { id: 'i-1', topic: 'Rainy Day Reads', format: 'Long-video', idea: 'Cozy reading vlog', link: '', done: false },
  { id: 'i-2', topic: 'Camera Roll Dump', format: 'Short-video', idea: 'Weekly photo dump', link: '', done: false },
  { id: 'i-3', topic: 'Q&A Sunday', format: 'Long-video', idea: 'Answer comments on camera', link: '', done: true },
]

export const hashtagGroups: HashtagGroup[] = [
  { id: 'g-niche', title: 'Niche', tags: ['selfcare', 'budget', 'desksetup'] },
  { id: 'g-reach', title: 'Reach', tags: ['morning', 'vlog', 'slow'] },
  { id: 'g-branded', title: 'Branded', tags: ['feed', 'aesthetic'] },
  { id: 'g-event', title: 'Event', tags: ['march'] },
]

export const metrics: Metric[] = [
  { postId: 'p-morning', platform: 'Instagram', likes: 1200, views: 15000 },
  { postId: 'p-morning', platform: 'TikTok', likes: 3400, views: 42000 },
  { postId: 'p-night', platform: 'Youtube', likes: 300, views: 5200 },
]

export const txns: Txn[] = [
  { id: 't-1', date: '2026-01-05', amount: 15000, kind: 'IN', category: 'Sponsorship', sub: 'Brand A' },
  { id: 't-2', date: '2026-01-12', amount: 3200, kind: 'OUT', category: 'Gear', sub: 'Mic' },
  { id: 't-3', date: '2026-02-03', amount: 15000, kind: 'IN', category: 'Sponsorship', sub: 'Brand A' },
  { id: 't-4', date: '2026-02-09', amount: 1500, kind: 'OUT', category: 'Props', sub: 'Set decor' },
  { id: 't-5', date: '2026-02-15', amount: 4800, kind: 'OUT', category: 'Editing', sub: 'Freelancer' },
  { id: 't-6', date: '2026-02-20', amount: 2200, kind: 'IN', category: 'Affiliate', sub: 'Links' },
]

export const brand: Brand = {
  channel: 'Studio Channel',
  positioning: 'Slow living, honest reviews',
  slogan: 'Make room for slow',
  audience: '20–34, city creatives',
  voice: 'Warm, direct, no hype',
  dos: ['Show the process', 'Credit sources'],
  donts: ['No clickbait', 'No fake urgency'],
  palette: ['#111111', '#555555', '#999999', '#CCCCCC'],
  fonts: ['Noto Sans Thai', 'DejaVu Sans'],
  logos: [],
  moodboard: [],
  radius: 12,
  fillOpacity: 100,
  strokeWidth: 1,
  shadow: 'soft',
}

export const platformGoals: PlatformGoal[] = [
  { platform: 'Facebook', start: 1200, goal: 5000 },
  { platform: 'Instagram', start: 3400, goal: 10000 },
  { platform: 'TikTok', start: 2100, goal: 10000 },
  { platform: 'Youtube', start: 800, goal: 5000 },
]

// ---- workspaces ----
/** A separate brand space; each owns its own connections and brand identity. */
export interface WorkspaceSummary {
  id: string
  name: string
  created: string
  /** Platforms currently connected in this workspace. */
  connected: number
  /** Platform slots the workspace has (usually 4). */
  total: number
  /** True when the signed-in account owns this workspace. */
  isOwner: boolean
}

/** One account on a workspace's member list (owner included). */
export interface WorkspaceMember {
  email: string
  name: string
  role: string
}

/** The owner plus the members the owner added, as the members page shows them. */
export interface WorkspaceMembers {
  owner: WorkspaceMember
  members: WorkspaceMember[]
}

/** Mock-only workspace record: adds the ownership data the server keeps. */
export interface MockWorkspace {
  id: string
  name: string
  created: string
  connected: number
  total: number
  /** Account email that owns this workspace. */
  owner: string
  /** Emails of the non-owner members the owner added. */
  members: string[]
}

export const workspaces: MockWorkspace[] = [
  {
    id: 'ws-default', name: 'workspace', created: '2026-01-01T00:00:00Z', connected: 1, total: 3,
    owner: 'thontrapoowadol@example.com',
    members: ['editor@studio.local', 'client@studio.local'],
  },
]

// ---- platform connections (wireframe OAuth state) ----
// One Meta login covers the Facebook Page and its linked Instagram account.
export type PlatformId = 'meta' | 'youtube' | 'tiktok'
export type ConnectStatus = 'disconnected' | 'connecting' | 'connected' | 'error'

export interface PlatformConnection {
  id: PlatformId
  status: ConnectStatus
  handle: string
  externalId: string
  scopes: string[]
  tokenType: 'short-lived' | 'long-lived' | 'refresh' | '—'
  expiresAt: string | null
  lastSync: string | null
  mediaCount: number
  note: string
  /** Runtime-only: a usable provider token exists (server list responses).
   *  `false` on a connected slot means the account must reconnect to sync. */
  hasToken?: boolean
}

export const connections: PlatformConnection[] = [
  {
    id: 'meta', status: 'connected',
    handle: 'https://www.facebook.com/profile.php?id=102400000000001',
    externalId: '102400000000001',
    scopes: [
      'pages_show_list', 'pages_read_engagement', 'pages_manage_posts',
      'read_insights', 'business_management',
      'instagram_basic', 'instagram_manage_insights',
    ],
    tokenType: 'long-lived', expiresAt: '2026-10-18', lastSync: '2026-03-01 06:00',
    mediaCount: 214, note: 'Facebook Page + linked Instagram · token refresh due day 45',
    hasToken: true,
  },
  {
    id: 'youtube', status: 'disconnected', handle: '', externalId: '',
    scopes: [], tokenType: '—', expiresAt: null, lastSync: null,
    mediaCount: 0, note: 'Quota 10,000 units/day',
  },
  {
    id: 'tiktok', status: 'disconnected', handle: '', externalId: '',
    scopes: [], tokenType: '—', expiresAt: null, lastSync: null,
    mediaCount: 0, note: 'Audited app required to publish',
  },
]

// ---- live mirror of the connected platforms (synced from the server) ----
export interface LivePost {
  id: string
  /** Connection that produced it: "meta" (Page) or "instagram" (linked IG). */
  platform: string
  kind: string
  caption: string
  mediaUrl: string
  thumbnailUrl: string
  permalink: string
  createdAt: string
  likes: number
  comments: number
  shares: number
  views: number
  /** Reaction counts by type (like, love, haha, wow, sad, angry). */
  reactions: Record<string, number>
  /** Accounts reached (Instagram reports it; Facebook deprecated it). */
  reach: number
  /** Saves (Instagram reports it; 0 elsewhere). */
  saves: number
  /** Total clicks (link clicks, photo views, ...). */
  clicks: number
  /** Link clicks alone (a subset of `clicks`). */
  linkClicks: number
  /** Video length and average watch time in seconds (videos only). */
  videoLength: number
  videoAvgWatchTime: number
  /** Attachment title and destination (link posts carry the target URL). */
  attachmentTitle: string
  linkUrl: string
  /** Unix seconds of the refresh that captured it. */
  fetchedAt: number
}

export interface LiveAccount {
  platform: string
  id: string
  username: string
  followers: number
  posts: number
  /** Page-level insights (Meta reports 28-day/lifetime windows). */
  engagements: number
  /** Net new follows over the reported window (can be negative). */
  netFollows: number
  pageViews: number
  videoViews: number
}

export interface LiveData {
  /** Unix seconds of the last successful refresh (null = never). */
  fetchedAt: number | null
  accounts: LiveAccount[]
  posts: LivePost[]
  /** Last refresh failure, shown so stale content is explained. */
  error: string
}

/** Demo mirror: fixed stamp so snapshots/tests stay deterministic. */
export const liveFetchedAt = 1772340000
export const live: LiveData = {
  fetchedAt: liveFetchedAt,
  accounts: [
    {
      platform: 'meta', id: '102400000000001', username: 'Studio Channel',
      followers: 4820, posts: 214,
      engagements: 312, netFollows: 46, pageViews: 1240, videoViews: 5400,
    },
    {
      platform: 'instagram', id: '17841400000000001', username: 'studio.channel',
      followers: 3140, posts: 128,
      engagements: 0, netFollows: 0, pageViews: 0, videoViews: 0,
    },
  ],
  posts: [
    {
      id: '102400000000001_9001', platform: 'meta', kind: 'photo',
      caption: 'Behind the scenes of the spring shoot.', mediaUrl: '', thumbnailUrl: '',
      permalink: 'https://www.facebook.com/102400000000001_9001',
      createdAt: '2026-03-01T06:00:00+0000', likes: 214, comments: 18, shares: 6, views: 0,
      reactions: { like: 186, love: 24, wow: 4 }, reach: 0, saves: 0,
      clicks: 62, linkClicks: 21, videoLength: 0, videoAvgWatchTime: 0,
      attachmentTitle: '', linkUrl: '',
      fetchedAt: liveFetchedAt,
    },
    {
      id: '102400000000001_9002', platform: 'meta', kind: 'video',
      caption: '60-second studio tour.', mediaUrl: '', thumbnailUrl: '',
      permalink: 'https://www.facebook.com/102400000000001_9002',
      createdAt: '2026-02-26T09:30:00+0000', likes: 132, comments: 9, shares: 4, views: 5400,
      reactions: { like: 118, love: 11 }, reach: 0, saves: 0,
      clicks: 48, linkClicks: 12, videoLength: 60, videoAvgWatchTime: 18.4,
      attachmentTitle: '60-second studio tour', linkUrl: '',
      fetchedAt: liveFetchedAt,
    },
    {
      id: '17841400000000001_7001', platform: 'instagram', kind: 'reel',
      caption: 'Slow living, honest reviews.', mediaUrl: '', thumbnailUrl: '',
      permalink: 'https://www.instagram.com/reel/7001/',
      createdAt: '2026-02-24T12:00:00+0000', likes: 96, comments: 7, shares: 0, views: 3100,
      reactions: {}, reach: 2600, saves: 31, clicks: 0, linkClicks: 0,
      videoLength: 22, videoAvgWatchTime: 9.1,
      attachmentTitle: '', linkUrl: '',
      fetchedAt: liveFetchedAt,
    },
  ],
  error: '',
}

// ---- Meta Ads mirror (campaigns, settings, performance) ----
// Names mirror the server's `Ad*` models; `Campaign` above is the content
// planner's campaign, these are the platform's advertising objects.

export interface AdAccount {
  /** `act_<id>` — the id Graph ads endpoints use. */
  id: string
  name: string
  /** active | disabled | unsettled | ... */
  status: string
  currency: string
  timezone: string
  business: string
}

export interface AdCampaign {
  id: string
  accountId: string
  name: string
  /** Graph objective, e.g. OUTCOME_TRAFFIC. */
  objective: string
  /** ACTIVE | PAUSED | DELETED | ARCHIVED */
  status: string
  effectiveStatus: string
  buyingType: string
  /** Minor currency units (50000 = 500.00). */
  dailyBudget: number
  lifetimeBudget: number
  budgetRemaining: number
  bidStrategy: string
  specialAdCategories: string[]
  startTime: string
  stopTime: string
  createdTime: string
}

export interface AdSet {
  id: string
  campaignId: string
  name: string
  status: string
  effectiveStatus: string
  dailyBudget: number
  lifetimeBudget: number
  optimizationGoal: string
  billingEvent: string
  bidAmount: number
  startTime: string
  endTime: string
  /** Short human summary (age/gender/geo/interests). */
  targeting: string
  promotedObject: string
}

export interface Ad {
  id: string
  adsetId: string
  name: string
  status: string
  effectiveStatus: string
  creativeTitle: string
  creativeBody: string
  imageUrl: string
  thumbnailUrl: string
  /** Page post this ad promotes (matches `LivePost.id`). */
  storyId: string
  previewUrl: string
}

export interface AdInsight {
  campaignId: string
  spend: number
  impressions: number
  reach: number
  frequency: number
  clicks: number
  ctr: number
  cpc: number
  cpm: number
  results: number
  resultLabel: string
  roas: number
}

export interface AdsAudit {
  at: string
  actor: string
  action: string
  target: string
  detail: string
}

export interface AdsData {
  /** Unix seconds of the last successful refresh (null = never). */
  fetchedAt: number | null
  accounts: AdAccount[]
  campaigns: AdCampaign[]
  adsets: AdSet[]
  ads: Ad[]
  insights: AdInsight[]
  /** Newest first; every management action is recorded here. */
  audit: AdsAudit[]
  error: string
}

/** The mirror plus whether this workspace allows campaign management. */
export interface AdsView extends AdsData {
  canManage: boolean
}

/** Demo mirror: fixed stamp so snapshots/tests stay deterministic. */
export const adsFetchedAt = 1772340000
/** Mutable flag so the mock can toggle the management opt-in. */
export const adsFlags = { manage: false }
export const ads: AdsData = {
  fetchedAt: adsFetchedAt,
  accounts: [
    {
      id: 'act_102400000000001', name: 'Studio Channel Ads', status: 'active',
      currency: 'THB', timezone: 'Asia/Bangkok', business: 'Studio Channel',
    },
  ],
  campaigns: [
    {
      id: 'cmp-ads-1', accountId: 'act_102400000000001',
      name: 'Spring Collection — Traffic', objective: 'OUTCOME_TRAFFIC',
      status: 'ACTIVE', effectiveStatus: 'ACTIVE', buyingType: 'AUCTION',
      dailyBudget: 50000, lifetimeBudget: 0, budgetRemaining: 32000,
      bidStrategy: 'LOWEST_COST_WITHOUT_CAP', specialAdCategories: ['NONE'],
      startTime: '2026-02-20T02:00:00+0000', stopTime: '', createdTime: '2026-02-18T04:00:00+0000',
    },
    {
      id: 'cmp-ads-2', accountId: 'act_102400000000001',
      name: 'Reels Engagement — Paused', objective: 'OUTCOME_ENGAGEMENT',
      status: 'PAUSED', effectiveStatus: 'PAUSED', buyingType: 'AUCTION',
      dailyBudget: 0, lifetimeBudget: 300000, budgetRemaining: 120000,
      bidStrategy: 'LOWEST_COST_WITH_BID_CAP', specialAdCategories: ['NONE'],
      startTime: '2026-02-10T02:00:00+0000', stopTime: '', createdTime: '2026-02-08T04:00:00+0000',
    },
  ],
  adsets: [
    {
      id: 'adset-1', campaignId: 'cmp-ads-1', name: 'Bangkok 25-45',
      status: 'ACTIVE', effectiveStatus: 'ACTIVE',
      dailyBudget: 0, lifetimeBudget: 0,
      optimizationGoal: 'LINK_CLICKS', billingEvent: 'IMPRESSIONS', bidAmount: 0,
      startTime: '2026-02-20T02:00:00+0000', endTime: '',
      targeting: '25-45 · TH · 3 interests', promotedObject: 'page 102400000000001',
    },
    {
      id: 'adset-2', campaignId: 'cmp-ads-2', name: 'Reels viewers',
      status: 'PAUSED', effectiveStatus: 'PAUSED',
      dailyBudget: 0, lifetimeBudget: 0,
      optimizationGoal: 'POST_ENGAGEMENT', billingEvent: 'IMPRESSIONS', bidAmount: 0,
      startTime: '', endTime: '',
      targeting: '18-34 · TH', promotedObject: '',
    },
  ],
  ads: [
    {
      id: 'ad-1', adsetId: 'adset-1', name: 'Spring photo — link',
      status: 'ACTIVE', effectiveStatus: 'ACTIVE',
      creativeTitle: 'Spring is here', creativeBody: 'Shop the new collection.',
      imageUrl: '', thumbnailUrl: '', storyId: '102400000000001_9001', previewUrl: '',
    },
    {
      id: 'ad-2', adsetId: 'adset-1', name: 'Studio tour — video',
      status: 'ACTIVE', effectiveStatus: 'ACTIVE',
      creativeTitle: '60-second studio tour', creativeBody: '',
      imageUrl: '', thumbnailUrl: '', storyId: '102400000000001_9002', previewUrl: '',
    },
  ],
  insights: [
    {
      campaignId: 'cmp-ads-1', spend: 1860, impressions: 42000, reach: 31500,
      frequency: 1.33, clicks: 1240, ctr: 2.95, cpc: 1.5, cpm: 44.29,
      results: 1240, resultLabel: 'Link clicks', roas: 0,
    },
    {
      campaignId: 'cmp-ads-2', spend: 640, impressions: 18200, reach: 15100,
      frequency: 1.21, clicks: 410, ctr: 2.25, cpc: 1.56, cpm: 35.16,
      results: 3400, resultLabel: 'Engagements', roas: 0,
    },
  ],
  audit: [],
  error: '',
}

// ---- CMS: content studio ----
export type ContentStatus = 'draft' | 'review' | 'scheduled' | 'published' | 'archived'
export type ContentKind = 'article' | 'page' | 'note'

export interface ContentRevision {
  revision: number
  savedAt: string
  author: string
  note: string
  title: string
  body: string
  excerpt: string
  seoTitle: string
  seoDescription: string
  tags: string[]
  status: ContentStatus
}

export interface Content {
  id: string
  title: string
  slug: string
  kind: ContentKind
  status: ContentStatus
  body: string
  excerpt: string
  heroImageUrl: string
  tags: string[]
  seoTitle: string
  seoDescription: string
  author: string
  createdAt: string
  updatedAt: string
  publishedAt: string | null
  scheduledFor: string | null
  version: number
  revisions: ContentRevision[]
}

/** Lightweight list row (no body/revisions). */
export interface ContentSummary {
  id: string
  title: string
  slug: string
  kind: ContentKind
  status: ContentStatus
  excerpt: string
  heroImageUrl: string
  tags: string[]
  author: string
  updatedAt: string
  publishedAt: string | null
  scheduledFor: string | null
  version: number
  revisionCount: number
  wordCount: number
}

/** Public delivery shape (published content only). */
export interface PublicContent {
  id: string
  title: string
  slug: string
  kind: ContentKind
  body: string
  excerpt: string
  heroImageUrl: string
  tags: string[]
  author: string
  publishedAt: string | null
  updatedAt: string
}

const CONTENT_NOW = Date.now()
const isoAt = (days: number, hours = 0): string =>
  new Date(CONTENT_NOW + days * 86_400_000 + hours * 3_600_000).toISOString().replace(/\.\d{3}Z$/, 'Z')

export const content: Content[] = [
  {
    id: 'c-welcome',
    title: 'Why slow mornings changed our year',
    slug: 'why-slow-mornings-changed-our-year',
    kind: 'article',
    status: 'published',
    body: '# Why slow mornings changed our year\n\nWe started filming before sunrise and learned more about our audience than any analytics dashboard told us.\n\n## What changed\n\n- We plan one meaningful post per day, not five rushed ones\n- The first hour is offline: no notifications, just notes\n- Every caption is written the evening before\n\n> Consistency is a byproduct of a calm routine, not the other way around.\n\nRead more on the [planner](#/planner/2).',
    excerpt: 'Our editorial experiment: one calm hour before the city wakes, and the content it produced.',
    heroImageUrl: '',
    tags: ['routine', 'behind-the-scenes'],
    seoTitle: 'Slow mornings, better content',
    seoDescription: 'A calm morning routine changed how we plan and publish content.',
    author: 'Studio Owner',
    createdAt: isoAt(-12),
    updatedAt: isoAt(-1),
    publishedAt: isoAt(-1),
    scheduledFor: null,
    version: 3,
    revisions: [
      {
        revision: 1, savedAt: isoAt(-2), author: 'Studio Owner', note: 'First draft',
        title: 'Slow mornings', body: 'Draft notes about our slow morning routine.',
        excerpt: 'Draft notes.', seoTitle: '', seoDescription: '',
        tags: ['routine'], status: 'draft',
      },
    ],
  },
  {
    id: 'c-bts',
    title: 'Behind the scenes: the February shoot',
    slug: 'behind-the-scenes-february-shoot',
    kind: 'article',
    status: 'draft',
    body: '## Shot list\n\n1. Kitchen counter reset\n2. Desk before/after\n3. Golden hour close-ups\n\n_Add captions before publishing._',
    excerpt: 'Set photos, mistakes and the gear that survived the day.',
    heroImageUrl: '',
    tags: ['behind-the-scenes'],
    seoTitle: '',
    seoDescription: '',
    author: 'Editor Earn',
    createdAt: isoAt(-1),
    updatedAt: isoAt(0, -6),
    publishedAt: null,
    scheduledFor: null,
    version: 1,
    revisions: [],
  },
  {
    id: 'c-march',
    title: 'March launch announcement',
    slug: 'march-launch-announcement',
    kind: 'page',
    status: 'scheduled',
    body: '**Save the date** — the new spring collection lands March 1.\n\nJoin the newsletter for first access.',
    excerpt: 'The spring collection lands March 1 — here is what to expect.',
    heroImageUrl: '',
    tags: ['launch', 'event'],
    seoTitle: 'March launch',
    seoDescription: 'Spring collection launching March 1.',
    author: 'Studio Owner',
    createdAt: isoAt(-1),
    updatedAt: isoAt(0, -2),
    publishedAt: null,
    scheduledFor: isoAt(7),
    version: 1,
    revisions: [],
  },
]

export function contentSummary(c: Content): ContentSummary {
  return {
    id: c.id,
    title: c.title,
    slug: c.slug,
    kind: c.kind,
    status: c.status,
    excerpt: c.excerpt,
    heroImageUrl: c.heroImageUrl,
    tags: [...c.tags],
    author: c.author,
    updatedAt: c.updatedAt,
    publishedAt: c.publishedAt,
    scheduledFor: c.scheduledFor,
    version: c.version,
    revisionCount: c.revisions.length,
    wordCount: c.body.split(/\s+/).filter(Boolean).length,
  }
}
