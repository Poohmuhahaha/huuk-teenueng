<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { useAllPosts, useMetrics, useSetup, useImportMetrics, usePermission } from '@/core/queries'
import { t } from '@/core/i18n'
import { platformGoals, plus7 } from '@/mock/db'
import BarChart from '@/components/ui/BarChart.vue'
import MonthToggle from '@/components/ui/MonthToggle.vue'
import LiveSection from '@/components/live/LiveSection.vue'
import { useWidePane } from '@/composables/useWidePane'
import { activeMonth } from '@/core/navdate'

const { data: metrics } = useMetrics()
const { data: setup } = useSetup()
const { data: allPosts, isError, error: loadError, refetch } = useAllPosts()

const imp = useImportMetrics()
const { can } = usePermission()
const canImport = computed(() => can('metrics.import'))
const root = ref<HTMLElement | null>(null)
useWidePane(root, 'perf-scroll')
const platform = ref('')
// The deck works on one month: this is the shared value every card follows.
const month = activeMonth

watch(setup, (s) => {
  if (s && !platform.value) platform.value = s.platforms[0] ?? ''
}, { immediate: true })

const posts = computed(() => allPosts.value ?? [])

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
    <h1>Performance</h1>

    <div class="perf-grid">
      <aside class="perf-side">
        <MonthToggle v-model="month" />
        <label class="lbl" for="perf-platform">Platform</label>
        <select id="perf-platform" class="field" v-model="platform">
          <option v-for="o in setup?.platforms ?? []" :key="o" :value="o">{{ o }}</option>
        </select>
        <button class="btn btn-primary" :disabled="!platform || imp.isPending.value || !canImport"
          :title="canImport ? '' : t('auth.noPerm')"
          @click="imp.mutate({ platform, month })">
          {{ imp.isPending.value ? 'Importing…' : 'Import' }}
        </button>
        <p v-if="imp.data.value" class="muted">Imported {{ imp.data.value.imported }} metric row(s) for {{ platform }} · month {{ month }}.</p>
        <p v-if="imp.error.value" class="muted">{{ imp.error.value instanceof Error ? imp.error.value.message : imp.error.value }}</p>
      </aside>

      <div class="perf-main">
        <div v-if="isError" class="card">
          <p class="muted">Could not load posts: {{ loadError?.message }}</p>
          <button class="btn" @click="() => refetch()">{{ t('common.tryAgain') }}</button>
        </div>
        <h2>Platform goals</h2>
        <div class="tblwrap">
          <table class="tbl">
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
          </table>
        </div>
        <h2>Views by month</h2>
        <BarChart :items="growth" />
        <h2>Per-post +7 days</h2>
        <div class="tblwrap">
          <table class="tbl">
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
          </table>
        </div>
        <LiveSection variant="table" />
      </div>
    </div>
  </div>
</template>
