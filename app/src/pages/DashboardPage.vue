<script setup lang="ts">
import { computed, ref } from 'vue'
import { useRouter } from 'vue-router'
import { useAds, useDashboard } from '@/core/queries'
import { guardSuppressed } from '@/core/protected'
import { t } from '@/core/i18n'
import { useHubTab } from '@/core/subnav'
import { activeMonth } from '@/core/navdate'
import { useWidePane } from '@/composables/useWidePane'
import MonthToggle from '@/components/ui/MonthToggle.vue'
import KpiCard from '@/components/ui/KpiCard.vue'
import BarChart from '@/components/ui/BarChart.vue'
import LiveSection from '@/components/live/LiveSection.vue'
import ProtectedModal from '@/components/overlays/ProtectedModal.vue'

const router = useRouter()
// The deck works on one month: this is the shared value every card follows.
const month = activeMonth
const { stats, postsQ, metricsQ } = useDashboard(month)
const { data: adsView } = useAds()
const guardOpen = ref(false)

const root = ref<HTMLElement | null>(null)
useWidePane(root, 'dash-scroll')

// Hub tab (subnavbar) — sections come from HUB_TABS['/dashboard'].
const { active: section } = useHubTab('/dashboard')

// What actually needs action this month: topics past their date that are not
// posted yet, and posted topics that still have no metric rows.
const todayIso = (() => {
  const d = new Date()
  return `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, '0')}-${String(d.getDate()).padStart(2, '0')}`
})()
const metricPostIds = computed(() => new Set((metricsQ.data.value ?? []).map((m) => m.postId)))
const attention = computed(() => {
  const rows = postsQ.data.value ?? []
  const late = rows.filter((p) => p.date && p.date < todayIso && p.status !== 'Done')
  const noMetrics = rows.filter(
    (p) => p.date && p.date <= todayIso && p.status === 'Done' && !metricPostIds.value.has(p.id),
  )
  const out: { text: string; to: string }[] = []
  if (late.length) {
    out.push({
      text: `${late.length} topic${late.length === 1 ? '' : 's'} past due, not posted yet`,
      to: '/plan?tab=monthly',
    })
  }
  if (noMetrics.length) {
    out.push({
      text: `${noMetrics.length} posted topic${noMetrics.length === 1 ? '' : 's'} still without metrics`,
      to: '/analyze?tab=performance',
    })
  }
  return out
})
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

const pillarBars = computed(() =>
  stats.value ? Object.entries(stats.value.pillarCounts).map(([label, value]) => ({ label, value })) : [],
)
const platformBars = computed(() =>
  stats.value ? Object.entries(stats.value.platformCounts).map(([label, value]) => ({ label, value })) : [],
)
const statusBars = computed(() =>
  stats.value ? Object.entries(stats.value.statusCounts).map(([label, value]) => ({ label, value })) : [],
)

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
      <aside class="dash-side">
        <MonthToggle v-model="month" />
        <div class="dash-quick">
          <RouterLink class="btn btn-primary" to="/plan?tab=monthly">+ New topic</RouterLink>
          <RouterLink class="btn" to="/analyze?tab=performance">Import metrics</RouterLink>
        </div>
        <div v-if="attention.length" class="dash-attention">
          <h3>Needs attention</h3>
          <ul>
            <li v-for="a in attention" :key="a.text">
              <RouterLink :to="a.to">{{ a.text }}</RouterLink>
            </li>
          </ul>
        </div>
      </aside>

      <div class="dash-main">
        <div v-if="postsQ.isError.value || metricsQ.isError.value" class="card">
          <p class="muted">Could not load dashboard data: {{ (postsQ.error.value ?? metricsQ.error.value)?.message }}</p>
        </div>
        <div v-else-if="!stats" class="muted">Computing…</div>
        <template v-else>
          <template v-if="section === 'overview'">
            <div class="grid6">
              <KpiCard label="Total" :value="stats.total" hint="topics this month" to="/plan?tab=monthly" />
              <KpiCard label="Posted" :value="stats.posted" :hint="`${postedPct}% of all`" to="/plan?tab=monthly" />
              <KpiCard label="Pending" :value="stats.pending" hint="to start" to="/plan?tab=monthly" />
              <KpiCard label="WIP" :value="stats.wip" hint="in design or dev" to="/plan?tab=monthly" />
              <KpiCard label="Views" :value="stats.views.toLocaleString('en-US')" hint="this month" to="/analyze?tab=performance" />
              <KpiCard label="Likes" :value="stats.likes.toLocaleString('en-US')" hint="this month" to="/analyze?tab=performance" />
            </div>
            <h2>Planned vs posted · by pillar</h2>
            <BarChart :items="pillarBars" />
            <div class="grid2 mt">
              <div>
                <h2 style="margin-top: 0;">By platform</h2>
                <BarChart :items="platformBars" />
              </div>
              <div>
                <h2 style="margin-top: 0;">By status</h2>
                <BarChart :items="statusBars" />
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
                <RouterLink class="btn ads-open" to="/ads">{{ t('ads.open') }}</RouterLink>
              </div>
              <template v-if="activeCampaigns.length || adsAlerts.length">
                <ul v-if="adsAlerts.length" class="ads-alerts">
                  <li v-for="(a, i) in adsAlerts" :key="i">{{ a }}</li>
                </ul>
                <div class="tblwrap mt">
                  <table class="tbl">
                    <thead><tr><th>{{ t('ads.campaign') }}</th><th>{{ t('ads.status') }}</th><th class="num">{{ t('ads.spend') }}</th></tr></thead>
                    <tbody>
                      <tr v-for="c in activeCampaigns" :key="c.id" style="cursor: default;">
                        <td>{{ c.name }}</td>
                        <td><span class="badge badge-success">{{ c.status }}</span></td>
                        <td class="num">{{ adsMoney(adSpend(c.id) * 100) }}</td>
                      </tr>
                    </tbody>
                  </table>
                </div>
              </template>
              <p v-else class="muted">No active campaigns.</p>
            </section>
          </template>

          <template v-else>
            <h2 style="margin-top: 0;">Top 5 by views</h2>
            <div class="tblwrap">
              <table class="tbl">
                <thead><tr><th>Topic</th><th>Status</th><th class="num">Views</th><th></th></tr></thead>
                <tbody>
                  <tr v-for="t in stats.top5" :key="t.post.id" style="cursor: default;">
                    <td>{{ t.post.topic }}</td>
                    <td>{{ t.post.status }}</td>
                    <td class="num">{{ t.views.toLocaleString('en-US') }}</td>
                    <td><button class="btn" @click="askEdit">Edit</button></td>
                  </tr>
                </tbody>
              </table>
            </div>
          </template>
        </template>
      </div>
    </div>

    <ProtectedModal :open="guardOpen" field="Top-5 views cell" @cancel="guardOpen = false" @confirm="goMaster" />
  </div>
</template>

<style scoped>
.dash-side {
  display: grid;
  gap: 10px;
  align-content: start;
}
.dash-quick {
  display: grid;
  gap: 8px;
}
.dash-attention {
  border-top: 1px solid var(--line);
  padding-top: 12px;
}
.dash-attention h3 {
  margin: 0 0 8px;
  font-size: 11.5px;
  font-weight: 700;
  letter-spacing: 0.08em;
  text-transform: uppercase;
  color: var(--muted);
}
.dash-attention ul {
  margin: 0;
  padding: 0;
  list-style: none;
  display: grid;
  gap: 8px;
}
.dash-attention a {
  font-size: 13px;
  line-height: 1.45;
  text-decoration: underline;
  text-underline-offset: 3px;
  text-decoration-color: var(--faint);
}
.dash-attention a:hover {
  text-decoration-color: var(--ink);
}
.ads-open {
  margin-left: 0;
}
.ads-alerts {
  margin: 10px 0 0;
  padding-left: 18px;
  color: var(--warning, #b26a00);
  font-size: 13.5px;
}
</style>
