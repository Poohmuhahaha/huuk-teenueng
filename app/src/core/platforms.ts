// Platform connection metadata + the globally selected platform(s).
// Wireframe only: the OAuth pipeline is simulated in PlatformLogin.vue.
//
// Connections: `meta`, `youtube`, `tiktok`. A single Meta login covers both the
// Facebook Page and its linked Instagram account (the Instagram Graph API is
// part of the same platform), so there is no separate Instagram connection.
// Instagram still exists as a *planner* platform ([`FeedPlatformId`]) because
// its content is planned and previewed with its own canvas sizes.
import { computed, ref } from 'vue'
import type { PlatformId } from '@/mock/db'

export type { PlatformId }

/** Platforms the planner can tag/preview; Instagram publishes through Meta. */
export type FeedPlatformId = PlatformId | 'instagram'

export interface FeedSize {
  w: number
  h: number
  ratio: string
}

export interface FeedVariant extends FeedSize {
  label: string
}

export interface PlatformMeta {
  id: PlatformId
  name: string
  setupName: string // matches the Set up platform list
  auth: string
  externalLabel: string
  scopes: string[]
  steps: string[]
  constraint: string
  feed: FeedSize & { variants: FeedVariant[] } // primary = feed default canvas
}

/**
 * Public profile URL for a connected account. Accepts a full URL or an @handle;
 * returns null when the value cannot be linked safely.
 */
const PROFILE_HOSTS: Record<FeedPlatformId, string[]> = {
  linkedin: ['linkedin.com', 'www.linkedin.com'],
  threads: ['threads.net', 'www.threads.net'],
  pinterest: ['pinterest.com', 'www.pinterest.com'],
  bluesky: ['bsky.app'],
  youtube: ['youtube.com', 'www.youtube.com', 'youtu.be'],
  x: ['x.com', 'twitter.com', 'www.x.com', 'www.twitter.com'],
  google_business: ['business.google.com', 'google.com', 'www.google.com', 'maps.google.com'],
  instagram: ['instagram.com', 'www.instagram.com'],
  substack: ['substack.com'],
  mastodon: ['mastodon.social'],
  tiktok: ['tiktok.com', 'www.tiktok.com'],
  meta: ['facebook.com', 'www.facebook.com', 'm.facebook.com'],
}

/** True when the URL points at an actual profile, not the platform home page. */
function isProfileUrl(id: FeedPlatformId, url: URL): boolean {
  if (url.protocol !== 'https:' && url.protocol !== 'http:') return false
  const host = url.hostname.toLowerCase()
  if (id === 'substack') {
    if (!host.endsWith('.substack.com') && host !== 'substack.com') return false
  } else if (id === 'mastodon') {
    // any fediverse host is allowed
  } else {
    const allowed = PROFILE_HOSTS[id] ?? []
    if (!allowed.some((h) => host === h || host.endsWith('.' + h))) return false
  }
  const path = url.pathname.replace(/^\/+|\/+$/g, '')
  if (!path && id !== 'google_business') return false
  const ignored = new Set(['', 'index.html'])
  return !ignored.has(path.toLowerCase())
}

export function profileUrl(id: FeedPlatformId, handle: string | null | undefined): string | null {
  const raw = (handle ?? '').trim()
  if (!raw) return null
  if (/^https?:\/\//i.test(raw)) {
    try {
      const url = new URL(raw)
      return isProfileUrl(id, url) ? url.toString() : null
    } catch {
      return null
    }
  }
  const clean = raw.replace(/^@/, '')
  if (!clean) return null
  if (id === 'mastodon') {
    if (!/^@?[a-zA-Z0-9_.-]+(@[a-zA-Z0-9_.-]+)?$/.test(raw)) return null
  } else if (id === 'linkedin') {
    if (!/^[\w./-]{1,60}$/.test(clean)) return null
  } else if (id === 'google_business') {
    // google business handle can be empty or ID
  } else {
    if (!/^[\w.-]{1,60}$/.test(clean)) return null
  }
  switch (id) {
    case 'linkedin':
      return clean.startsWith('in/') || clean.startsWith('company/')
        ? `https://linkedin.com/${clean}`
        : `https://linkedin.com/company/${clean}`
    case 'threads':
      return `https://threads.net/@${clean}`
    case 'pinterest':
      return `https://pinterest.com/${clean}`
    case 'bluesky':
      return `https://bsky.app/profile/${clean}`
    case 'youtube':
      return `https://youtube.com/@${clean}`
    case 'x':
      return `https://x.com/${clean}`
    case 'google_business':
      return `https://business.google.com`
    case 'instagram':
      return `https://instagram.com/${clean}`
    case 'substack':
      return `https://${clean}.substack.com`
    case 'mastodon':
      return clean.includes('/') || clean.includes('@')
        ? (clean.startsWith('http') ? clean : `https://${clean.replace(/^@/, '')}`)
        : `https://mastodon.social/@${clean}`
    case 'tiktok':
      return `https://tiktok.com/@${clean}`
    case 'meta':
      return `https://facebook.com/${clean}`
  }
}

/** Validates a profile URL/handle before it is stored on a connection. */
export function validProfileHandle(id: FeedPlatformId, handle: string): boolean {
  const raw = handle.trim()
  if (!raw) return false
  if (/^https?:\/\//i.test(raw)) {
    try {
      return isProfileUrl(id, new URL(raw))
    } catch {
      return false
    }
  }
  if (id === 'mastodon') {
    return /^@?[a-zA-Z0-9_.-]+(@[a-zA-Z0-9_.-]+)?$/.test(raw)
  }
  return /^@?[\w.-]{1,60}$/.test(raw)
}

export const PLATFORM_META: Record<PlatformId, PlatformMeta> = {
  linkedin: {
    id: 'linkedin',
    name: 'LinkedIn',
    setupName: 'LinkedIn',
    auth: 'OAuth 2.0 · LinkedIn REST API',
    externalLabel: 'Organization ID',
    scopes: ['r_liteprofile', 'w_member_social', 'r_organization_social', 'w_organization_social'],
    steps: [
      'Redirect → LinkedIn OAuth dialog',
      'Exchange code → member access token',
      'GET /v2/organizationAcls → manage pages',
      'Store organization URN for publishing',
    ],
    constraint: 'Community Management API permissions required for organization brand publishing.',
    feed: {
      w: 1200,
      h: 627,
      ratio: '1.91:1',
      variants: [{ w: 1080, h: 1080, ratio: '1:1', label: 'Square' }],
    },
  },
  threads: {
    id: 'threads',
    name: 'Threads',
    setupName: 'Threads',
    auth: 'OAuth 2.0 · Threads API',
    externalLabel: 'Threads User ID',
    scopes: ['threads_basic', 'threads_content_publish', 'threads_read_replies'],
    steps: [
      'Redirect → Threads OAuth authorization',
      'Exchange short-lived code → long-lived user token',
      'GET /v1.0/me → Threads user ID & username',
      'Schedule periodic text/media publishing container',
    ],
    constraint: 'Container-based publishing: create media container then publish within 24h.',
    feed: {
      w: 1080,
      h: 1080,
      ratio: '1:1',
      variants: [{ w: 1080, h: 1350, ratio: '4:5', label: 'Portrait' }],
    },
  },
  pinterest: {
    id: 'pinterest',
    name: 'Pinterest',
    setupName: 'Pinterest',
    auth: 'OAuth 2.0 · Pinterest API v5',
    externalLabel: 'Board / Profile ID',
    scopes: ['boards:read', 'pins:read', 'pins:write', 'user_accounts:read'],
    steps: [
      'Redirect → Pinterest OAuth consent dialog',
      'Exchange code → user token & refresh token',
      'GET /v5/user_account → account overview',
      'GET /v5/boards → boards list for pin targeting',
    ],
    constraint: 'Pins require valid target board and public destination link.',
    feed: {
      w: 1000,
      h: 1500,
      ratio: '2:3',
      variants: [{ w: 1000, h: 1000, ratio: '1:1', label: 'Square' }],
    },
  },
  bluesky: {
    id: 'bluesky',
    name: 'Bluesky',
    setupName: 'Bluesky',
    auth: 'AT Protocol · OAuth / App Password',
    externalLabel: 'DID (did:plc:…)',
    scopes: ['atproto', 'com.atproto.repo.createRecord'],
    steps: [
      'Resolve handle → DID identity document',
      'Create XRPC session via com.atproto.server.createSession',
      'Verify Lexicon schema for app.bsky.feed.post',
      'Store session accessJwt and refreshJwt',
    ],
    constraint: 'Open federated protocol: records are signed and cryptographically verifiable.',
    feed: {
      w: 1200,
      h: 675,
      ratio: '16:9',
      variants: [{ w: 1080, h: 1080, ratio: '1:1', label: 'Square' }],
    },
  },
  youtube: {
    id: 'youtube',
    name: 'YouTube',
    setupName: 'Youtube',
    auth: 'OAuth 2.0 · Google Identity',
    externalLabel: 'Channel ID',
    scopes: ['youtube.readonly', 'yt-analytics.readonly'],
    steps: [
      'Redirect → Google consent screen (access_type=offline)',
      'Exchange code → access token + refresh token',
      'GET /youtube/v3/channels?mine=true → Channel ID',
      'Store refresh token · refresh access token when it expires',
      'Schedule analytics worker (YouTube Analytics API)',
    ],
    constraint: 'Data API quota 10,000 units/day per project — queue bulk video fetches.',
    feed: {
      w: 1280,
      h: 720,
      ratio: '16:9',
      variants: [{ w: 1080, h: 1920, ratio: '9:16', label: 'Shorts' }],
    },
  },
  x: {
    id: 'x',
    name: 'X',
    setupName: 'X',
    auth: 'OAuth 2.0 · X Developer API v2',
    externalLabel: 'Account ID',
    scopes: ['tweet.read', 'tweet.write', 'users.read', 'offline.access'],
    steps: [
      'Redirect → X OAuth 2.0 PKCE consent authorization',
      'Exchange authorization code → user token & refresh token',
      'GET /2/users/me → verify account handle and ID',
      'Post tweets and media attachments via v2 endpoint',
    ],
    constraint: 'Free/Basic tier rate limits apply for tweet posting and user metrics.',
    feed: {
      w: 1200,
      h: 675,
      ratio: '16:9',
      variants: [{ w: 1080, h: 1080, ratio: '1:1', label: 'Square' }],
    },
  },
  google_business: {
    id: 'google_business',
    name: 'Google Business',
    setupName: 'Google Business',
    auth: 'OAuth 2.0 · Google Business Profile API',
    externalLabel: 'Location ID',
    scopes: ['https://www.googleapis.com/auth/business.manage'],
    steps: [
      'Redirect → Google OAuth authorization for Business Profile',
      'Exchange code → offline access & refresh token',
      'GET /v4/accounts/{id}/locations → verified locations',
      'Publish business updates, offers, and local photos',
    ],
    constraint: 'Verification required by Google before local posts appear publicly on Maps.',
    feed: {
      w: 1024,
      h: 768,
      ratio: '4:3',
      variants: [{ w: 720, h: 720, ratio: '1:1', label: 'Square' }],
    },
  },
  instagram: {
    id: 'instagram',
    name: 'Instagram',
    setupName: 'Instagram',
    auth: 'OAuth 2.0 · Instagram Graph API',
    externalLabel: 'IG Business ID',
    scopes: ['instagram_basic', 'instagram_content_publish', 'instagram_manage_insights'],
    steps: [
      'Redirect → Meta / Instagram OAuth consent dialog',
      'Exchange short-lived token → 60-day long-lived token',
      'GET /me/accounts → locate connected Instagram Business Account',
      'Schedule media upload and direct publishing containers',
    ],
    constraint: 'Business or Creator account connected to a Facebook Page is required.',
    feed: {
      w: 1080,
      h: 1080,
      ratio: '1:1',
      variants: [
        { w: 1080, h: 1350, ratio: '4:5', label: 'Portrait' },
        { w: 1080, h: 1920, ratio: '9:16', label: 'Story/Reel' },
      ],
    },
  },
  substack: {
    id: 'substack',
    name: 'Substack',
    setupName: 'Substack',
    auth: 'API · Publication Integration',
    externalLabel: 'Publication Subdomain',
    scopes: ['publication:read', 'publication:draft', 'subscribers:read'],
    steps: [
      'Authorize Substack publication integration key',
      'Verify publication subdomain and newsletter feeds',
      'Create draft newsletter editions and publication notes',
      'Sync engagement and subscriber analytics',
    ],
    constraint: 'Draft staging mode; public dispatch requires editorial sign-off.',
    feed: {
      w: 1200,
      h: 630,
      ratio: '1.91:1',
      variants: [{ w: 1200, h: 400, ratio: '3:1', label: 'Banner' }],
    },
  },
  mastodon: {
    id: 'mastodon',
    name: 'Mastodon',
    setupName: 'Mastodon',
    auth: 'OAuth 2.0 · Mastodon Fediverse API',
    externalLabel: 'Fediverse Account ID',
    scopes: ['read', 'write:statuses', 'write:media'],
    steps: [
      'Discover instance endpoint (POST /api/v1/apps)',
      'Redirect → instance authorization endpoint with client ID',
      'Exchange authorization code → instance bearer token',
      'GET /api/v1/accounts/verify_credentials → profile',
    ],
    constraint: 'Independent server instance rate limits and federation rules apply.',
    feed: {
      w: 1200,
      h: 675,
      ratio: '16:9',
      variants: [{ w: 1080, h: 1080, ratio: '1:1', label: 'Square' }],
    },
  },
  tiktok: {
    id: 'tiktok',
    name: 'TikTok',
    setupName: 'TikTok',
    auth: 'OAuth 2.0 · TikTok for Developers',
    externalLabel: 'open_id',
    scopes: ['user.info.basic', 'video.list'],
    steps: [
      'Redirect → TikTok authorize (client_key, scope)',
      'Exchange code → access token + refresh token',
      'GET /v2/user/info/ → open_id + profile',
      'Store refresh token (~365 days) · rotate on use',
      'Schedule video list + metrics sync',
    ],
    constraint: 'Content Posting API requires an audited app before public publishing.',
    feed: {
      w: 1080,
      h: 1920,
      ratio: '9:16',
      variants: [{ w: 1080, h: 1080, ratio: '1:1', label: 'Square' }],
    },
  },
  meta: {
    id: 'meta',
    name: 'Meta',
    setupName: 'Meta',
    auth: 'OAuth 2.0 · Meta Graph API',
    externalLabel: 'Page ID',
    scopes: [
      'pages_show_list',
      'pages_read_engagement',
      'pages_manage_posts',
      'read_insights',
      'business_management',
      'instagram_basic',
      'instagram_manage_insights',
    ],
    steps: [
      'Redirect → Meta OAuth dialog (dialog/oauth?client_id=…&scope=…)',
      'Exchange code → short-lived user token (~1 hour)',
      'Exchange fb_exchange_token → long-lived token (~60 days)',
      'GET /me/accounts → Page list (Business pages via /me/assigned_pages)',
      'Link the Page profile URL as the connection handle',
      'Store Page Access Token for offline publishing + insights',
    ],
    constraint:
      'Page publishing needs pages_manage_posts; insights need read_insights. Instagram data comes from the linked account on the same Page.',
    feed: {
      w: 1200,
      h: 630,
      ratio: '1.91:1',
      variants: [{ w: 1080, h: 1080, ratio: '1:1', label: 'Square' }],
    },
  },
}

export const PLATFORMS: PlatformMeta[] = Object.values(PLATFORM_META)

export const FEED_META: Record<FeedPlatformId, { name: string; feed: PlatformMeta['feed'] }> = {
  linkedin: { name: 'LinkedIn', feed: PLATFORM_META.linkedin.feed },
  threads: { name: 'Threads', feed: PLATFORM_META.threads.feed },
  pinterest: { name: 'Pinterest', feed: PLATFORM_META.pinterest.feed },
  bluesky: { name: 'Bluesky', feed: PLATFORM_META.bluesky.feed },
  youtube: { name: 'YouTube', feed: PLATFORM_META.youtube.feed },
  x: { name: 'X', feed: PLATFORM_META.x.feed },
  google_business: { name: 'Google Business', feed: PLATFORM_META.google_business.feed },
  instagram: { name: 'Instagram', feed: PLATFORM_META.instagram.feed },
  substack: { name: 'Substack', feed: PLATFORM_META.substack.feed },
  mastodon: { name: 'Mastodon', feed: PLATFORM_META.mastodon.feed },
  tiktok: { name: 'TikTok', feed: PLATFORM_META.tiktok.feed },
  meta: { name: 'Meta', feed: PLATFORM_META.meta.feed },
}

// Resolve a Set up platform name back to its planner id.
export function platformIdBySetupName(name: string): FeedPlatformId | undefined {
  const n = name.trim().toLowerCase()
  if (n === 'facebook') return 'meta'
  if (n === 'twitter') return 'x'
  if (n === 'google' || n === 'google business' || n === 'gmb') return 'google_business'
  const match = (Object.entries(FEED_META) as [FeedPlatformId, { name: string }][]).find(
    ([, m]) => m.name.toLowerCase() === n,
  )
  return match?.[0]
}

// Checked platforms in the Social Media bar (keep in registry order; min 1).
export const activePlatforms = ref<PlatformId[]>(['meta'])

// Primary platform = first checked one. Pages that need a single platform use this.
export const activePlatform = computed<PlatformId>(
  () => PLATFORMS.find((p) => activePlatforms.value.includes(p.id))?.id ?? 'meta',
)

export function togglePlatform(id: PlatformId): void {
  const cur = activePlatforms.value
  if (cur.includes(id)) {
    if (cur.length === 1) return // never leave zero platforms checked
    activePlatforms.value = cur.filter((x) => x !== id)
  } else {
    const next = new Set([...cur, id])
    activePlatforms.value = PLATFORMS.filter((p) => next.has(p.id)).map((p) => p.id)
  }
}
