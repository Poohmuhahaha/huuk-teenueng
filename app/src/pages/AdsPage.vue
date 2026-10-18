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
import TableFrame from '@/components/tables/TableFrame.vue'
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
  <div class="mt-2">
    <h1>{{ t('ads.title') }}<InfoTip :text="t('ads.subtitle')" /></h1>
    <div class="tbl-tools">
      <span v-if="view?.fetchedAt" class="muted">{{ t('ads.synced') }} {{ new Date(view.fetchedAt * 1000).toLocaleString() }}</span>
      <button class="btn btn-primary ml-0" :disabled="!mayManage || sync.isPending.value"
        :title="mayManage ? '' : t('auth.noPerm')" @click="run(() => sync.mutateAsync(), t('ads.syncedNow'))">
        {{ sync.isPending.value ? t('ads.syncing') : t('ads.sync') }}
      </button>
    </div>
    <p v-if="isError" class="card muted">{{ loadError?.message }}</p>
    <p v-else-if="reconnectError" class="autherr" role="alert">{{ reconnectError }}</p>
    <p v-else-if="sync.error.value" class="autherr" role="alert">{{ sync.error.value.message }}</p>
    <p v-else-if="view?.error" class="autherr" role="alert">{{ view.error }}</p>
    <p v-if="actionError" class="autherr" role="alert">{{ actionError }}</p>
    <p v-else-if="notice" class="mt-2.5 text-sm-minus font-semibold text-success">{{ notice }}</p>

    <!-- opt-in row -->
    <div class="row my-3.5 mb-1.5 gap-2.5">
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
    <div v-else-if="!campaigns.length" class="panel flex flex-col items-start gap-2.5">
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
      <div class="row my-3.5 mb-2.5 gap-3 items-center">
        <label v-if="accounts.length > 1" class="flex flex-col gap-1 text-sm-minus">
          <span class="muted">{{ t('ads.account') }}</span>
          <select v-model="accountId" class="py-1.5 px-2.5 border border-line rounded-lg bg-surface font-inherit">
            <option v-for="a in accounts" :key="a.id" :value="a.id">
              {{ a.name }} ({{ a.currency }})
            </option>
          </select>
        </label>
        <span v-else-if="activeAccount" class="chip">
          {{ activeAccount.name }} · {{ activeAccount.currency }} · {{ activeAccount.timezone }}
        </span>
        <button class="btn ads-boost ml-0" :disabled="!canWrite || !boostPosts.length"
          :title="canWrite ? '' : t('ads.manageNeeded')" @click="openBoost()">
          {{ t('ads.boost') }}
        </button>
      </div>

      <div class="grid grid-auto-cards gap-3 mb-3.5">
        <div class="panel flex flex-col gap-1 [&>strong]:text-xl">
          <span class="muted">{{ t('ads.spend') }}</span>
          <strong>{{ money(totals.spend * 100) }}</strong>
        </div>
        <div class="panel flex flex-col gap-1 [&>strong]:text-xl">
          <span class="muted">{{ t('ads.results') }}<template v-if="totals.label"> · {{ totals.label }}</template></span>
          <strong>{{ fmt(totals.results) }}</strong>
        </div>
        <div class="panel flex flex-col gap-1 [&>strong]:text-xl">
          <span class="muted">{{ t('ads.ctr') }}</span>
          <strong>{{ totals.ctr.toFixed(2) }}%</strong>
        </div>
        <div class="panel flex flex-col gap-1 [&>strong]:text-xl">
          <span class="muted">{{ t('ads.cpc') }}</span>
          <strong>{{ money(totals.cpc * 100) }}</strong>
        </div>
      </div>

      <!-- campaign table -->
      <TableFrame class="ads-tbl">
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
                  <button class="ads-link p-0 border-0 bg-transparent font-inherit font-semibold text-inherit cursor-pointer text-left hover:text-accent transition" @click.stop="toggle(c.id)">{{ c.name }}</button>
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
              <tr v-if="openId === c.id" :key="`${c.id}-detail`" class="[&>td]:bg-surface-2">
                <td colspan="9">
                  <div class="py-1.5 px-0.5 pb-2.5">
                    <div class="grid2">
                      <div>
                        <h4 class="my-3 mb-1.5 text-xs-plus uppercase tracking-wider text-muted">{{ t('ads.settings') }}</h4>
                        <dl class="grid grid-kv gap-x-3 gap-y-1 m-0 text-sm-minus [&>dt]:text-muted [&>dd]:m-0">
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
                        <h4 class="my-3 mb-1.5 text-xs-plus uppercase tracking-wider text-muted">{{ t('ads.actions') }}</h4>
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

                    <h4 class="my-3 mb-1.5 text-xs-plus uppercase tracking-wider text-muted">{{ t('ads.adsets') }}</h4>
                    <div v-for="s in adsetsOf(c.id)" :key="s.id" class="panel mb-2">
                      <div class="row">
                        <strong>{{ s.name }}</strong>
                        <span class="badge" :class="statusClass(s.status)">{{ s.status }}</span>
                        <span class="muted">{{ s.optimizationGoal }} · {{ s.billingEvent }}</span>
                      </div>
                      <p class="muted my-1.5">
                        {{ t('ads.targeting') }}: {{ s.targeting || '—' }}
                        <template v-if="s.promotedObject"> · {{ t('ads.promoted') }}: {{ s.promotedObject }}</template>
                      </p>
                      <ul class="mt-1 pl-4.5 flex flex-col gap-1 text-sm-minus">
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
      </TableFrame>
    </template>

    <!-- audit trail -->
    <section v-if="audit.length" class="mt-5 [&>ul]:mt-1.5 [&>ul]:pl-4.5">
      <h3>{{ t('ads.audit') }}</h3>
      <ul class="muted">
        <li v-for="(entry, i) in audit" :key="i">
          {{ entry.at }} · {{ entry.actor }} · {{ entry.action }} · {{ entry.target }}
          <template v-if="entry.detail"> · {{ entry.detail }}</template>
        </li>
      </ul>
    </section>

    <!-- budget dialog -->
    <div v-if="budgetFor" class="fixed inset-0 z-60 grid place-items-center bg-black/35" @click.self="budgetFor = null">
      <div class="panel ads-dialog dialog-sheet overflow-auto" role="dialog" aria-modal="true">
        <h3>{{ t('ads.editBudget') }}</h3>
        <p class="muted">{{ budgetFor.name }}</p>
        <label class="flex flex-col gap-1 mt-2.5 text-sm-minus">
          <span>{{ t('ads.dailyBudget') }} ({{ currency }})</span>
          <input v-model="budgetDaily" class="py-1.5 px-2.5 border border-line rounded-lg bg-surface font-inherit" type="number" min="0" step="0.01" />
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
    <div v-if="manageConfirm" class="fixed inset-0 z-60 grid place-items-center bg-black/35" @click.self="manageConfirm = false">
      <div class="panel ads-dialog dialog-sheet overflow-auto" role="dialog" aria-modal="true">
        <h3>{{ t('ads.manageEnable') }}</h3>
        <p class="muted">{{ t('ads.manageConfirm') }}</p>
        <div class="row mt">
          <button class="btn btn-primary" @click="confirmManage">{{ t('ads.confirm') }}</button>
          <button class="btn" @click="manageConfirm = false">{{ t('ads.cancel') }}</button>
        </div>
      </div>
    </div>

    <!-- boost dialog -->
    <div v-if="boostOpen" class="fixed inset-0 z-60 grid place-items-center bg-black/35" @click.self="boostOpen = false">
      <div class="panel ads-dialog dialog-sheet overflow-auto" role="dialog" aria-modal="true">
        <h3>{{ t('ads.boostTitle') }}<InfoTip :text="t('ads.boostHint')" /></h3>
        <label class="flex flex-col gap-1 mt-2.5 text-sm-minus">
          <span>{{ t('ads.boostName') }}</span>
          <input v-model="boostForm.name" class="py-1.5 px-2.5 border border-line rounded-lg bg-surface font-inherit" maxlength="120" />
        </label>
        <label class="flex flex-col gap-1 mt-2.5 text-sm-minus">
          <span>{{ t('ads.objective') }}</span>
          <select v-model="boostForm.objective" class="py-1.5 px-2.5 border border-line rounded-lg bg-surface font-inherit">
            <option v-for="o in OBJECTIVES" :key="o" :value="o">{{ o }}</option>
          </select>
        </label>
        <div class="grid2">
          <label class="flex flex-col gap-1 mt-2.5 text-sm-minus">
            <span>{{ t('ads.dailyBudget') }} ({{ currency }})</span>
            <input v-model="boostForm.dailyBudget" class="py-1.5 px-2.5 border border-line rounded-lg bg-surface font-inherit" type="number" min="0" step="0.01" />
          </label>
          <label class="flex flex-col gap-1 mt-2.5 text-sm-minus">
            <span>{{ t('ads.boostDays') }}</span>
            <input v-model.number="boostForm.days" class="py-1.5 px-2.5 border border-line rounded-lg bg-surface font-inherit" type="number" min="1" max="90" />
          </label>
        </div>
        <label class="flex flex-col gap-1 mt-2.5 text-sm-minus">
          <span>{{ t('ads.boostCountries') }}</span>
          <input v-model="boostForm.countries" class="py-1.5 px-2.5 border border-line rounded-lg bg-surface font-inherit" placeholder="TH" />
        </label>
        <label class="flex flex-col gap-1 mt-2.5 text-sm-minus">
          <span>{{ t('ads.boostPost') }}</span>
          <select v-model="boostForm.storyId" class="py-1.5 px-2.5 border border-line rounded-lg bg-surface font-inherit">
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

