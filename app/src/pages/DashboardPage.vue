<script setup lang="ts">
import { computed, ref } from 'vue'
import { useRouter } from 'vue-router'
import { useAds, useDashboard } from '@/core/queries'
import { guardSuppressed } from '@/core/protected'
import { t } from '@/core/i18n'
import { useHubTab } from '@/core/subnav'
import { activeMonth, activeYear } from '@/core/navdate'
import { useWidePane } from '@/composables/useWidePane'
import MonthToggle from '@/components/ui/MonthToggle.vue'
import TableFrame from '@/components/tables/TableFrame.vue'
import DockBar from '@/components/ui/DockBar.vue'
import KpiCard from '@/components/ui/KpiCard.vue'
import BarChart from '@/components/ui/BarChart.vue'
import LineChart from '@/components/ui/LineChart.vue'
import LiveSection from '@/components/live/LiveSection.vue'
import ProtectedModal from '@/components/overlays/ProtectedModal.vue'

const router = useRouter()
// The deck works on one month: this is the shared value every card follows.
const month = activeMonth
const year = activeYear
const { stats, postsQ, metricsQ } = useDashboard(month, year)
const { data: adsView } = useAds()
const guardOpen = ref(false)

const root = ref<HTMLElement | null>(null)
useWidePane(root, 'dash-scroll')

// Hub tab (subnavbar) — sections come from HUB_TABS['/dashboard'].
const { active: section } = useHubTab('/dashboard')

const postedPct = computed(() =>
  stats.value?.total ? Math.round((stats.value.posted / stats.value.total) * 100) : 0,
)

// Active Meta Ads campaigns at a glance + simple alerts (Phase 2 integration).
const activeCampaigns = computed(() =>
  (adsView.value?.campaigns ?? []).filter((c) => c.status === 'ACTIVE').slice(0, 3),
)
const adsCurrency = computed(() => adsView.value?.accounts[0]?.currency ?? '')
function adsMoney(minor: number): string {
  const value = (minor / 100).toLocaleString('en-US', { minimumFractionDigits: 2, maximumFractionDigits: 2 })
  return adsCurrency.value ? `${value} ${adsCurrency.value}` : value
}
function adSpend(campaignId: string): number {
  return adsView.value?.insights.find((i) => i.campaignId === campaignId)?.spend ?? 0
}
const adsAlerts = computed(() => {
  const rows: string[] = []
  const now = Date.now()
  for (const c of adsView.value?.campaigns ?? []) {
    if (c.status !== 'ACTIVE') continue
    if (c.stopTime && new Date(c.stopTime).getTime() < now) rows.push(`${c.name}: ${t('ads.alertEnded')}`)
    else if ((c.dailyBudget || c.lifetimeBudget) && c.budgetRemaining === 0) rows.push(`${c.name}: ${t('ads.alertBudget')}`)
  }
  return rows.slice(0, 4)
})

const pillarBars = computed(() => {
  if (!stats.value) return []
  const rows = postsQ.data.value ?? []
  const total = rows.length
  return Object.entries(stats.value.pillarCounts).map(([label, value]) => {
    const pillarRows = rows.filter((r) => r.pillar === label)
    const posted = pillarRows.filter((r) => r.status === 'Done').length
    const pending = pillarRows.filter((r) => r.status === 'Start').length
    const wip = pillarRows.filter((r) => r.status === 'Design' || r.status === 'Dev').length
    return {
      label: label || 'Unassigned',
      value,
      total,
      posted,
      pending,
      wip,
      sublabel: `${posted} posted · ${pending} pending · ${wip} WIP`,
    }
  })
})

const platformBars = computed(() => {
  if (!stats.value) return []
  const rows = postsQ.data.value ?? []
  const total = rows.length
  return Object.entries(stats.value.platformCounts).map(([label, value]) => {
    const plRows = rows.filter((r) => r.platforms.includes(label))
    const posted = plRows.filter((r) => r.status === 'Done').length
    const remaining = plRows.length - posted
    return {
      label: label || 'Direct',
      value,
      total,
      posted,
      pending: remaining,
      sublabel: `${posted} posted · ${remaining} in progress`,
    }
  })
})

const statusBars = computed(() => {
  if (!stats.value) return []
  const rows = postsQ.data.value ?? []
  const total = rows.length
  const descriptions: Record<string, string> = {
    Start: 'In queue · awaiting production',
    Design: 'Creative design & visuals',
    Dev: 'Copywriting & media drafting',
    Done: 'Published & completed',
  }
  return Object.entries(stats.value.statusCounts)
    .filter(([_, count]) => count > 0)
    .map(([label, value]) => ({
      label,
      value,
      total,
      sublabel: descriptions[label] ?? `${value} topics`,
    }))
})

// Daily engagement and post volume timeline across the month
const trendData = computed(() => {
  const posts = postsQ.data.value ?? []
  const mets = metricsQ.data.value ?? []

  const postMetrics = new Map<string, { views: number; likes: number }>()
  for (const m of mets) {
    const cur = postMetrics.get(m.postId) ?? { views: 0, likes: 0 }
    cur.views += m.views
    cur.likes += m.likes
    postMetrics.set(m.postId, cur)
  }

  const daysInMonth = new Date(year.value, month.value, 0).getDate()

  const dailyViews = new Array(daysInMonth).fill(0)
  const dailyLikes = new Array(daysInMonth).fill(0)
  const dailyTotal = new Array(daysInMonth).fill(0)
  const dailyPosted = new Array(daysInMonth).fill(0)
  const dailyPending = new Array(daysInMonth).fill(0)
  const dailyWip = new Array(daysInMonth).fill(0)

  for (const p of posts) {
    let day = 0
    if (p.date) {
      const parts = p.date.split('-')
      day = parseInt(parts[2] ?? '0', 10)
    }
    const idx = day >= 1 && day <= daysInMonth ? day - 1 : 0
    dailyTotal[idx]++
    if (p.status === 'Done') dailyPosted[idx]++
    else if (p.status === 'Start') dailyPending[idx]++
    else if (p.status === 'Design' || p.status === 'Dev') dailyWip[idx]++

    const pm = postMetrics.get(p.id)
    if (pm) {
      dailyViews[idx] += pm.views
      dailyLikes[idx] += pm.likes
    }
  }

  const labels = Array.from({ length: daysInMonth }, (_, i) => `${i + 1}`)

  return {
    labels,
    series: [
      { label: 'Views', values: dailyViews, area: true },
      { label: 'Likes', values: dailyLikes },
    ],
    sparks: {
      total: dailyTotal,
      posted: dailyPosted,
      pending: dailyPending,
      wip: dailyWip,
      views: dailyViews,
      likes: dailyLikes,
    },
  }
})

function askEdit(): void {
  if (guardSuppressed()) {
    goMaster()
    return
  }
  guardOpen.value = true
}
function goMaster(): void {
  guardOpen.value = false
  void router.push(`/planner/${month.value}`)
}
</script>

<template>
  <div class="dash-page" ref="root">
    <h1>Dashboard</h1>

    <div class="dash-grid">
      <DockBar label="Dashboard controls">
        <MonthToggle v-model="month" v-model:year="year" />
        <span class="dock-sep" aria-hidden="true" />
        <RouterLink class="btn btn-icon" to="/analyze?tab=performance" title="Import metrics" aria-label="Import metrics">
          <svg viewBox="0 0 16 16" width="16" height="16" fill="none" stroke="currentColor"
            stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
            <path d="M8 2.5v7.5m0 0L5.2 7.2M8 10l2.8-2.8M3 13.5h10" />
          </svg>
        </RouterLink>
        <RouterLink class="btn btn-primary btn-icon" to="/plan?tab=monthly" title="New topic" aria-label="New topic">+</RouterLink>
      </DockBar>

      <div class="dash-main">
        <div v-if="postsQ.isError.value || metricsQ.isError.value" class="card">
          <p class="muted">Could not load dashboard data: {{ (postsQ.error.value ?? metricsQ.error.value)?.message }}</p>
        </div>
        <div v-else-if="!stats" class="muted">Computing…</div>
        <template v-else>
          <template v-if="section === 'overview'">
            <div class="grid6">
              <KpiCard label="Total" :value="stats.total" hint="topics this month" :spark="trendData.sparks.total" to="/plan?tab=monthly" />
              <KpiCard label="Posted" :value="stats.posted" :hint="`${postedPct}% of all`" :spark="trendData.sparks.posted" to="/plan?tab=monthly" />
              <KpiCard label="Pending" :value="stats.pending" hint="to start" :spark="trendData.sparks.pending" to="/plan?tab=monthly" />
              <KpiCard label="WIP" :value="stats.wip" hint="in design or dev" :spark="trendData.sparks.wip" to="/plan?tab=monthly" />
              <KpiCard label="Views" :value="stats.views.toLocaleString('en-US')" hint="this month" :spark="trendData.sparks.views" to="/analyze?tab=performance" />
              <KpiCard label="Likes" :value="stats.likes.toLocaleString('en-US')" hint="this month" :spark="trendData.sparks.likes" to="/analyze?tab=performance" />
            </div>

            <!-- Views & Engagement Line Chart -->
            <div class="mt-6">
              <div class="flex items-center justify-between mb-2">
                <h2 class="m-0 text-md font-bold tracking-tight">Views & Engagement Growth</h2>
              </div>
              <LineChart :labels="trendData.labels" :series="trendData.series" :height="160" />
            </div>

            <div class="mt-6">
              <h2 class="mb-2 text-md font-bold tracking-tight">Planned vs posted · by pillar</h2>
              <BarChart :items="pillarBars" unit="topics" />
            </div>

            <div class="grid2 mt-5">
              <div>
                <h2 class="mb-2 text-md font-bold tracking-tight" style="margin-top: 0;">By platform</h2>
                <BarChart :items="platformBars" unit="posts" />
              </div>
              <div>
                <h2 class="mb-2 text-md font-bold tracking-tight" style="margin-top: 0;">By status</h2>
                <BarChart :items="statusBars" unit="topics" />
              </div>
            </div>
          </template>

          <template v-else-if="section === 'live'">
            <LiveSection :limit="3" />
          </template>

          <template v-else-if="section === 'ads'">
            <section class="ads-glance">
              <div class="row">
                <h2 style="margin: 0;">{{ t('ads.active') }}</h2>
                <RouterLink class="btn ml-0" to="/ads">{{ t('ads.open') }}</RouterLink>
              </div>
              <template v-if="activeCampaigns.length || adsAlerts.length">
                <ul v-if="adsAlerts.length" class="mt-2.5 pl-4.5 text-warning text-sm-minus">
                  <li v-for="(a, i) in adsAlerts" :key="i">{{ a }}</li>
                </ul>
                <TableFrame>
                  <thead><tr><th>{{ t('ads.campaign') }}</th><th>{{ t('ads.status') }}</th><th class="num">{{ t('ads.spend') }}</th></tr></thead>
                  <tbody>
                    <tr v-for="c in activeCampaigns" :key="c.id" style="cursor: default;">
                      <td>{{ c.name }}</td>
                      <td><span class="badge badge-success">{{ c.status }}</span></td>
                      <td class="num">{{ adsMoney(adSpend(c.id) * 100) }}</td>
                      </tr>
                    </tbody>
                </TableFrame>
              </template>
              <p v-else class="muted">No active campaigns.</p>
            </section>
          </template>

          <template v-else>
            <h2 style="margin-top: 0;">Top 5 by views</h2>
            <TableFrame>
              <thead><tr><th>Topic</th><th>Status</th><th class="num">Views</th><th></th></tr></thead>
              <tbody>
                <tr v-for="t in stats.top5" :key="t.post.id" style="cursor: default;">
                  <td>{{ t.post.topic }}</td>
                  <td>{{ t.post.status }}</td>
                  <td class="num">{{ t.views.toLocaleString('en-US') }}</td>
                  <td><button class="btn" @click="askEdit">Edit</button></td>
                </tr>
              </tbody>
            </TableFrame>
          </template>
        </template>
      </div>
    </div>

    <ProtectedModal :open="guardOpen" field="Top-5 views cell" @cancel="guardOpen = false" @confirm="goMaster" />
  </div>
</template>
