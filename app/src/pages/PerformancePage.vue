<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { useAllPosts, useMetrics, useSetup, useImportMetrics, usePermission } from '@/core/queries'
import { t } from '@/core/i18n'
import { platformGoals, plus7 } from '@/mock/db'
import BarChart from '@/components/ui/BarChart.vue'
import MonthToggle from '@/components/ui/MonthToggle.vue'
import TableFrame from '@/components/tables/TableFrame.vue'
import DockBar from '@/components/ui/DockBar.vue'
import LiveSection from '@/components/live/LiveSection.vue'
import PlatformLogo from '@/components/ui/PlatformLogo.vue'
import { useWidePane } from '@/composables/useWidePane'
import { activeMonth, activeYear } from '@/core/navdate'

const { data: metrics, isPending: metricsPending } = useMetrics()
const { data: setup } = useSetup()
const { data: allPosts, isPending: postsPending, isError, error: loadError, refetch } = useAllPosts()
const isPending = computed(() => postsPending.value && metricsPending.value)

const imp = useImportMetrics()
const { can } = usePermission()
const canImport = computed(() => can('metrics.import'))
const root = ref<HTMLElement | null>(null)
useWidePane(root, 'perf-scroll')
const platform = ref('')
// The deck works on one month: this is the shared value every card follows.
const month = activeMonth
const year = activeYear

watch(setup, (s) => {
  if (s && !platform.value) platform.value = s.platforms[0] ?? ''
}, { immediate: true })

const posts = computed(() => allPosts.value ?? [])
const isEmpty = computed(() => posts.value.length === 0)

const byPost = computed(() => {
  const mets = metrics.value ?? []
  const rows = posts.value
  return rows.map((p) => {
    const rows = mets.filter((m) => m.postId === p.id)
    return {
      post: p,
      likes: rows.reduce((s, m) => s + m.likes, 0),
      views: rows.reduce((s, m) => s + m.views, 0),
    }
  })
})

const postPageIndex = ref(0)
const postPageSize = ref(8)
const totalByPost = computed(() => byPost.value.length)
const totalByPostPages = computed(() => Math.max(1, Math.ceil(totalByPost.value / postPageSize.value)))
const rangePostStart = computed(() => totalByPost.value === 0 ? 0 : postPageIndex.value * postPageSize.value + 1)
const rangePostEnd = computed(() => Math.min(totalByPost.value, (postPageIndex.value + 1) * postPageSize.value))
const visibleByPost = computed(() => {
  const all = byPost.value
  return all.slice(postPageIndex.value * postPageSize.value, (postPageIndex.value + 1) * postPageSize.value)
})

const goalRows = computed(() => {
  const mets = metrics.value ?? []
  return platformGoals.map((g) => {
    const likes = mets.filter((m) => m.platform === g.platform).reduce((s, m) => s + m.likes, 0)
    const now = g.start + likes
    return { ...g, now, delta: now - g.goal }
  })
})

const growth = computed(() => {
  const mets = metrics.value ?? []
  const rows = posts.value
  const byMonth: Record<string, number> = {}
  for (const m of mets) {
    const p = rows.find((x) => x.id === m.postId)
    const key = p?.date ? p.date.slice(0, 7) : 'undated'
    byMonth[key] = (byMonth[key] ?? 0) + m.views
  }
  return Object.entries(byMonth)
    .sort(([a], [b]) => a.localeCompare(b))
    .map(([label, value]) => ({ label, value }))
})

const totalGrowthViews = computed(() => growth.value.reduce((s, it) => s + it.value, 0))
</script>

<template>
  <div class="perf-page" ref="root">
    <div class="perf-grid">
      <DockBar label="Performance controls">
        <div class="flex items-center justify-between w-full gap-3">
          <div class="flex items-center gap-2">
            <MonthToggle v-model="month" v-model:year="year" />
            <span class="dock-sep" aria-hidden="true" />
            <label class="sr-only" for="perf-platform">Platform</label>
            <select id="perf-platform" class="field h-10 min-w-28 m-0 rounded-full" v-model="platform" aria-label="Platform">
              <option v-for="o in setup?.platforms ?? []" :key="o" :value="o">{{ o }}</option>
            </select>
          </div>
          <div class="flex items-center gap-2 ml-auto">
            <button class="btn btn-primary h-10 whitespace-nowrap px-5 flex-none rounded-full" :disabled="!platform || imp.isPending.value || !canImport"
              :title="canImport ? '' : t('auth.noPerm')"
              @click="imp.mutate({ platform, month, year })">
              {{ imp.isPending.value ? 'Importing…' : 'Import' }}
            </button>
          </div>
        </div>
      </DockBar>

      <div class="perf-main">
        <span v-if="imp.data.value" class="text-xs text-muted">Imported {{ imp.data.value.imported }} metric row(s) for {{ platform }} · {{ month }}/{{ year }}.</span>
        <span v-if="imp.error.value" class="text-xs text-danger">{{ imp.error.value instanceof Error ? imp.error.value.message : imp.error.value }}</span>
        <div v-if="isPending" class="muted">Loading performance…</div>
        <div v-if="isError" class="card" role="alert">
          <p class="muted">Could not load posts: {{ loadError?.message }}</p>
          <button class="btn" @click="() => refetch()">{{ t('common.tryAgain') }}</button>
        </div>
        <div class="mb-6">
          <h2 class="text-xs font-bold uppercase tracking-widest text-muted mb-2.5">Platform goals</h2>
          <TableFrame>
            <thead><tr><th>Platform</th><th class="num">Start</th><th class="num">Goal</th><th class="num">Now</th><th class="num">Delta</th></tr></thead>
            <tbody>
              <tr v-for="g in goalRows" :key="g.platform" style="cursor: default;">
                <td class="font-medium text-ink flex items-center gap-2">
                  <PlatformLogo :name="g.platform" :size="15" />
                  <span>{{ g.platform }}</span>
                </td>
                <td class="num font-mono">{{ g.start.toLocaleString('en-US') }}</td>
                <td class="num font-mono">{{ g.goal.toLocaleString('en-US') }}</td>
                <td class="num font-mono font-bold">{{ g.now.toLocaleString('en-US') }}</td>
                <td class="num font-mono">
                  <span class="inline-flex items-center px-2 py-0.5 rounded-full text-xs font-semibold tabular-nums"
                    :class="g.delta >= 0 ? 'bg-emerald-500/10 text-emerald-700 border border-emerald-500/20' : 'bg-rose-500/10 text-rose-700 border border-rose-500/20'">
                    {{ g.delta >= 0 ? '+' : '' }}{{ g.delta.toLocaleString('en-US') }}
                  </span>
                </td>
              </tr>
            </tbody>
          </TableFrame>
        </div>

        <div class="mb-6 card-fade p-4 sm:p-5 flex flex-col justify-between">
          <div>
            <div class="flex items-center justify-between gap-2 mb-2">
              <div class="flex items-center gap-2">
                <h2 class="text-[12.5px] font-semibold text-ink-soft uppercase tracking-wider m-0">Views by month</h2>
                <span class="inline-flex items-center px-2 py-0.5 rounded-full text-[11px] font-semibold bg-black text-white">
                  {{ growth.length }} active mos
                </span>
              </div>
              <span class="text-xs font-semibold text-muted font-mono">2026</span>
            </div>
            <p class="tabular text-[22px] sm:text-[24px] font-bold leading-tight tracking-[-0.03em] text-ink font-mono mb-2">
              {{ totalGrowthViews.toLocaleString('en-US') }} views
            </p>
          </div>
          <BarChart :items="growth" variant="towers" :card="false" />
        </div>

        <div class="mb-6">
          <h2 class="text-xs font-bold uppercase tracking-widest text-muted mb-2.5">Per-post +7 days</h2>
          <p v-if="isEmpty && !isPending && !isError" class="muted">No posts recorded yet.</p>
          <TableFrame v-else>
            <thead><tr><th>Topic</th><th>Posted</th><th>+7 days</th><th class="num">Likes</th><th class="num">Views</th></tr></thead>
            <tbody>
              <tr v-for="r in visibleByPost" :key="r.post.id" style="cursor: default;">
                <td class="font-semibold text-ink">{{ r.post.topic }}</td>
                <td class="text-muted text-xs font-mono">{{ r.post.date ?? '—' }}</td>
                <td class="text-muted text-xs font-mono">{{ r.post.date ? plus7(r.post.date) : '—' }}</td>
                <td class="num font-mono font-bold">{{ r.likes.toLocaleString('en-US') }}</td>
                <td class="num font-mono font-bold text-ink">{{ r.views.toLocaleString('en-US') }}</td>
              </tr>
            </tbody>
          </TableFrame>

          <!-- Pagination bar -->
          <div
            v-if="totalByPost > 0"
            class="flex items-center justify-between gap-3 px-5 py-3 mt-2.5 bg-surface rounded-xl border border-line/60 shadow-xs text-sm text-muted min-h-[50px] select-none"
          >
            <span class="truncate">Showing <strong class="font-semibold text-ink font-mono">{{ rangePostStart }}–{{ rangePostEnd }}</strong> of <strong class="font-semibold text-ink font-mono">{{ totalByPost }}</strong> posts</span>
            <div v-if="totalByPostPages > 1" class="inline-flex items-center rounded-lg border border-line bg-surface shadow-xs overflow-hidden flex-none ml-auto h-8 sm:h-8.5">
              <button
                type="button"
                class="px-3 h-full inline-flex items-center justify-center text-sm font-semibold text-ink hover:bg-wash transition-colors disabled:opacity-30 disabled:cursor-not-allowed cursor-pointer border-r border-line"
                :disabled="postPageIndex === 0"
                aria-label="Previous page"
                title="Previous page"
                @click="postPageIndex--"
              >
                ‹
              </button>
              <span class="px-3 h-full inline-flex items-center justify-center font-mono text-xs sm:text-sm font-medium text-ink whitespace-nowrap select-none">
                {{ postPageIndex + 1 }} / {{ totalByPostPages }}
              </span>
              <button
                type="button"
                class="px-3 h-full inline-flex items-center justify-center text-sm font-semibold text-ink hover:bg-wash transition-colors disabled:opacity-30 disabled:cursor-not-allowed cursor-pointer border-l border-line"
                :disabled="postPageIndex >= totalByPostPages - 1"
                aria-label="Next page"
                title="Next page"
                @click="postPageIndex++"
              >
                ›
              </button>
            </div>
          </div>
        </div>
        <LiveSection variant="table" />
      </div>
    </div>
  </div>
</template>
