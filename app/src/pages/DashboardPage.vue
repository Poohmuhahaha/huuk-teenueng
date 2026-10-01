<script setup lang="ts">
import { computed, ref } from 'vue'
import { useRouter } from 'vue-router'
import { useAds, useDashboard } from '@/core/queries'
import { MONTHS } from '@/mock/db'
import { guardSuppressed } from '@/core/protected'
import { t } from '@/core/i18n'
import CarouselTabs from '@/components/ui/CarouselTabs.vue'
import KpiCard from '@/components/ui/KpiCard.vue'
import BarChart from '@/components/ui/BarChart.vue'
import Chip from '@/components/ui/Chip.vue'
import LiveSection from '@/components/live/LiveSection.vue'
import ProtectedModal from '@/components/overlays/ProtectedModal.vue'

const router = useRouter()
const month = ref<number>(2)
const { stats, postsQ, metricsQ } = useDashboard(month)
const { data: adsView } = useAds()
const guardOpen = ref(false)

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
  <h1>Dashboard</h1>
  <CarouselTabs :items="MONTHS" v-model="month" />
  <div v-if="postsQ.isError.value || metricsQ.isError.value" class="card">
    <p class="muted">Could not load dashboard data: {{ (postsQ.error.value ?? metricsQ.error.value)?.message }}</p>
  </div>
  <div v-else-if="!stats" class="muted">Computing…</div>
  <div v-else>
    <div class="grid6 mt">
      <KpiCard label="Total" :value="stats.total" />
      <KpiCard label="Posted" :value="stats.posted" />
      <KpiCard label="Pending" :value="stats.pending" />
      <KpiCard label="WIP" :value="stats.wip" />
      <KpiCard label="Views" :value="stats.views.toLocaleString('en-US')" />
      <KpiCard label="Likes" :value="stats.likes.toLocaleString('en-US')" />
    </div>
    <LiveSection :limit="3" />
    <section v-if="activeCampaigns.length || adsAlerts.length" class="ads-glance">
      <div class="row">
        <h2 style="margin: 0;">{{ t('ads.active') }}</h2>
        <RouterLink class="btn ads-open" to="/ads">{{ t('ads.open') }}</RouterLink>
      </div>
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
    </section>
    <h2>Planned vs posted · by pillar</h2>
    <BarChart :items="pillarBars" />
    <div class="grid2 mt">
      <div class="panel">
        <strong>By platform</strong>
        <div class="mt"><Chip v-for="(n, p) in stats.platformCounts" :key="p" :label="`${p} · ${n}`" /></div>
      </div>
      <div class="panel">
        <strong>By status</strong>
        <div class="mt"><Chip v-for="(n, s) in stats.statusCounts" :key="s" :label="`${s} · ${n}`" :dark="s === 'Done'" /></div>
      </div>
    </div>
    <h2>Top 5 by views</h2>
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
  </div>
  <ProtectedModal :open="guardOpen" field="Top-5 views cell" @cancel="guardOpen = false" @confirm="goMaster" />
</template>

<style scoped>
.ads-glance {
  margin-top: 24px;
}
.ads-open {
  margin-left: auto;
}
.ads-alerts {
  margin: 10px 0 0;
  padding-left: 18px;
  color: var(--warning, #b26a00);
  font-size: 13.5px;
}
</style>
