<script setup lang="ts">
import { computed, ref, onMounted, onBeforeUnmount } from 'vue'
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
import CountUpNumber from '@/components/ui/CountUpNumber.vue'
import BarChart from '@/components/ui/BarChart.vue'
import type { BarItem } from '@/components/ui/BarChart.vue'
import LineChart from '@/components/ui/LineChart.vue'
import StatusChip from '@/components/ui/StatusChip.vue'
import PlatformLogo from '@/components/ui/PlatformLogo.vue'
import ProtectedModal from '@/components/overlays/ProtectedModal.vue'
import Drawer from '@/components/overlays/Drawer.vue'
import PerformancePage from '@/pages/PerformancePage.vue'
import FinancePage from '@/pages/FinancePage.vue'
import type { Post } from '@/mock/db'

const router = useRouter()
// The deck works on one month: this is the shared value every card follows.
const month = activeMonth
const year = activeYear
const { stats, postsQ, metricsQ } = useDashboard(month, year)
const { data: adsView } = useAds()
const guardOpen = ref(false)

const root = ref<HTMLElement | null>(null)
useWidePane(root, 'dash-scroll')

const isMobile = ref(typeof window !== 'undefined' ? window.innerWidth < 768 : false)
let mql: MediaQueryList | null = null
const onMql = (e: MediaQueryListEvent | MediaQueryList): void => {
  isMobile.value = e.matches
}
onMounted(() => {
  if (typeof window !== 'undefined' && window.matchMedia) {
    mql = window.matchMedia('(max-width: 767px)')
    isMobile.value = mql.matches
    mql.addEventListener('change', onMql)
  }
})
onBeforeUnmount(() => {
  mql?.removeEventListener('change', onMql)
})

interface BreakdownContext {
  type: 'pillar' | 'platform' | 'status'
  item: BarItem
}

const breakdown = ref<BreakdownContext | null>(null)
const breakdownSheetOpen = ref(false)
const breakdownSubFilter = ref<'all' | 'done' | 'pending'>('all')

function openBreakdown(type: 'pillar' | 'platform' | 'status', item: BarItem): void {
  breakdown.value = { type, item }
  breakdownSubFilter.value = 'all'
  breakdownSheetOpen.value = true
}

const breakdownPosts = computed<Post[]>(() => {
  if (!breakdown.value) return []
  const all = postsQ.data.value ?? []
  const { type, item } = breakdown.value

  let filtered = all.filter((p) => {
    if (type === 'pillar') {
      return item.label === 'Unassigned' ? !p.pillar : p.pillar === item.label
    }
    if (type === 'platform') {
      return item.label === 'Direct' ? (!p.platforms || p.platforms.length === 0) : p.platforms.includes(item.label)
    }
    if (type === 'status') {
      return p.status === item.label
    }
    return true
  })

  if (breakdownSubFilter.value === 'done') {
    filtered = filtered.filter((p) => p.status === 'Done')
  } else if (breakdownSubFilter.value === 'pending') {
    filtered = filtered.filter((p) => p.status !== 'Done')
  }

  return filtered
})

function goToPost(post: Post): void {
  breakdownSheetOpen.value = false
  void router.push({ path: '/plan', query: { tab: 'monthly', m: String(month.value), post: post.id } })
}

function goToPlanner(): void {
  breakdownSheetOpen.value = false
  void router.push({ path: '/plan', query: { tab: 'monthly', m: String(month.value) } })
}

// Hub tab (subnavbar) — sections come from HUB_TABS['/dashboard'].
const { active: section } = useHubTab('/dashboard')
const sectionLabel = computed(() => {
  if (section.value === 'ads') return 'Meta Ads'
  if (section.value === 'top') return 'Top Posts'
  if (section.value === 'performance') return 'Performance'
  if (section.value === 'finance') return 'Finance'
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

const showAllInsights = ref(false)
const visibleInsights = computed(() => {
  const insights = adsView.value?.insights ?? []
  if (showAllInsights.value || insights.length <= 5) return insights
  return insights.slice(0, 5)
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

  function calcTrend(arr: number[]): string {
    if (arr.length < 2) return '+0%'
    const mid = Math.floor(arr.length / 2)
    const first = arr.slice(0, mid).reduce((a, b) => a + b, 0)
    const second = arr.slice(mid).reduce((a, b) => a + b, 0)
    if (first === 0 && second === 0) return '+0%'
    if (first === 0) return '+100%'
    const diff = ((second - first) / first) * 100
    return `${diff >= 0 ? '+' : ''}${diff.toFixed(1)}%`
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
    trends: {
      views: calcTrend(dailyViews),
      likes: calcTrend(dailyLikes),
    },
  }
})

function postLikes(postId: string): number {
  return (metricsQ.data.value ?? []).filter((m) => m.postId === postId).reduce((s, m) => s + m.likes, 0)
}

const top5TotalViews = computed(() =>
  (stats.value?.top5 ?? []).reduce((s, t) => s + t.views, 0),
)

const top5TotalLikes = computed(() =>
  (stats.value?.top5 ?? []).reduce((s, t) => s + postLikes(t.post.id), 0),
)

const bestPost = computed(() => stats.value?.top5?.[0] ?? null)

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
      <DockBar v-if="section !== 'performance'" label="Dashboard controls">
        <div class="flex items-center justify-between w-full gap-3">
          <div class="flex items-center gap-2">
            <MonthToggle v-model="month" v-model:year="year" />
            <span class="dock-sep" aria-hidden="true" />
            <span class="text-xs font-semibold text-muted tracking-wider uppercase">{{ sectionLabel }}</span>
          </div>
          <div class="flex items-center gap-2 ml-auto">
            <RouterLink class="btn btn-icon w-auto" to="/dashboard?tab=performance" title="Import metrics" aria-label="Import metrics">
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
        <template v-if="section === 'performance'">
          <PerformancePage />
        </template>

        <template v-else-if="section === 'finance'">
          <FinancePage />
        </template>

        <template v-else>
          <div v-if="postsQ.isError.value || metricsQ.isError.value" class="card">
            <p class="muted">Could not load dashboard data: {{ (postsQ.error.value ?? metricsQ.error.value)?.message }}</p>
          </div>
          <div v-else-if="!stats" class="muted">Computing…</div>
          <template v-else>
            <template v-if="section === 'ads'">
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
                <KpiCard
                  label="Total Ad Spend"
                  :value="adsMoney(adsTotals.spend * 100)"
                  hint="Across active campaigns"
                  to="/promote?tab=ads"
                />
                <KpiCard
                  label="Active Campaigns"
                  :value="adsTotals.activeCount"
                  hint="Running now"
                  to="/promote?tab=ads"
                />
                <KpiCard
                  label="Total Impressions"
                  :value="adsTotals.impressions.toLocaleString('en-US')"
                  hint="Paid audience reach"
                  to="/promote?tab=ads"
                />
                <KpiCard
                  label="Clicks & CTR"
                  :value="adsTotals.clicks.toLocaleString('en-US')"
                  :hint="`${adsTotals.ctr.toFixed(2)}% avg CTR`"
                  to="/promote?tab=ads"
                />
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
                    class="card-fade rounded-2xl p-4 sm:p-5 shadow-xs flex flex-col justify-between gap-3 hover:border-ink hover:shadow-sm transition-all">
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
                        <span class="font-bold text-ink font-mono">
                          <CountUpNumber :value="adsMoney(adSpend(c.id) * 100)" />
                        </span>
                      </div>
                      <div v-if="adInsight(c.id)" class="flex items-center justify-between text-2xs text-muted">
                        <span><CountUpNumber :value="`${adInsight(c.id)?.impressions.toLocaleString()} views`" /></span>
                        <span class="font-semibold text-ink"><CountUpNumber :value="`${adInsight(c.id)?.clicks.toLocaleString()} clicks`" /> ({{ adInsight(c.id)?.ctr }}%)</span>
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
                <div class="flex items-center justify-between">
                  <span class="text-xs font-bold uppercase tracking-wider text-muted">Campaign Performance Breakdown</span>
                  <button
                    v-if="(adsView?.insights?.length ?? 0) > 5"
                    type="button"
                    class="text-xs font-semibold text-accent hover:underline cursor-pointer bg-transparent border-0 p-0"
                    @click="showAllInsights = !showAllInsights"
                  >
                    {{ showAllInsights ? 'Show top 5' : `View all (${adsView.insights.length})` }}
                  </button>
                </div>
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
                    <tr v-for="ins in visibleInsights" :key="ins.campaignId">
                      <td class="font-semibold text-ink">
                        {{ adsView.campaigns.find(c => c.id === ins.campaignId)?.name || ins.campaignId }}
                      </td>
                      <td class="num font-bold">
                        <CountUpNumber :value="adsMoney(ins.spend * 100)" />
                      </td>
                      <td class="num text-muted">
                        <CountUpNumber :value="ins.impressions.toLocaleString()" />
                      </td>
                      <td class="num text-muted">
                        <CountUpNumber :value="ins.clicks.toLocaleString()" />
                      </td>
                      <td class="num text-accent font-semibold">{{ ins.ctr }}%</td>
                      <td class="num text-muted">
                        <CountUpNumber :value="adsMoney(ins.cpc * 100)" />
                      </td>
                    </tr>
                  </tbody>
                </TableFrame>
              </div>
            </section>
          </template>

          <template v-else-if="section === 'top'">
            <section class="top-posts-glance flex flex-col gap-6 text-left">
              <!-- Header with Action -->
              <div class="flex items-center justify-between flex-wrap gap-4">
                <div>
                  <div class="text-xs font-bold uppercase tracking-wider text-muted mb-0.5">Engagement Leaderboard</div>
                  <h2 class="text-2xl sm:text-3xl font-extrabold uppercase tracking-tight m-0 font-display text-ink">Top 5 by views</h2>
                </div>
                <button type="button" class="btn text-xs py-2 px-4 inline-flex items-center gap-2 font-bold hover:border-ink shadow-xs" @click="askEdit">
                  <span>Open Planner</span>
                  <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
                    <path d="M5 12h14M12 5l7 7-7 7" />
                  </svg>
                </button>
              </div>

              <!-- Top KPI Summary Cards -->
              <div class="grid grid-cols-1 sm:grid-cols-3 gap-3">
                <KpiCard
                  label="Top 5 Total Views"
                  :value="top5TotalViews.toLocaleString('en-US')"
                  hint="Combined views of top 5 posts"
                />
                <KpiCard
                  label="Top 5 Total Likes"
                  :value="top5TotalLikes.toLocaleString('en-US')"
                  hint="Direct audience engagement"
                />
                <div class="card-fade rounded-2xl p-4 sm:p-5 shadow-xs flex flex-col justify-between">
                  <div class="text-2xs font-extrabold uppercase tracking-wider text-muted">Top Performer</div>
                  <div class="text-base font-bold text-ink truncate mt-1.5" :title="bestPost?.post.topic">{{ bestPost?.post.topic ?? '-' }}</div>
                  <div class="text-2xs text-success font-semibold mt-1 flex items-center gap-1.5">
                    <span class="w-1.5 h-1.5 rounded-full bg-success"></span>
                    <CountUpNumber v-if="bestPost" :value="`${bestPost.views.toLocaleString()} views`" />
                    <span v-else>-</span>
                  </div>
                </div>
              </div>

              <!-- #1 Spotlight Hero Card -->
              <div v-if="bestPost" class="card-fade rounded-2xl p-5 shadow-xs flex flex-col md:flex-row md:items-center justify-between gap-4">
                <div class="flex items-start gap-4 min-w-0">
                  <div class="w-12 h-12 rounded-xl bg-amber-600 border border-amber-700/40 text-white shadow-sm flex items-center justify-center font-extrabold text-xl font-mono flex-none">
                    #1
                  </div>
                  <div class="flex flex-col gap-1 min-w-0">
                    <div class="flex items-center gap-2 flex-wrap">
                      <span class="text-xs uppercase font-extrabold tracking-wider text-muted">Monthly Champion</span>
                      <span v-if="bestPost.post.pillar" class="inline-flex px-2 py-0.5 rounded text-2xs font-semibold bg-wash text-muted border border-faint">
                        {{ bestPost.post.pillar }}
                      </span>
                      <span v-if="bestPost.post.format" class="inline-flex px-2 py-0.5 rounded text-2xs font-medium text-muted">
                        {{ bestPost.post.format }}
                      </span>
                    </div>
                    <div class="text-lg sm:text-xl font-bold text-ink truncate" :title="bestPost.post.topic">
                      {{ bestPost.post.topic }}
                    </div>
                    <div v-if="bestPost.post.hook || bestPost.post.caption" class="text-xs text-muted line-clamp-1 italic">
                      "{{ bestPost.post.hook || bestPost.post.caption }}"
                    </div>
                    <div v-if="bestPost.post.platforms?.length" class="flex items-center gap-1.5 mt-1">
                      <PlatformLogo v-for="pl in bestPost.post.platforms" :key="pl" :name="pl" :size="14" />
                    </div>
                  </div>
                </div>

                <div class="flex items-center gap-6 border-t md:border-t-0 md:border-l border-faint pt-3 md:pt-0 md:pl-6 flex-none">
                  <div class="flex flex-col">
                    <span class="text-2xs uppercase font-bold text-muted">Total Views</span>
                    <span class="text-2xl font-extrabold font-mono text-ink">
                      <CountUpNumber :value="bestPost.views.toLocaleString()" />
                    </span>
                  </div>
                  <div class="flex flex-col">
                    <span class="text-2xs uppercase font-bold text-muted">Likes</span>
                    <span class="text-2xl font-extrabold font-mono text-accent">
                      <CountUpNumber :value="postLikes(bestPost.post.id).toLocaleString()" />
                    </span>
                  </div>
                  <button type="button" class="btn text-xs py-1.5 px-3.5 font-semibold" @click="askEdit">
                    Edit
                  </button>
                </div>
              </div>

              <!-- Top 5 Rankings Table -->
              <div class="flex flex-col gap-2">
                <span class="text-xs font-bold uppercase tracking-wider text-muted">Full Rankings</span>
                <TableFrame>
                  <thead>
                    <tr>
                      <th class="w-[50px] text-center">Rank</th>
                      <th>Topic & Details</th>
                      <th class="w-[100px]">Status</th>
                      <th class="num w-[120px]">Views</th>
                      <th class="num w-[100px]">Likes</th>
                      <th class="w-[90px] text-right"></th>
                    </tr>
                  </thead>
                  <tbody>
                    <tr v-for="(t, idx) in stats?.top5 ?? []" :key="t.post.id">
                      <td class="text-center">
                        <span class="inline-flex items-center justify-center w-6 h-6 rounded-md font-mono text-xs font-extrabold"
                          :class="idx === 0 ? 'bg-amber-600 text-white shadow-2xs border border-amber-700/40' : idx === 1 ? 'bg-slate-600 text-white shadow-2xs border border-slate-700/40' : idx === 2 ? 'bg-amber-800 text-white shadow-2xs border border-amber-900/40' : 'bg-ink text-white shadow-2xs border border-ink'">
                          #{{ idx + 1 }}
                        </span>
                      </td>
                      <td>
                        <div class="flex flex-col gap-0.5">
                          <strong class="font-bold text-ink text-sm">{{ t.post.topic }}</strong>
                          <div class="flex items-center gap-2 text-2xs text-muted flex-wrap">
                            <span v-if="t.post.pillar" class="font-medium text-muted">{{ t.post.pillar }}</span>
                            <span v-if="t.post.format">· {{ t.post.format }}</span>
                            <span v-if="t.post.date">· {{ t.post.date }}</span>
                            <div v-if="t.post.platforms?.length" class="inline-flex items-center gap-1 ml-1">
                              <PlatformLogo v-for="pl in t.post.platforms" :key="pl" :name="pl" :size="12" />
                            </div>
                          </div>
                        </div>
                      </td>
                      <td>
                        <StatusChip :status="t.post.status" />
                      </td>
                      <td class="num font-bold font-mono text-ink text-sm">
                        <CountUpNumber :value="t.views.toLocaleString('en-US')" />
                      </td>
                      <td class="num font-mono text-muted text-sm">
                        <CountUpNumber :value="postLikes(t.post.id).toLocaleString('en-US')" />
                      </td>
                      <td class="text-right">
                        <button type="button" class="btn text-xs py-1 px-3 w-auto inline-flex items-center justify-center font-semibold hover:border-ink" @click="askEdit">
                          Edit
                        </button>
                      </td>
                    </tr>
                  </tbody>
                </TableFrame>
              </div>
            </section>
          </template>

          <template v-else>
            <div class="grid6 dash-kpi-grid grid grid-cols-2 md:grid-cols-3 gap-3">
              <KpiCard label="Total" :value="stats.total" hint="topics this month" :spark="trendData.sparks.total" to="/plan?tab=monthly" />
              <KpiCard label="Posted" :value="stats.posted" :hint="`${postedPct}% of all`" :trend="`${postedPct}%`" :spark="trendData.sparks.posted" to="/plan?tab=monthly" />
              <KpiCard label="Pending" :value="stats.pending" hint="to start" :spark="trendData.sparks.pending" to="/plan?tab=monthly" />
              <KpiCard label="WIP" :value="stats.wip" hint="in design or dev" :spark="trendData.sparks.wip" to="/plan?tab=monthly" />
              <KpiCard label="Views" :value="stats.views.toLocaleString('en-US')" hint="this month" :trend="trendData.trends.views" :spark="trendData.sparks.views" to="/dashboard?tab=performance" />
              <KpiCard label="Likes" :value="stats.likes.toLocaleString('en-US')" hint="this month" :trend="trendData.trends.likes" :spark="trendData.sparks.likes" to="/dashboard?tab=performance" />
            </div>

            <!-- Views & Engagement Line Chart -->
            <div class="mt-6">
              <div class="flex items-center justify-between mb-2">
                <h2 class="m-0 text-md font-bold tracking-tight normal-case [font-family:var(--font)]">Views & Engagement Growth</h2>
              </div>
              <LineChart :labels="trendData.labels" :series="trendData.series" :height="160" />
            </div>

            <div class="mt-6">
              <div class="flex items-center justify-between mb-2">
                <h2 class="m-0 text-md font-bold tracking-tight normal-case [font-family:var(--font)]">Planned vs posted · by pillar</h2>
                <RouterLink class="text-xs font-semibold text-muted hover:text-ink transition-colors flex items-center gap-1" :to="`/plan?tab=monthly&m=${month}`">
                  <span>View in Planner</span>
                  <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round">
                    <path d="M7 17L17 7M17 7H7M17 7V17" />
                  </svg>
                </RouterLink>
              </div>
              <BarChart :items="pillarBars" @select-item="(it) => openBreakdown('pillar', it)" />
            </div>

            <div class="grid2 mt-5">
              <div>
                <div class="flex items-center justify-between mb-2">
                  <h2 class="m-0 text-md font-bold tracking-tight normal-case [font-family:var(--font)]" style="margin-top: 0;">By platform</h2>
                </div>
                <BarChart :items="platformBars" @select-item="(it) => openBreakdown('platform', it)" />
              </div>
              <div>
                <div class="flex items-center justify-between mb-2">
                  <h2 class="m-0 text-md font-bold tracking-tight normal-case [font-family:var(--font)]" style="margin-top: 0;">By status</h2>
                </div>
                <BarChart :items="statusBars" @select-item="(it) => openBreakdown('status', it)" />
              </div>
            </div>
          </template>
        </template>
      </template>
      </div>
    </div>

    <ProtectedModal :open="guardOpen" field="Top-5 views cell" @cancel="guardOpen = false" @confirm="goMaster" />

    <!-- Category Drill-down Posts Sheet (Bottom Sheet on mobile, Right Drawer on desktop) -->
    <Drawer
      v-model="breakdownSheetOpen"
      :side="isMobile ? 'bottom' : 'right'"
      :width="'460px'"
      :title="breakdown ? `${breakdown.type === 'pillar' ? 'Pillar' : breakdown.type === 'platform' ? 'Platform' : 'Status'}: ${breakdown.item.label}` : 'Category Topics'"
    >
      <div v-if="breakdown" class="flex flex-col gap-4 text-left p-1">
        <!-- Summary KPI Capsule -->
        <div class="p-3.5 rounded-xl bg-wash border border-line flex flex-col gap-2.5">
          <div class="flex items-center justify-between gap-2">
            <div class="flex items-center gap-2">
              <span v-if="breakdown.item.platform" class="flex-none" :style="{ color: breakdown.item.color }">
                <PlatformLogo :name="breakdown.item.platform" :size="16" />
              </span>
              <span v-else class="w-3 h-3 rounded-full flex-none shadow-2xs" :style="{ background: breakdown.item.color }" />
              <strong class="text-sm font-bold text-ink">{{ breakdown.item.label }}</strong>
            </div>
            <div class="flex items-center gap-1.5">
              <span class="text-xs font-mono font-bold text-ink">{{ breakdown.item.value }} topics</span>
              <span class="text-2xs font-semibold px-2 py-0.5 rounded-full bg-black text-white">
                {{ breakdown.item.sublabel || `${breakdown.item.value} total` }}
              </span>
            </div>
          </div>

          <!-- Sub-filters inside sheet: All / Done / In progress -->
          <div class="flex items-center gap-1.5 pt-2 border-t border-line/60">
            <button
              type="button"
              class="px-2.5 py-1 rounded-full text-2xs font-semibold transition-all cursor-pointer select-none"
              :class="breakdownSubFilter === 'all' ? 'bg-ink text-white font-bold shadow-xs' : 'text-muted hover:text-ink hover:bg-surface/80'"
              @click="breakdownSubFilter = 'all'"
            >
              All ({{ breakdownPosts.length }})
            </button>
            <button
              type="button"
              class="px-2.5 py-1 rounded-full text-2xs font-semibold transition-all cursor-pointer select-none"
              :class="breakdownSubFilter === 'done' ? 'bg-emerald-600 text-white font-bold shadow-xs' : 'text-muted hover:text-ink hover:bg-surface/80'"
              @click="breakdownSubFilter = 'done'"
            >
              Done
            </button>
            <button
              type="button"
              class="px-2.5 py-1 rounded-full text-2xs font-semibold transition-all cursor-pointer select-none"
              :class="breakdownSubFilter === 'pending' ? 'bg-amber-600 text-white font-bold shadow-xs' : 'text-muted hover:text-ink hover:bg-surface/80'"
              @click="breakdownSubFilter = 'pending'"
            >
              In Progress
            </button>
          </div>
        </div>

        <!-- Topics List -->
        <div class="flex flex-col gap-2">
          <span class="text-2xs font-bold uppercase tracking-wider text-muted">Scheduled Topics for {{ month }}/{{ year }}</span>
          
          <div v-if="!breakdownPosts.length" class="p-8 border border-dashed border-line rounded-xl text-center text-muted text-xs">
            No topics in this category match the filter.
          </div>

          <div
            v-for="post in breakdownPosts"
            :key="post.id"
            class="group p-3 rounded-xl border border-line bg-surface hover:border-ink hover:bg-wash/40 transition-all cursor-pointer flex flex-col gap-2"
            role="button"
            tabindex="0"
            @click="goToPost(post)"
            @keydown.enter="goToPost(post)"
          >
            <div class="flex items-start justify-between gap-2">
              <strong class="text-xs font-bold text-ink leading-snug line-clamp-2">{{ post.topic }}</strong>
              <StatusChip :status="post.status" class="flex-none scale-90 origin-top-right" />
            </div>

            <div class="flex items-center justify-between text-2xs text-muted pt-1 border-t border-faint">
              <div class="flex items-center gap-1.5 flex-wrap">
                <span v-if="post.date" class="font-mono text-ink font-medium">{{ post.date }}</span>
                <span v-if="post.format">· {{ post.format }}</span>
                <div v-if="post.platforms?.length" class="inline-flex items-center gap-1 ml-1">
                  <PlatformLogo v-for="pl in post.platforms" :key="pl" :name="pl" :size="11" />
                </div>
              </div>
              <span class="text-ink font-semibold flex items-center gap-0.5 opacity-0 group-hover:opacity-100 transition-opacity">
                Edit ↗
              </span>
            </div>
          </div>
        </div>
      </div>

      <template #footer>
        <div class="flex items-center justify-between w-full gap-2">
          <button type="button" class="btn text-xs py-2 px-3.5 cursor-pointer" @click="breakdownSheetOpen = false">
            Close
          </button>
          <button
            type="button"
            class="btn btn-primary text-xs py-2 px-4 font-semibold cursor-pointer shadow-xs inline-flex items-center gap-1.5"
            @click="goToPlanner"
          >
            <span>Open Planner</span>
            <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round">
              <path d="M5 12h14M12 5l7 7-7 7" />
            </svg>
          </button>
        </div>
      </template>
    </Drawer>
  </div>
</template>
