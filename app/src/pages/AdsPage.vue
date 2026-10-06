<script setup lang="ts">
// Meta Ads: the workspace's ad accounts, campaigns, ad sets and ads with their
// settings and last-30-days performance, mirrored from the Graph API by the
// server (apps/server/src/ads.rs). Management (pause/resume, budget, duplicate,
// boost) only unlocks after the explicit opt-in and is recorded in the audit
// trail — every write also requires the `platforms.manage` permission.
import { computed, ref } from 'vue'
import {
  useAds, useLive, usePlatforms, usePermission,
  useSyncAds, useSetAdsManage, useAdCampaignStatus, useAdCampaignBudget,
  useDuplicateAdCampaign, useCreateAdBoost,
} from '@/core/queries'
import { beginOAuth } from '@/core/oauth'
import { t } from '@/core/i18n'
import InfoTip from '@/components/ui/InfoTip.vue'
import type { AdCampaign } from '@/mock/db'

const { data: view, isPending, isError, error: loadError } = useAds()
const { data: live } = useLive()
const { data: connections } = usePlatforms()
const { can } = usePermission()

const mayManage = computed(() => can('platforms.manage'))
const canWrite = computed(() => mayManage.value && (view.value?.canManage ?? false))

const sync = useSyncAds()
const setManage = useSetAdsManage()
const status = useAdCampaignStatus()
const budget = useAdCampaignBudget()
const duplicate = useDuplicateAdCampaign()
const boost = useCreateAdBoost()

const actionError = ref('')
const notice = ref('')

function reportError(e: unknown): void {
  actionError.value = e instanceof Error ? e.message : String(e)
}
function clearMessages(): void {
  actionError.value = ''
  notice.value = ''
}

// ---- reconnect (connected slot without a usable token) ----
const meta = computed(() => connections.value?.find((c) => c.id === 'meta') ?? null)
const needsReconnect = computed(
  () => meta.value?.status === 'connected' && meta.value?.hasToken === false,
)
const reconnectError = ref('')
const reconnecting = ref(false)
async function onReconnect(): Promise<void> {
  if (!mayManage.value || reconnecting.value) return
  reconnecting.value = true
  reconnectError.value = ''
  const error = await beginOAuth('meta')
  if (error) reconnectError.value = error
  reconnecting.value = false
}

// ---- mirror slices ----
const accounts = computed(() => view.value?.accounts ?? [])
const accountId = ref('')
const activeAccount = computed(
  () => accounts.value.find((a) => a.id === accountId.value) ?? accounts.value[0] ?? null,
)
const currency = computed(() => activeAccount.value?.currency ?? '')

const campaigns = computed(() => {
  const rows = view.value?.campaigns ?? []
  if (!activeAccount.value) return rows
  return rows.filter((c) => c.accountId === activeAccount.value?.id)
})
const insightOf = (campaignId: string) =>
  view.value?.insights.find((i) => i.campaignId === campaignId) ?? null
const adsetsOf = (campaignId: string) =>
  (view.value?.adsets ?? []).filter((s) => s.campaignId === campaignId)
const adsOf = (adsetId: string) => (view.value?.ads ?? []).filter((a) => a.adsetId === adsetId)

const totals = computed(() => {
  const rows = campaigns.value.map((c) => insightOf(c.id)).filter((i) => i !== null)
  const spend = rows.reduce((s, i) => s + (i?.spend ?? 0), 0)
  const results = rows.reduce((s, i) => s + (i?.results ?? 0), 0)
  const clicks = rows.reduce((s, i) => s + (i?.clicks ?? 0), 0)
  const impressions = rows.reduce((s, i) => s + (i?.impressions ?? 0), 0)
  return {
    spend,
    results,
    ctr: impressions ? (clicks / impressions) * 100 : 0,
    cpc: clicks ? spend / clicks : 0,
    label: rows.find((i) => i?.resultLabel)?.resultLabel ?? '',
  }
})

const openId = ref('')
function toggle(id: string): void {
  openId.value = openId.value === id ? '' : id
}

function money(minor: number): string {
  const value = (minor / 100).toLocaleString('en-US', {
    minimumFractionDigits: 2,
    maximumFractionDigits: 2,
  })
  return currency.value ? `${value} ${currency.value}` : value
}
function budgetLabel(campaign: AdCampaign): string {
  if (campaign.lifetimeBudget) return `${t('ads.lifetime')} ${money(campaign.lifetimeBudget)}`
  if (campaign.dailyBudget) return `${t('ads.daily')} ${money(campaign.dailyBudget)}`
  return '—'
}
function fmt(n: number): string {
  return n.toLocaleString('en-US')
}
function statusClass(value: string): string {
  if (value === 'ACTIVE') return 'badge-success'
  if (value === 'PAUSED') return 'badge-warning'
  return 'badge-danger'
}

// ---- management actions ----
async function run(action: () => Promise<unknown>, done = ''): Promise<void> {
  clearMessages()
  try {
    await action()
    notice.value = done
  } catch (e) {
    reportError(e)
  }
}
function toggleStatus(campaign: AdCampaign): void {
  const next = campaign.status === 'ACTIVE' ? 'PAUSED' : 'ACTIVE'
  void run(
    () => status.mutateAsync({ id: campaign.id, status: next }),
    t('ads.saved'),
  )
}
function onDuplicate(campaign: AdCampaign): void {
  void run(() => duplicate.mutateAsync(campaign.id), t('ads.duplicated'))
}

const budgetFor = ref<AdCampaign | null>(null)
const budgetDaily = ref('')
function openBudget(campaign: AdCampaign): void {
  clearMessages()
  budgetFor.value = campaign
  budgetDaily.value = campaign.dailyBudget ? (campaign.dailyBudget / 100).toFixed(2) : ''
}
function saveBudget(): void {
  const campaign = budgetFor.value
  if (!campaign) return
  const major = Number.parseFloat(budgetDaily.value)
  if (!Number.isFinite(major) || major <= 0) {
    actionError.value = t('ads.budgetInvalid')
    return
  }
  const minor = Math.round(major * 100)
  void run(async () => {
    await budget.mutateAsync({ id: campaign.id, dailyBudget: minor })
    budgetFor.value = null
  }, t('ads.saved'))
}

// ---- opt-in ----
const manageConfirm = ref(false)
function onToggleManage(): void {
  clearMessages()
  if (view.value?.canManage) {
    void run(() => setManage.mutateAsync(false), t('ads.manageOffNotice'))
  } else {
    manageConfirm.value = true
  }
}
function confirmManage(): void {
  manageConfirm.value = false
  void run(() => setManage.mutateAsync(true), t('ads.manageOnNotice'))
}

// ---- boost a post ----
const boostOpen = ref(false)
const boostForm = ref({
  name: '',
  objective: 'OUTCOME_TRAFFIC',
  dailyBudget: '5.00',
  days: 7,
  countries: 'TH',
  storyId: '',
})
const boostPosts = computed(() => (live.value?.posts ?? []).filter((p) => p.platform === 'meta'))
const OBJECTIVES = [
  'OUTCOME_AWARENESS', 'OUTCOME_TRAFFIC', 'OUTCOME_ENGAGEMENT', 'OUTCOME_LEADS', 'OUTCOME_SALES',
]
function openBoost(storyId = ''): void {
  clearMessages()
  boostForm.value = {
    name: '',
    objective: 'OUTCOME_TRAFFIC',
    dailyBudget: '5.00',
    days: 7,
    countries: activeAccount.value?.currency === 'THB' ? 'TH' : '',
    storyId: storyId || boostPosts.value[0]?.id || '',
  }
  boostOpen.value = true
}
function submitBoost(): void {
  const form = boostForm.value
  const major = Number.parseFloat(form.dailyBudget)
  if (!form.name.trim()) {
    actionError.value = t('ads.boostNameRequired')
    return
  }
  if (!Number.isFinite(major) || major <= 0) {
    actionError.value = t('ads.budgetInvalid')
    return
  }
  if (!form.storyId) {
    actionError.value = t('ads.boostPostRequired')
    return
  }
  const countries = form.countries
    .split(',')
    .map((c) => c.trim().toUpperCase())
    .filter(Boolean)
  if (!countries.length) {
    actionError.value = t('ads.boostCountriesRequired')
    return
  }
  void run(async () => {
    await boost.mutateAsync({
      name: form.name.trim(),
      objective: form.objective,
      dailyBudget: Math.round(major * 100),
      days: form.days,
      countries,
      storyId: form.storyId,
    })
    boostOpen.value = false
  }, t('ads.boostCreated'))
}

const audit = computed(() => view.value?.audit ?? [])
</script>

<template>
  <div class="adspage">
    <h1>{{ t('ads.title') }}<InfoTip :text="t('ads.subtitle')" /></h1>
    <div class="tbl-tools">
      <span v-if="view?.fetchedAt" class="muted">{{ t('ads.synced') }} {{ new Date(view.fetchedAt * 1000).toLocaleString() }}</span>
      <button class="btn btn-primary ads-sync" :disabled="!mayManage || sync.isPending.value"
        :title="mayManage ? '' : t('auth.noPerm')" @click="run(() => sync.mutateAsync(), t('ads.syncedNow'))">
        {{ sync.isPending.value ? t('ads.syncing') : t('ads.sync') }}
      </button>
    </div>
    <p v-if="isError" class="card muted">{{ loadError?.message }}</p>
    <p v-else-if="reconnectError" class="autherr" role="alert">{{ reconnectError }}</p>
    <p v-else-if="sync.error.value" class="autherr" role="alert">{{ sync.error.value.message }}</p>
    <p v-else-if="view?.error" class="autherr" role="alert">{{ view.error }}</p>
    <p v-if="actionError" class="autherr" role="alert">{{ actionError }}</p>
    <p v-else-if="notice" class="oknote">{{ notice }}</p>

    <!-- opt-in row -->
    <div class="row ads-manage">
      <span class="badge" :class="view?.canManage ? 'badge-success' : ''">
        {{ view?.canManage ? t('ads.manageOn') : t('ads.manageOff') }}
      </span>
      <button class="btn ads-manage-toggle" :disabled="!mayManage || setManage.isPending.value"
        :title="mayManage ? '' : t('auth.noPerm')" @click="onToggleManage">
        {{ view?.canManage ? t('ads.manageDisable') : t('ads.manageEnable') }}
      </button>
      <InfoTip :text="t('ads.manageHint')" />
    </div>

    <p v-if="isPending" class="muted">{{ t('common.loading') }}</p>

    <!-- empty states -->
    <div v-else-if="!campaigns.length" class="panel ads-empty">
      <p class="muted">
        {{ needsReconnect ? t('ads.reconnectNeeded')
          : accounts.length ? t('ads.empty') : t('ads.emptyAccounts') }}
      </p>
      <button v-if="needsReconnect" class="btn btn-primary" :disabled="!mayManage || reconnecting"
        @click="onReconnect">
        {{ reconnecting ? t('slogin.connecting') : t('ads.reconnect') }}
      </button>
    </div>

    <template v-else>
      <!-- account picker + summary -->
      <div class="row ads-accounts">
        <label v-if="accounts.length > 1" class="ads-field">
          <span class="muted">{{ t('ads.account') }}</span>
          <select v-model="accountId" class="ads-input">
            <option v-for="a in accounts" :key="a.id" :value="a.id">
              {{ a.name }} ({{ a.currency }})
            </option>
          </select>
        </label>
        <span v-else-if="activeAccount" class="chip">
          {{ activeAccount.name }} · {{ activeAccount.currency }} · {{ activeAccount.timezone }}
        </span>
        <button class="btn ads-boost" :disabled="!canWrite || !boostPosts.length"
          :title="canWrite ? '' : t('ads.manageNeeded')" @click="openBoost()">
          {{ t('ads.boost') }}
        </button>
      </div>

      <div class="ads-stats">
        <div class="panel ads-stat">
          <span class="muted">{{ t('ads.spend') }}</span>
          <strong>{{ money(totals.spend * 100) }}</strong>
        </div>
        <div class="panel ads-stat">
          <span class="muted">{{ t('ads.results') }}<template v-if="totals.label"> · {{ totals.label }}</template></span>
          <strong>{{ fmt(totals.results) }}</strong>
        </div>
        <div class="panel ads-stat">
          <span class="muted">{{ t('ads.ctr') }}</span>
          <strong>{{ totals.ctr.toFixed(2) }}%</strong>
        </div>
        <div class="panel ads-stat">
          <span class="muted">{{ t('ads.cpc') }}</span>
          <strong>{{ money(totals.cpc * 100) }}</strong>
        </div>
      </div>

      <!-- campaign table -->
      <div class="tblwrap">
        <table class="tbl ads-tbl">
          <thead>
            <tr>
              <th>{{ t('ads.campaign') }}</th>
              <th>{{ t('ads.objective') }}</th>
              <th>{{ t('ads.status') }}</th>
              <th>{{ t('ads.budget') }}</th>
              <th class="num">{{ t('ads.spend') }}</th>
              <th class="num">{{ t('ads.results') }}</th>
              <th class="num">{{ t('ads.ctr') }}</th>
              <th class="num">{{ t('ads.cpc') }}</th>
              <th></th>
            </tr>
          </thead>
          <tbody>
            <template v-for="c in campaigns" :key="c.id">
              <tr :class="{ selected: openId === c.id }" @click="toggle(c.id)">
                <td>
                  <button class="ads-link" @click.stop="toggle(c.id)">{{ c.name }}</button>
                </td>
                <td class="muted">{{ c.objective.replace('OUTCOME_', '') }}</td>
                <td><span class="badge" :class="statusClass(c.status)">{{ c.status }}</span></td>
                <td>{{ budgetLabel(c) }}</td>
                <td class="num">{{ insightOf(c.id) ? money(insightOf(c.id)!.spend * 100) : '—' }}</td>
                <td class="num">{{ insightOf(c.id) ? fmt(insightOf(c.id)!.results) : '—' }}</td>
                <td class="num">{{ insightOf(c.id) ? insightOf(c.id)!.ctr.toFixed(2) + '%' : '—' }}</td>
                <td class="num">{{ insightOf(c.id) ? money(insightOf(c.id)!.cpc * 100) : '—' }}</td>
                <td>
                  <button class="btn ads-toggle" :disabled="!canWrite || status.isPending.value"
                    :title="canWrite ? '' : t('ads.manageNeeded')" @click.stop="toggleStatus(c)">
                    {{ c.status === 'ACTIVE' ? t('ads.pause') : t('ads.resume') }}
                  </button>
                </td>
              </tr>

              <!-- expanded: settings + ad sets + ads + actions -->
              <tr v-if="openId === c.id" :key="`${c.id}-detail`" class="ads-detailrow">
                <td colspan="9">
                  <div class="ads-detail">
                    <div class="grid2">
                      <div>
                        <h4>{{ t('ads.settings') }}</h4>
                        <dl class="ads-dl">
                          <dt>{{ t('ads.objective') }}</dt><dd>{{ c.objective }}</dd>
                          <dt>{{ t('ads.bidStrategy') }}</dt><dd>{{ c.bidStrategy || '—' }}</dd>
                          <dt>{{ t('ads.schedule') }}</dt>
                          <dd>{{ c.startTime?.slice(0, 10) || '—' }} → {{ c.stopTime?.slice(0, 10) || '—' }}</dd>
                          <dt>{{ t('ads.budgetRemaining') }}</dt>
                          <dd>{{ c.budgetRemaining ? money(c.budgetRemaining) : '—' }}</dd>
                          <dt>{{ t('ads.special') }}</dt>
                          <dd>{{ c.specialAdCategories.join(', ') || '—' }}</dd>
                        </dl>
                      </div>
                      <div>
                        <h4>{{ t('ads.actions') }}</h4>
                        <div class="row">
                          <button class="btn" :disabled="!canWrite" @click="openBudget(c)">
                            {{ t('ads.editBudget') }}
                          </button>
                          <button class="btn" :disabled="!canWrite || duplicate.isPending.value"
                            @click="onDuplicate(c)">
                            {{ t('ads.duplicate') }}
                          </button>
                          <button class="btn" :disabled="!canWrite || !boostPosts.length"
                            @click="openBoost(adsOf(adsetsOf(c.id)[0]?.id ?? '')[0]?.storyId ?? '')">
                            {{ t('ads.boost') }}
                          </button>
                        </div>
                        <p v-if="!canWrite" class="muted">{{ t('ads.manageNeeded') }}</p>
                      </div>
                    </div>

                    <h4>{{ t('ads.adsets') }}</h4>
                    <div v-for="s in adsetsOf(c.id)" :key="s.id" class="panel ads-adset">
                      <div class="row">
                        <strong>{{ s.name }}</strong>
                        <span class="badge" :class="statusClass(s.status)">{{ s.status }}</span>
                        <span class="muted">{{ s.optimizationGoal }} · {{ s.billingEvent }}</span>
                      </div>
                      <p class="muted ads-line">
                        {{ t('ads.targeting') }}: {{ s.targeting || '—' }}
                        <template v-if="s.promotedObject"> · {{ t('ads.promoted') }}: {{ s.promotedObject }}</template>
                      </p>
                      <ul class="ads-ads">
                        <li v-for="a in adsOf(s.id)" :key="a.id">
                          <span class="badge" :class="statusClass(a.status)">{{ a.status }}</span>
                          {{ a.creativeTitle || a.name }}
                          <span v-if="a.storyId" class="muted">· {{ t('ads.boostedPost') }} {{ a.storyId }}</span>
                        </li>
                      </ul>
                      <p v-if="!adsOf(s.id).length" class="muted">{{ t('ads.noAds') }}</p>
                    </div>
                    <p v-if="!adsetsOf(c.id).length" class="muted">{{ t('ads.noAdsets') }}</p>
                  </div>
                </td>
              </tr>
            </template>
          </tbody>
        </table>
      </div>
    </template>

    <!-- audit trail -->
    <section v-if="audit.length" class="ads-audit">
      <h3>{{ t('ads.audit') }}</h3>
      <ul class="muted">
        <li v-for="(entry, i) in audit" :key="i">
          {{ entry.at }} · {{ entry.actor }} · {{ entry.action }} · {{ entry.target }}
          <template v-if="entry.detail"> · {{ entry.detail }}</template>
        </li>
      </ul>
    </section>

    <!-- budget dialog -->
    <div v-if="budgetFor" class="ads-overlay" @click.self="budgetFor = null">
      <div class="panel ads-dialog" role="dialog" aria-modal="true">
        <h3>{{ t('ads.editBudget') }}</h3>
        <p class="muted">{{ budgetFor.name }}</p>
        <label class="ads-field">
          <span>{{ t('ads.dailyBudget') }} ({{ currency }})</span>
          <input v-model="budgetDaily" class="ads-input" type="number" min="0" step="0.01" />
        </label>
        <div class="row mt">
          <button class="btn btn-primary" :disabled="budget.isPending.value" @click="saveBudget">
            {{ t('ads.save') }}
          </button>
          <button class="btn" @click="budgetFor = null">{{ t('ads.cancel') }}</button>
        </div>
      </div>
    </div>

    <!-- opt-in confirm -->
    <div v-if="manageConfirm" class="ads-overlay" @click.self="manageConfirm = false">
      <div class="panel ads-dialog" role="dialog" aria-modal="true">
        <h3>{{ t('ads.manageEnable') }}</h3>
        <p class="muted">{{ t('ads.manageConfirm') }}</p>
        <div class="row mt">
          <button class="btn btn-primary" @click="confirmManage">{{ t('ads.confirm') }}</button>
          <button class="btn" @click="manageConfirm = false">{{ t('ads.cancel') }}</button>
        </div>
      </div>
    </div>

    <!-- boost dialog -->
    <div v-if="boostOpen" class="ads-overlay" @click.self="boostOpen = false">
      <div class="panel ads-dialog" role="dialog" aria-modal="true">
        <h3>{{ t('ads.boostTitle') }}<InfoTip :text="t('ads.boostHint')" /></h3>
        <label class="ads-field">
          <span>{{ t('ads.boostName') }}</span>
          <input v-model="boostForm.name" class="ads-input" maxlength="120" />
        </label>
        <label class="ads-field">
          <span>{{ t('ads.objective') }}</span>
          <select v-model="boostForm.objective" class="ads-input">
            <option v-for="o in OBJECTIVES" :key="o" :value="o">{{ o }}</option>
          </select>
        </label>
        <div class="grid2">
          <label class="ads-field">
            <span>{{ t('ads.dailyBudget') }} ({{ currency }})</span>
            <input v-model="boostForm.dailyBudget" class="ads-input" type="number" min="0" step="0.01" />
          </label>
          <label class="ads-field">
            <span>{{ t('ads.boostDays') }}</span>
            <input v-model.number="boostForm.days" class="ads-input" type="number" min="1" max="90" />
          </label>
        </div>
        <label class="ads-field">
          <span>{{ t('ads.boostCountries') }}</span>
          <input v-model="boostForm.countries" class="ads-input" placeholder="TH" />
        </label>
        <label class="ads-field">
          <span>{{ t('ads.boostPost') }}</span>
          <select v-model="boostForm.storyId" class="ads-input">
            <option v-for="p in boostPosts" :key="p.id" :value="p.id">
              {{ (p.caption || p.id).slice(0, 60) }}
            </option>
          </select>
        </label>
        <div class="row mt">
          <button class="btn btn-primary" :disabled="boost.isPending.value" @click="submitBoost">
            {{ t('ads.boostCreate') }}
          </button>
          <button class="btn" @click="boostOpen = false">{{ t('ads.cancel') }}</button>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.adspage {
  margin-top: 8px;
}
.ads-head {
  align-items: center;
  gap: 12px;
}
.ads-sync {
  margin-left: 0;
}
.ads-manage {
  margin: 14px 0 6px;
  gap: 10px;
}
.ads-accounts {
  margin: 14px 0 10px;
  gap: 12px;
}
.ads-boost {
  margin-left: 0;
}
.ads-stats {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(160px, 1fr));
  gap: 12px;
  margin-bottom: 14px;
}
.ads-stat {
  display: flex;
  flex-direction: column;
  gap: 4px;
}
.ads-stat strong {
  font-size: 20px;
}
.ads-tbl tbody tr {
  cursor: pointer;
}
.ads-link {
  padding: 0;
  border: 0;
  background: none;
  font: inherit;
  font-weight: 600;
  color: inherit;
  cursor: pointer;
  text-align: left;
}
.ads-link:hover {
  color: var(--accent, #0b7a75);
}
.ads-detailrow td {
  background: var(--surface-2, #fafafa);
}
.ads-detail {
  padding: 6px 2px 10px;
}
.ads-detail h4 {
  margin: 12px 0 6px;
  font-size: 13px;
  text-transform: uppercase;
  letter-spacing: 0.04em;
  color: var(--muted, #777);
}
.ads-dl {
  display: grid;
  grid-template-columns: auto 1fr;
  gap: 4px 12px;
  margin: 0;
  font-size: 13.5px;
}
.ads-dl dt {
  color: var(--muted, #777);
}
.ads-dl dd {
  margin: 0;
}
.ads-adset {
  margin-bottom: 8px;
}
.ads-line {
  margin: 6px 0;
}
.ads-ads {
  margin: 4px 0 0;
  padding-left: 18px;
  display: flex;
  flex-direction: column;
  gap: 4px;
  font-size: 13.5px;
}
.ads-audit {
  margin-top: 20px;
}
.ads-audit ul {
  margin: 6px 0 0;
  padding-left: 18px;
}
.ads-overlay {
  position: fixed;
  inset: 0;
  z-index: 60;
  display: grid;
  place-items: center;
  background: rgba(20, 20, 20, 0.35);
}
.ads-dialog {
  width: min(520px, 92vw);
  max-height: 88vh;
  overflow: auto;
}
.ads-field {
  display: flex;
  flex-direction: column;
  gap: 4px;
  margin-top: 10px;
  font-size: 13.5px;
}
.ads-input {
  padding: 7px 9px;
  border: 1px solid var(--line, #d8d8d8);
  border-radius: 8px;
  background: var(--surface, #fff);
  font: inherit;
}
.ads-empty {
  display: flex;
  flex-direction: column;
  align-items: flex-start;
  gap: 10px;
}
.oknote {
  margin: 10px 0 0;
  font-size: 13.5px;
  font-weight: 600;
  color: var(--success, #1a7f37);
}
</style>
