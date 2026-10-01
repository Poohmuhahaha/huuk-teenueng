<script setup lang="ts">
// Live mirror of the connected platforms: real posts + engagement pulled from
// Meta by the server (apps/server/src/live.rs) and refreshed on demand with Sync.
// Read-only — the planner stays the place where content is authored.
import { computed, ref } from 'vue'
import { useLive, usePlatforms, useSyncPlatform, usePermission, useAds } from '@/core/queries'
import { beginOAuth } from '@/core/oauth'
import { t } from '@/core/i18n'

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

/** Ad campaigns that promote this post (matched by the ad's story id). */
function boostedBy(postId: string): number {
  const campaigns = adsView.value?.campaigns ?? []
  const adsets = adsView.value?.adsets ?? []
  const ads = adsView.value?.ads ?? []
  const ids = new Set(
    ads.filter((a) => a.storyId === postId).map((a) => adsets.find((s) => s.id === a.adsetId)?.campaignId),
  )
  return campaigns.filter((c) => ids.has(c.id)).length
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

function fmt(n: number): string {
  return n.toLocaleString('en-US')
}

/** Relative "synced" label from a Unix-seconds stamp. */
function syncedLabel(stamp: number | null | undefined): string {
  if (!stamp) return t('live.never')
  const secs = Math.max(0, Math.floor(Date.now() / 1000) - stamp)
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
  <section class="livesec">
    <div class="row live-head">
      <h2 style="margin: 0;">{{ t('live.title') }}</h2>
      <span class="muted live-stamp">{{ t('live.synced') }} {{ syncedLabel(live?.fetchedAt) }}</span>
      <button class="btn live-sync" :disabled="!mayManage || sync.isPending.value"
        :title="mayManage ? '' : t('auth.noPerm')" @click="onSync">
        {{ sync.isPending.value ? t('live.syncing') : t('live.sync') }}
      </button>
    </div>

    <div v-if="metaAccount || igAccount" class="row live-accounts">
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

    <p v-if="isError" class="card muted">
      {{ loadError?.message }}
    </p>
    <p v-else-if="reconnectError" class="autherr" role="alert">{{ reconnectError }}</p>
    <p v-else-if="sync.error.value" class="autherr" role="alert">{{ sync.error.value.message }}</p>
    <div v-else-if="live?.error" class="live-empty">
      <p class="muted">{{ t('live.stale') }} — {{ live.error }}</p>
      <button v-if="tokenProblem && mayManage" class="btn" :disabled="reconnecting" @click="onReconnect">
        {{ reconnecting ? t('slogin.connecting') : t('live.reconnect') }}
      </button>
    </div>

    <p v-if="isPending" class="muted">{{ t('common.loading') }}</p>
    <div v-else-if="!posts.length" class="live-empty">
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
    <div v-else-if="variant !== 'table'" class="livegrid" :class="`livegrid-${variant}`">
      <a v-for="p in posts" :key="p.id" class="livecard" :href="p.permalink || undefined"
        target="_blank" rel="noopener noreferrer">
        <div class="livemedia">
          <img v-if="p.thumbnailUrl || p.mediaUrl" :src="p.thumbnailUrl || p.mediaUrl"
            :alt="p.caption" loading="lazy" />
          <span v-else class="liveletter">{{ (p.caption || platformLabel(p.platform)).slice(0, 1).toUpperCase() }}</span>
          <span class="livekind">{{ platformLabel(p.platform) }} · {{ p.kind }}</span>
          <span v-if="boostedBy(p.id)" class="liveboosted">{{ t('live.boosted') }}</span>
        </div>
        <div class="livebody">
          <p class="livecaption">{{ p.caption || p.attachmentTitle || '—' }}</p>
          <div class="livestats muted">
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
          <span v-if="reactionLabel(p.reactions)" class="muted live-date">{{ reactionLabel(p.reactions) }}</span>
          <a v-if="p.linkUrl" class="muted live-date" :href="p.linkUrl" target="_blank"
            rel="noopener noreferrer">{{ p.linkUrl }}</a>
          <span class="muted live-date">{{ p.createdAt.slice(0, 10) }}</span>
        </div>
      </a>
    </div>

    <!-- compact table layout -->
    <div v-else class="tblwrap">
      <table class="tbl">
        <thead>
          <tr>
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
            <td class="live-caption">{{ p.caption || '—' }}</td>
            <td>{{ p.createdAt.slice(0, 10) }}</td>
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
      </table>
    </div>
  </section>
</template>

<style scoped>
.livesec {
  margin-top: 24px;
}
.live-head {
  align-items: center;
  gap: 12px;
}
.live-stamp {
  font-size: 13px;
}
.live-sync {
  margin-left: auto;
}
.live-accounts {
  gap: 8px;
  flex-wrap: wrap;
  margin: 8px 0 12px;
}
.livegrid {
  display: grid;
  gap: 12px;
  margin-top: 12px;
}
.livegrid-cards {
  grid-template-columns: repeat(auto-fill, minmax(220px, 1fr));
}
.livegrid-grid {
  grid-template-columns: repeat(auto-fill, minmax(160px, 1fr));
}
.livecard {
  display: flex;
  flex-direction: column;
  overflow: hidden;
  color: inherit;
  text-decoration: none;
  background: var(--surface, #fff);
  border: 1px solid var(--faint, #e6e6e6);
  border-radius: var(--radius, 12px);
}
.livecard:hover {
  border-color: var(--accent, #0b7a75);
}
.livemedia {
  position: relative;
  aspect-ratio: 1 / 1;
  display: grid;
  place-items: center;
  background: var(--wash, #f2f3f5);
  overflow: hidden;
}
.livemedia img {
  width: 100%;
  height: 100%;
  object-fit: cover;
}
.liveletter {
  font-size: 40px;
  font-weight: 700;
  color: var(--muted, #999);
}
.liveboosted {
  position: absolute;
  top: 6px;
  right: 6px;
  padding: 2px 6px;
  font-size: 11px;
  background: var(--accent, #0b7a75);
  color: #fff;
  border-radius: 6px;
}
.livekind {
  position: absolute;
  top: 6px;
  left: 6px;
  padding: 2px 6px;
  font-size: 11px;
  background: rgba(0, 0, 0, 0.6);
  color: #fff;
  border-radius: 6px;
}
.livebody {
  display: flex;
  flex-direction: column;
  gap: 6px;
  padding: 10px;
}
.livecaption {
  margin: 0;
  font-size: 13px;
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
  overflow: hidden;
}
.livestats {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  font-size: 12px;
}
.live-date {
  font-size: 12px;
}
.live-caption {
  max-width: 360px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.live-empty {
  display: flex;
  flex-direction: column;
  align-items: flex-start;
  gap: 10px;
}
</style>
