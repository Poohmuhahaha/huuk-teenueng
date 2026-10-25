<script setup lang="ts">
// Live mirror of the connected platforms: real posts + engagement pulled from
// Meta by the server (apps/server/src/live.rs) and refreshed on demand with Sync.
// Read-only — the planner stays the place where content is authored.
import { computed, onMounted, onUnmounted, ref } from 'vue'
import { useLive, usePlatforms, useSyncPlatform, usePermission, useAds } from '@/core/queries'
import { beginOAuth } from '@/core/oauth'
import { t } from '@/core/i18n'
import TableFrame from '@/components/tables/TableFrame.vue'

const props = withDefaults(
  defineProps<{ variant?: 'cards' | 'grid' | 'table'; limit?: number }>(),
  { variant: 'cards', limit: 0 },
)

const { data: live, isPending, isError, error: loadError } = useLive()
const { data: connections } = usePlatforms()
const { data: adsView } = useAds()
const sync = useSyncPlatform()
const { can } = usePermission()

const mayManage = computed(() => can('platforms.manage'))
const meta = computed(() => connections.value?.find((c) => c.id === 'meta') ?? null)
const metaConnected = computed(() => meta.value?.status === 'connected')
/** A connected slot without a stored token cannot sync — it needs a reconnect. */
const needsReconnect = computed(() => metaConnected.value && meta.value?.hasToken === false)
/** A refresh failure that a fresh login would fix (expired/legacy token). */
const tokenProblem = computed(() => /reconnect/i.test(live.value?.error ?? ''))
const reconnectError = ref('')
const reconnecting = ref(false)
const metaAccount = computed(
  () => live.value?.accounts.find((a) => a.platform === 'meta') ?? null,
)
const igAccount = computed(
  () => live.value?.accounts.find((a) => a.platform === 'instagram') ?? null,
)
const posts = computed(() => {
  const all = live.value?.posts ?? []
  return props.limit > 0 ? all.slice(0, props.limit) : all
})

/** Ad campaigns promoting each post, memoized so the template does a Map lookup
 * instead of a posts×ads join on every render. */
const boostMap = computed(() => {
  const campaigns = adsView.value?.campaigns ?? []
  const adsets = adsView.value?.adsets ?? []
  const ads = adsView.value?.ads ?? []
  const byPost = new Map<string, Set<string>>()
  for (const a of ads) {
    if (!a.storyId) continue
    const campaignId = adsets.find((s) => s.id === a.adsetId)?.campaignId
    if (!campaignId) continue
    let set = byPost.get(a.storyId)
    if (!set) {
      set = new Set<string>()
      byPost.set(a.storyId, set)
    }
    set.add(campaignId)
  }
  const valid = new Set(campaigns.map((c) => c.id))
  const out = new Map<string, number>()
  for (const [postId, set] of byPost) {
    let n = 0
    for (const id of set) if (valid.has(id)) n++
    out.set(postId, n)
  }
  return out
})

/** Ad campaigns that promote this post (matched by the ad's story id). */
function boostedBy(postId: string): number {
  return boostMap.value.get(postId) ?? 0
}

/** "Like 186 · Love 24" — reaction breakdown, empty when none. */
function reactionLabel(reactions: Record<string, number> | undefined): string {
  if (!reactions) return ''
  return Object.entries(reactions)
    .map(([kind, count]) => `${kind.charAt(0).toUpperCase()}${kind.slice(1)} ${fmt(count)}`)
    .join(' · ')
}

function reactionTotal(reactions: Record<string, number> | undefined): number {
  return Object.values(reactions ?? {}).reduce((sum, n) => sum + n, 0)
}

/** Seconds -> "0:18" for watch time / video length. */
function mmss(seconds: number): string {
  const s = Math.max(0, Math.round(seconds))
  return `${Math.floor(s / 60)}:${String(s % 60).padStart(2, '0')}`
}

function platformLabel(id: string): string {
  return id === 'instagram' ? 'Instagram' : 'Meta'
}

function fmt(n: number | null | undefined): string {
  return (n ?? 0).toLocaleString('en-US')
}

/** Never throws: missing stamps render as an em dash. */
function dateOf(stamp: string | null | undefined): string {
  return String(stamp ?? '').slice(0, 10) || '—'
}

// Ticking clock for the "synced x min ago" label — without it the label is
// frozen at the fetched render time.
const now = ref(Date.now())
let nowTimer: number | undefined
onMounted(() => {
  nowTimer = window.setInterval(() => { now.value = Date.now() }, 30000)
})
onUnmounted(() => window.clearInterval(nowTimer))

/** Relative "synced" label from a Unix-seconds stamp. */
function syncedLabel(stamp: number | null | undefined): string {
  if (!stamp) return t('live.never')
  const secs = Math.max(0, Math.floor(now.value / 1000) - stamp)
  if (secs < 60) return t('live.justNow')
  if (secs < 3600) return `${Math.floor(secs / 60)} ${t('live.minAgo')}`
  if (secs < 86400) return `${Math.floor(secs / 3600)} ${t('live.hourAgo')}`
  return `${Math.floor(secs / 86400)} ${t('live.dayAgo')}`
}

function onSync(): void {
  if (!mayManage.value || sync.isPending.value) return
  sync.mutate('meta')
}

/** One-click re-authorization: the browser goes to Meta and comes back here. */
async function onReconnect(): Promise<void> {
  if (!mayManage.value || reconnecting.value) return
  reconnecting.value = true
  reconnectError.value = ''
  const error = await beginOAuth('meta')
  if (error) reconnectError.value = error
  reconnecting.value = false
}
</script>

<template>
  <section class="mt-6">
    <div class="flex items-center flex-wrap gap-3">
      <h2 class="section-title m-0 text-sm font-bold tracking-widest uppercase text-ink">{{ t('live.title') }}</h2>
      <span class="text-muted text-[13px]">{{ t('live.synced') }} {{ syncedLabel(live?.fetchedAt) }}</span>
      <button class="btn live-sync ml-auto w-auto px-5" :disabled="!mayManage || sync.isPending.value"
        :title="mayManage ? '' : t('auth.noPerm')" @click="onSync">
        {{ sync.isPending.value ? t('live.syncing') : t('live.sync') }}
      </button>
    </div>

    <div v-if="metaAccount || igAccount" class="flex items-center gap-2 flex-wrap my-2 mb-3">
      <span v-if="metaAccount" class="chip">
        Meta · {{ metaAccount.username }} · {{ fmt(metaAccount.followers) }} {{ t('live.followers') }}
        <template v-if="metaAccount.engagements">· {{ fmt(metaAccount.engagements) }} {{ t('live.engagements') }}</template>
        <template v-if="metaAccount.netFollows">· {{ metaAccount.netFollows > 0 ? '+' : '' }}{{ fmt(metaAccount.netFollows) }} {{ t('live.netFollows') }}</template>
        <template v-if="metaAccount.pageViews">· {{ fmt(metaAccount.pageViews) }} {{ t('live.pageViews') }}</template>
      </span>
      <span v-if="igAccount" class="chip">
        Instagram · @{{ igAccount.username }} · {{ fmt(igAccount.followers) }} {{ t('live.followers') }}
        · {{ fmt(igAccount.posts) }} {{ t('live.posts') }}
      </span>
    </div>

    <!-- Independent error blocks: concurrent failures (load + sync + stale)
      must all surface instead of masking each other. -->
    <p v-if="isError" class="card muted" role="alert">
      {{ loadError?.message }}
    </p>
    <p v-if="reconnectError" class="autherr" role="alert">{{ reconnectError }}</p>
    <p v-if="sync.error.value" class="autherr" role="alert">{{ sync.error.value.message }}</p>
    <div v-if="live?.error" class="live-empty flex flex-col items-start gap-2.5" role="alert">
      <p class="muted">{{ t('live.stale') }} — {{ live.error }}</p>
      <button v-if="tokenProblem && mayManage" class="btn" :disabled="reconnecting" @click="onReconnect">
        {{ reconnecting ? t('slogin.connecting') : t('live.reconnect') }}
      </button>
    </div>

    <p v-if="isPending" class="muted">{{ t('common.loading') }}</p>
    <div v-else-if="!posts.length" class="live-empty flex flex-col items-start gap-2.5">
      <p class="muted">
        {{ needsReconnect || tokenProblem ? t('live.reconnectNeeded')
          : metaConnected ? t('live.emptyConnected') : t('live.emptyDisconnected') }}
      </p>
      <button v-if="needsReconnect || tokenProblem" class="btn btn-primary"
        :disabled="!mayManage || reconnecting" :title="mayManage ? '' : t('auth.noPerm')"
        @click="onReconnect">
        {{ reconnecting ? t('slogin.connecting') : t('live.reconnect') }}
      </button>
    </div>

    <!-- card / grid layouts -->
    <div v-else-if="variant !== 'table'" class="grid gap-3 mt-3"
      :class="variant === 'grid' ? 'grid-cols-[repeat(auto-fill,minmax(160px,1fr))]' : 'grid-cols-[repeat(auto-fill,minmax(220px,1fr))]'">
      <article v-for="p in posts" :key="p.id" class="livecard flex flex-col overflow-hidden text-inherit no-underline bg-surface border border-faint rounded-[var(--radius,12px)] hover:border-accent transition">
        <div class="relative aspect-square grid place-items-center bg-wash overflow-hidden">
          <img v-if="p.thumbnailUrl || p.mediaUrl" :src="p.thumbnailUrl || p.mediaUrl"
            :alt="p.caption" loading="lazy" class="w-full h-full object-cover" />
          <span v-else class="text-[40px] font-bold text-muted">{{ (p.caption || platformLabel(p.platform)).slice(0, 1).toUpperCase() }}</span>
          <span class="absolute top-1.5 left-1.5 px-1.5 py-0.5 text-[11px] bg-black/60 text-white rounded-full">{{ platformLabel(p.platform) }} · {{ p.kind }}</span>
          <span v-if="boostedBy(p.id)" class="absolute top-1.5 right-1.5 px-1.5 py-0.5 text-[11px] bg-accent text-white rounded-full">{{ t('live.boosted') }}</span>
        </div>
        <div class="flex flex-col gap-1.5 p-2.5">
          <p class="m-0 text-[13px] line-clamp-2">{{ p.caption || p.attachmentTitle || '—' }}</p>
          <div class="flex flex-wrap gap-2 text-xs text-muted">
            <span>{{ t('live.likes') }} {{ fmt(p.likes) }}</span>
            <span>{{ t('live.comments') }} {{ fmt(p.comments) }}</span>
            <span v-if="p.shares">{{ t('live.shares') }} {{ fmt(p.shares) }}</span>
            <span v-if="p.views">{{ t('live.views') }} {{ fmt(p.views) }}</span>
            <span v-if="p.reach">{{ t('live.reach') }} {{ fmt(p.reach) }}</span>
            <span v-if="p.clicks">{{ t('live.clicks') }} {{ fmt(p.clicks) }}</span>
            <span v-if="p.linkClicks">{{ t('live.linkClicks') }} {{ fmt(p.linkClicks) }}</span>
            <span v-if="p.saves">{{ t('live.saves') }} {{ fmt(p.saves) }}</span>
            <span v-if="p.videoAvgWatchTime">{{ t('live.avgWatch') }} {{ mmss(p.videoAvgWatchTime) }}</span>
          </div>
          <span v-if="reactionLabel(p.reactions)" class="text-xs text-muted">{{ reactionLabel(p.reactions) }}</span>
          <a v-if="p.linkUrl" class="text-xs text-muted" :href="p.linkUrl" target="_blank"
            rel="noopener noreferrer" @click.stop>{{ p.linkUrl }}</a>
          <span class="text-xs text-muted">{{ dateOf(p.createdAt) }}</span>
          <a v-if="p.permalink" class="btn liveopen" :href="p.permalink" target="_blank"
            rel="noopener noreferrer">{{ t('live.open') }}</a>
        </div>
      </article>
    </div>

    <!-- compact table layout -->
    <TableFrame v-else>
      <thead>
          <tr>
            <!-- TODO(i18n): t('live.platform')/t('live.post')/t('live.date') have no
              core/i18n keys — literals until the keys land (t() would render
              the raw key and break the header). -->
            <th>Platform</th><th>Post</th><th>Date</th>
            <th class="num">{{ t('live.likes') }}</th>
            <th class="num">{{ t('live.comments') }}</th>
            <th class="num">{{ t('live.shares') }}</th>
            <th class="num">{{ t('live.views') }}</th>
            <th class="num">{{ t('live.reactions') }}</th>
            <th class="num">{{ t('live.clicks') }}</th>
            <th class="num">{{ t('live.avgWatch') }}</th>
            <th></th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="p in posts" :key="p.id" style="cursor: default;">
            <td>{{ platformLabel(p.platform) }}</td>
            <td class="max-w-[360px] truncate">{{ p.caption || '—' }}</td>
            <td>{{ dateOf(p.createdAt) }}</td>
            <td class="num">{{ fmt(p.likes) }}</td>
            <td class="num">{{ fmt(p.comments) }}</td>
            <td class="num">{{ fmt(p.shares) }}</td>
            <td class="num">{{ fmt(p.views) }}</td>
            <td class="num">{{ fmt(reactionTotal(p.reactions)) }}</td>
            <td class="num">{{ fmt(p.clicks) }}</td>
            <td class="num">{{ p.videoAvgWatchTime ? mmss(p.videoAvgWatchTime) : '—' }}</td>
            <td>
              <a v-if="p.permalink" class="btn" :href="p.permalink" target="_blank"
                rel="noopener noreferrer">{{ t('live.open') }}</a>
            </td>
          </tr>
        </tbody>
    </TableFrame>
  </section>
</template>
