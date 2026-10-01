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
  meta: ['facebook.com', 'www.facebook.com', 'm.facebook.com'],
  instagram: ['instagram.com', 'www.instagram.com'],
  youtube: ['youtube.com', 'www.youtube.com', 'youtu.be'],
  tiktok: ['tiktok.com', 'www.tiktok.com'],
}

/** True when the URL points at an actual profile, not the platform home page. */
function isProfileUrl(id: FeedPlatformId, url: URL): boolean {
  if (url.protocol !== 'https:' && url.protocol !== 'http:') return false
  if (!PROFILE_HOSTS[id].includes(url.hostname.toLowerCase())) return false
  // A bare domain (optionally with a locale prefix) is the home page — reject.
  const path = url.pathname.replace(/^\/+|\/+$/g, '')
  if (!path) return false
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
  if (!/^[\w.-]{1,60}$/.test(clean)) return null
  switch (id) {
    case 'meta': return `https://facebook.com/${clean}`
    case 'instagram': return `https://instagram.com/${clean}`
    case 'youtube': return `https://youtube.com/@${clean}`
    case 'tiktok': return `https://tiktok.com/@${clean}`
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
  return /^@?[\w.-]{1,60}$/.test(raw)
}

export const PLATFORM_META: Record<PlatformId, PlatformMeta> = {
  meta: {
    id: 'meta',
    name: 'Meta',
    setupName: 'Meta',
    auth: 'OAuth 2.0 · Meta Graph API',
    externalLabel: 'Page ID',
    // Must match the scopes the server actually requests (server/src/oauth.rs).
    // One login covers the Facebook Page and its linked Instagram account.
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
}

export const PLATFORMS: PlatformMeta[] = Object.values(PLATFORM_META)

// Instagram canvas specs: no separate connection (Meta covers it), but content
// tagged Instagram is still previewed at Instagram sizes.
const INSTAGRAM_FEED: PlatformMeta['feed'] = {
  w: 1080,
  h: 1080,
  ratio: '1:1',
  variants: [
    { w: 1080, h: 1350, ratio: '4:5', label: 'Portrait' },
    { w: 1080, h: 1920, ratio: '9:16', label: 'Story/Reel' },
  ],
}

/** Canvas + label per planner platform (Meta, Instagram, YouTube, TikTok). */
export const FEED_META: Record<FeedPlatformId, { name: string; feed: PlatformMeta['feed'] }> = {
  meta: { name: 'Meta', feed: PLATFORM_META.meta.feed },
  instagram: { name: 'Instagram', feed: INSTAGRAM_FEED },
  youtube: { name: 'YouTube', feed: PLATFORM_META.youtube.feed },
  tiktok: { name: 'TikTok', feed: PLATFORM_META.tiktok.feed },
}

// Resolve a Set up platform name back to its planner id. "Facebook" is a legacy
// label for the same Meta login, so both map to `meta`.
export function platformIdBySetupName(name: string): FeedPlatformId | undefined {
  const n = name.trim().toLowerCase()
  if (n === 'facebook') return 'meta'
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
