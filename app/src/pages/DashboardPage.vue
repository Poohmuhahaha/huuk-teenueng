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
const sectionLabel = computed(() => {
  if (section.value === 'ads') return 'Meta Ads'
  if (section.value === 'top') return 'Top Posts'
  return 'Overview'
})

const postedPct = computed(() =>
  stats.value?.total ? Math.round((stats.value.posted / stats.value.total) * 100) : 0,
)

// Active Meta Ads campaigns at a glance + simple alerts (Phase 2 integration).
const activeCampaigns = computed(() =>
  (adsView.value?.campaigns ?? []).filter((c) => c.status === 'ACTIVE'),
)
const adsCurrency = computed(() => adsView.value?.accounts[0]?.currency ?? '')
function adsMoney(minor: number): string {
  const value = (minor / 100).toLocaleString('en-US', { minimumFractionDigits: 2, maximumFractionDigits: 2 })
  return adsCurrency.value ? `${value} ${adsCurrency.value}` : value
}
function adSpend(campaignId: string): number {
  return adsView.value?.insights.find((i) => i.campaignId === campaignId)?.spend ?? 0
}
function adInsight(campaignId: string) {
  return adsView.value?.insights.find((i) => i.campaignId === campaignId)
}
function formatObjective(obj: string): string {
  if (!obj) return ''
  return obj.replace(/^OUTCOME_/, '').toLowerCase().replace(/_/g, ' ')
}

const adsTotals = computed(() => {
  const insights = adsView.value?.insights ?? []
  let totalSpend = 0
  let totalImpressions = 0
  let totalClicks = 0
  for (const ins of insights) {
    totalSpend += ins.spend
    totalImpressions += ins.impressions
    totalClicks += ins.clicks
  }
  const avgCtr = totalImpressions > 0 ? (totalClicks / totalImpressions) * 100 : 0
  return {
    spend: totalSpend,
    impressions: totalImpressions,
    clicks: totalClicks,
    ctr: avgCtr,
    activeCount: activeCampaigns.value.length,
  }
})
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

const PLATFORM_BRAND: Record<string, string> = {
  facebook: '#1877F2',
  meta: '#1877F2',
  instagram: '#E4405F',
  youtube: '#FF0000',
  tiktok: '#0F0F0F',
}

const platformBars = computed(() => {
  if (!stats.value) return []
  const rows = postsQ.data.value ?? []
  const total = rows.length
  return Object.entries(stats.value.platformCounts).map(([label, value]) => {
    const plRows = rows.filter((r) => r.platforms.includes(label))
    const posted = plRows.filter((r) => r.status === 'Done').length
    const remaining = plRows.length - posted
    const brand = label ? PLATFORM_BRAND[label.toLowerCase()] : undefined
    return {
      label: label || 'Direct',
      value,
      total,
      posted,
      pending: remaining,
      sublabel: `${posted} posted · ${remaining} in progress`,
      platform: brand ? label : undefined,
      color: brand,
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
    <div class="dash-grid">
      <DockBar label="Dashboard controls">
        <div class="flex items-center justify-between w-full gap-3">
          <div class="flex items-center gap-2">
            <MonthToggle v-model="month" v-model:year="year" />
            <span class="dock-sep" aria-hidden="true" />
            <span class="text-xs font-semibold text-muted tracking-wider uppercase">{{ sectionLabel }}</span>
          </div>
          <div class="flex items-center gap-2 ml-auto">
            <RouterLink class="btn btn-icon w-auto" to="/analyze?tab=performance" title="Import metrics" aria-label="Import metrics">
              <svg viewBox="0 0 16 16" width="16" height="16" fill="none" stroke="currentColor"
                stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
                <path d="M8 2.5v7.5m0 0L5.2 7.2M8 10l2.8-2.8M3 13.5h10" />
              </svg>
            </RouterLink>
            <RouterLink class="btn btn-primary btn-icon w-auto" to="/plan?tab=monthly" title="New topic" aria-label="New topic">+</RouterLink>
          </div>
        </div>
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
                <h2 class="m-0 text-md font-bold tracking-tight normal-case [font-family:var(--font)]">Views & Engagement Growth</h2>
              </div>
              <LineChart :labels="trendData.labels" :series="trendData.series" :height="160" />
            </div>

            <div class="mt-6">
              <h2 class="mb-2 text-md font-bold tracking-tight normal-case [font-family:var(--font)]">Planned vs posted · by pillar</h2>
              <BarChart :items="pillarBars" />
            </div>

            <div class="grid2 mt-5">
              <div>
                <h2 class="mb-2 text-md font-bold tracking-tight normal-case [font-family:var(--font)]" style="margin-top: 0;">By platform</h2>
                <BarChart :items="platformBars" />
              </div>
              <div>
                <h2 class="mb-2 text-md font-bold tracking-tight normal-case [font-family:var(--font)]" style="margin-top: 0;">By status</h2>
                <BarChart :items="statusBars" />
              </div>
            </div>
          </template>

          <template v-else-if="section === 'ads'">
            <section class="ads-glance flex flex-col gap-6 text-left">
              <!-- Header with Action -->
              <div class="flex items-center justify-between flex-wrap gap-4">
                <div>
                  <div class="text-xs font-bold uppercase tracking-wider text-muted mb-0.5">Campaigns & Spend</div>
                  <h2 class="text-2xl sm:text-3xl font-extrabold uppercase tracking-tight m-0 font-display text-ink">{{ t('ads.active') }}</h2>
                </div>
                <RouterLink class="btn text-xs py-2 px-4 inline-flex items-center gap-2 font-bold hover:border-ink shadow-xs" to="/promote?tab=ads">
                  <span>{{ t('ads.open') }}</span>
                  <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
                    <path d="M5 12h14M12 5l7 7-7 7" />
                  </svg>
                </RouterLink>
              </div>

              <!-- Top KPI Summary Cards -->
              <div class="grid grid-cols-2 lg:grid-cols-4 gap-3">
                <div class="border border-line rounded-xl bg-surface p-3.5 shadow-xs flex flex-col justify-between">
                  <div class="text-2xs font-extrabold uppercase tracking-wider text-muted">Total Ad Spend</div>
                  <div class="text-xl sm:text-2xl font-extrabold font-mono text-ink mt-1.5">{{ adsMoney(adsTotals.spend * 100) }}</div>
                  <div class="text-2xs text-muted mt-1">Across active campaigns</div>
                </div>

                <div class="border border-line rounded-xl bg-surface p-3.5 shadow-xs flex flex-col justify-between">
                  <div class="text-2xs font-extrabold uppercase tracking-wider text-muted">Active Campaigns</div>
                  <div class="text-xl sm:text-2xl font-extrabold font-mono text-ink mt-1.5">{{ adsTotals.activeCount }}</div>
                  <div class="text-2xs text-success font-semibold mt-1 flex items-center gap-1.5">
                    <span class="w-2 h-2 rounded-full bg-success"></span> Running now
                  </div>
                </div>

                <div class="border border-line rounded-xl bg-surface p-3.5 shadow-xs flex flex-col justify-between">
                  <div class="text-2xs font-extrabold uppercase tracking-wider text-muted">Total Impressions</div>
                  <div class="text-xl sm:text-2xl font-extrabold font-mono text-ink mt-1.5">{{ adsTotals.impressions.toLocaleString() }}</div>
                  <div class="text-2xs text-muted mt-1">Paid audience reach</div>
                </div>

                <div class="border border-line rounded-xl bg-surface p-3.5 shadow-xs flex flex-col justify-between">
                  <div class="text-2xs font-extrabold uppercase tracking-wider text-muted">Clicks & CTR</div>
                  <div class="text-xl sm:text-2xl font-extrabold font-mono text-ink mt-1.5">{{ adsTotals.clicks.toLocaleString() }}</div>
                  <div class="text-2xs text-accent font-semibold mt-1">{{ adsTotals.ctr.toFixed(2) }}% avg CTR</div>
                </div>
              </div>

              <!-- Alerts if any -->
              <div v-if="adsAlerts.length" class="p-3.5 rounded-xl border border-warning/30 bg-warning-wash text-warning text-xs flex flex-col gap-1">
                <div v-for="(a, i) in adsAlerts" :key="i" class="flex items-center gap-2">
                  <span>⚠️</span>
                  <span>{{ a }}</span>
                </div>
              </div>

              <!-- Active Campaigns Section -->
              <div class="flex flex-col gap-3">
                <div class="flex items-center justify-between">
                  <span class="text-xs font-bold uppercase tracking-wider text-muted">Active Campaign Cards</span>
                  <span class="text-xs text-muted font-medium">{{ activeCampaigns.length }} total</span>
                </div>

                <div v-if="activeCampaigns.length" class="grid grid-cols-1 md:grid-cols-3 gap-3.5">
                  <div v-for="c in activeCampaigns" :key="c.id"
                    class="border border-line rounded-xl bg-surface p-4 shadow-xs flex flex-col justify-between gap-3 hover:border-ink hover:shadow-sm transition-all">
                    <div>
                      <div class="flex items-start justify-between gap-2 mb-2">
                        <div class="flex items-center gap-2 min-w-0">
                          <svg class="w-4 h-4 text-[#1877F2] flex-none" viewBox="0 0 24 24" fill="currentColor" aria-hidden="true">
                            <path d="M24 12.073c0-6.627-5.373-12-12-12s-12 5.373-12 12c0 5.99 4.388 10.954 10.125 11.854v-8.385H7.078v-3.47h3.047V9.43c0-3.007 1.792-4.669 4.533-4.669 1.312 0 2.686.235 2.686.235v2.953H15.83c-1.491 0-1.956.925-1.956 1.874v2.25h3.328l-.532 3.47h-2.796v8.385C19.612 23.027 24 18.062 24 12.073z"/>
                          </svg>
                          <strong class="text-sm font-bold text-ink truncate" :title="c.name">{{ c.name }}</strong>
                        </div>
                        <span class="badge badge-success flex-none text-2xs">{{ c.status }}</span>
                      </div>
                      <div v-if="c.objective" class="inline-flex px-2 py-0.5 rounded text-2xs font-semibold bg-wash text-muted uppercase tracking-wider mb-1">
                        {{ formatObjective(c.objective) }}
                      </div>
                    </div>

                    <div class="pt-3 border-t border-faint flex flex-col gap-1.5">
                      <div class="flex items-center justify-between text-xs">
                        <span class="text-muted">{{ t('ads.spend') }}</span>
                        <span class="font-bold text-ink font-mono">{{ adsMoney(adSpend(c.id) * 100) }}</span>
                      </div>
                      <div v-if="adInsight(c.id)" class="flex items-center justify-between text-2xs text-muted">
                        <span>{{ adInsight(c.id)?.impressions.toLocaleString() }} views</span>
                        <span class="font-semibold text-ink">{{ adInsight(c.id)?.clicks.toLocaleString() }} clicks ({{ adInsight(c.id)?.ctr }}%)</span>
                      </div>
                    </div>
                  </div>
                </div>

                <div v-else class="p-8 border border-dashed border-line rounded-xl text-center text-muted">
                  No active campaigns.
                </div>
              </div>

              <!-- Performance Breakdown Table -->
              <div v-if="adsView?.insights?.length" class="flex flex-col gap-2 mt-2">
                <span class="text-xs font-bold uppercase tracking-wider text-muted">Campaign Performance Breakdown</span>
                <TableFrame>
                  <thead>
                    <tr>
                      <th>Campaign</th>
                      <th class="num">Spend</th>
                      <th class="num">Impressions</th>
                      <th class="num">Clicks</th>
                      <th class="num">CTR</th>
                      <th class="num">CPC</th>
                    </tr>
                  </thead>
                  <tbody>
                    <tr v-for="ins in adsView.insights" :key="ins.campaignId">
                      <td class="font-semibold text-ink">
                        {{ adsView.campaigns.find(c => c.id === ins.campaignId)?.name || ins.campaignId }}
                      </td>
                      <td class="num font-bold">{{ adsMoney(ins.spend * 100) }}</td>
                      <td class="num text-muted">{{ ins.impressions.toLocaleString() }}</td>
                      <td class="num text-muted">{{ ins.clicks.toLocaleString() }}</td>
                      <td class="num text-accent font-semibold">{{ ins.ctr }}%</td>
                      <td class="num text-muted">{{ adsMoney(ins.cpc * 100) }}</td>
                    </tr>
                  </tbody>
                </TableFrame>
              </div>
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
                  <td>{{ t.views.toLocaleString('en-US') }}</td>
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
