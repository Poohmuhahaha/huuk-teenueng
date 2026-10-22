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
</script>

<template>
  <div class="perf-page" ref="root">
    <div class="perf-grid">
      <DockBar label="Performance controls">
        <MonthToggle v-model="month" v-model:year="year" />
        <span class="dock-sep" aria-hidden="true" />
        <div class="flex items-center gap-2">
          <label class="sr-only" for="perf-platform">Platform</label>
          <select id="perf-platform" class="field h-11 min-w-25 m-0" v-model="platform" aria-label="Platform">
            <option v-for="o in setup?.platforms ?? []" :key="o" :value="o">{{ o }}</option>
          </select>
          <button class="btn btn-primary h-11 whitespace-nowrap px-4 flex-none" :disabled="!platform || imp.isPending.value || !canImport"
            :title="canImport ? '' : t('auth.noPerm')"
            @click="imp.mutate({ platform, month, year })">
            {{ imp.isPending.value ? 'Importing…' : 'Import' }}
          </button>
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
        <h2>Platform goals</h2>
        <TableFrame>
          <thead><tr><th>Platform</th><th class="num">Start</th><th class="num">Goal</th><th class="num">Now</th><th class="num">Delta</th></tr></thead>
          <tbody>
            <tr v-for="g in goalRows" :key="g.platform" style="cursor: default;">
              <td>{{ g.platform }}</td>
              <td class="num">{{ g.start.toLocaleString('en-US') }}</td>
              <td class="num">{{ g.goal.toLocaleString('en-US') }}</td>
              <td class="num">{{ g.now.toLocaleString('en-US') }}</td>
              <td class="num">{{ g.delta >= 0 ? '+' : '' }}{{ g.delta.toLocaleString('en-US') }}</td>
            </tr>
          </tbody>
        </TableFrame>
        <h2>Views by month</h2>
        <BarChart :items="growth" />
        <h2>Per-post +7 days</h2>
        <p v-if="isEmpty && !isPending && !isError" class="muted">No posts recorded yet.</p>
        <TableFrame v-else>
          <thead><tr><th>Topic</th><th>Posted</th><th>+7 days</th><th class="num">Likes</th><th class="num">Views</th></tr></thead>
          <tbody>
            <tr v-for="r in byPost" :key="r.post.id" style="cursor: default;">
              <td>{{ r.post.topic }}</td>
              <td>{{ r.post.date ?? '—' }}</td>
              <td>{{ r.post.date ? plus7(r.post.date) : '—' }}</td>
              <td class="num">{{ r.likes.toLocaleString('en-US') }}</td>
              <td class="num">{{ r.views.toLocaleString('en-US') }}</td>
            </tr>
          </tbody>
        </TableFrame>
        <LiveSection variant="table" />
      </div>
    </div>
  </div>
</template>
