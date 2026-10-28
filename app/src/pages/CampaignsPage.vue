<script setup lang="ts">
// Campaign manager: brief, schedule window, targets and the content schedule.
import { computed, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import {
  useCampaigns, useContentList, useCreateCampaign, useDeleteCampaign,
  usePermission, useSetup, useUpdateCampaign,
} from '@/core/queries'
import type { Campaign, CampaignMetric, CampaignStatus } from '@/mock/db'
import DatePickerPopup from '@/components/ui/DatePickerPopup.vue'
import TableFrame from '@/components/tables/TableFrame.vue'
import InfoTip from '@/components/ui/InfoTip.vue'
import MonthToggle from '@/components/ui/MonthToggle.vue'
import DockBar from '@/components/ui/DockBar.vue'
import PlatformLogo from '@/components/ui/PlatformLogo.vue'
import Drawer from '@/components/overlays/Drawer.vue'
import { activeMonth, activeYear } from '@/core/navdate'
import { deckFull } from '@/core/deck'
import { useWidePane } from '@/composables/useWidePane'
import { t } from '@/core/i18n'

const route = useRoute()
const router = useRouter()
const { data: campaigns, isPending, isError, error: loadError, refetch } = useCampaigns()
const { data: setup } = useSetup()
const { data: contentRows } = useContentList({})
const create = useCreateCampaign()
const update = useUpdateCampaign()
const remove = useDeleteCampaign()
const { can } = usePermission()

const canWrite = computed(() => can('campaigns.write'))
const canDelete = computed(() => can('campaigns.delete'))

// Month filter: the deck works on one (year, month), so campaigns follow the
// shared values like the planner does. Dateless campaigns always show; dated
// ones show when the selected year-month falls inside their window.
const month = ref<number>(activeMonth.value)
watch(month, (value) => { activeMonth.value = value })
watch(activeMonth, (value) => { if (value !== month.value) month.value = value })
const year = ref<number>(activeYear.value)
watch(year, (value) => { activeYear.value = value })
watch(activeYear, (value) => { if (value !== year.value) year.value = value })

function setMonth(value: number): void { month.value = value }
function setYear(value: number): void { year.value = value }

function monthOf(d: string | null): number | null {
  const m = /^\d{4}-(\d{2})-\d{2}/.exec(d ?? '')
  const n = m ? Number(m[1]) : NaN
  return n >= 1 && n <= 12 ? n : null
}

function yearOf(d: string | null): number | null {
  const m = /^(\d{4})-\d{2}-\d{2}/.exec(d ?? '')
  const n = m ? Number(m[1]) : NaN
  return n >= 1970 && n <= 9999 ? n : null
}

function covers(m: number, s: number, e: number): boolean {
  let cur = s
  for (let i = 0; i < 12; i++) {
    if (cur === m) return true
    if (cur === e) return false
    cur = (cur % 12) + 1
  }
  return false
}

const visible = computed(() => (campaigns.value ?? []).filter((c) => {
  const s = monthOf(c.startDate)
  const e = monthOf(c.endDate)
  if (s === null && e === null) return true
  const a = s ?? e
  const b = e ?? s
  if (a === null || b === null) return true
  // Year-bounded windows compare absolute month indexes; a window with no
  // usable years (or running backwards) keeps the legacy month-only check.
  const sy = yearOf(c.startDate)
  const ey = yearOf(c.endDate)
  if (sy !== null && ey !== null) {
    const start = sy * 12 + a
    const end = ey * 12 + b
    if (start <= end) return sy * 12 + a <= year.value * 12 + month.value && year.value * 12 + month.value <= end
  }
  return covers(month.value, a, b)
}))

const root = ref<HTMLElement | null>(null)
const { wide } = useWidePane(root, 'camp-scroll')

const props = defineProps<{ id?: string; embedded?: boolean }>()
const emit = defineEmits<{ 'update:id': [string] }>()

const selectedId = ref<string | null>(
  props.id ?? (typeof route.params.id === 'string' ? route.params.id : null),
)
watch(
  () => props.id,
  (value) => {
    if (props.embedded) selectedId.value = value ?? null
  },
)
const draft = ref<Campaign | null>(null)
const hashtagsText = ref('')
const error = ref('')
const saved = ref(false)
// The campaign editor is a bottom sheet inside the card.
const editorOpen = ref(false)

const STATUSES: CampaignStatus[] = ['draft', 'active', 'paused', 'completed']
const METRICS: CampaignMetric[] = ['views', 'likes', 'reach', 'posts']

function select(id: string | null): void {
  selectedId.value = id
  editorOpen.value = Boolean(id)
  if (props.embedded) {
    emit('update:id', id ?? '')
    return
  }
  void router.replace(id ? `/campaigns/${id}` : '/campaigns')
}

// Auto-select first campaign on desktop when loaded
watch(
  () => visible.value,
  (list) => {
    if ((deckFull.value || wide.value) && !selectedId.value && list && list.length > 0) {
      select(list[0].id)
    }
  },
  { immediate: true },
)

function fromCampaign(c: Campaign): Campaign {
  return { ...c, platforms: [...c.platforms], pillars: [...c.pillars], hashtags: [...c.hashtags], contentIds: [...c.contentIds] }
}

watch(
  [campaigns, selectedId],
  ([rows, id]) => {
    const found = rows?.find((c) => c.id === id) ?? null
    if (!found) {
      draft.value = null
      return
    }
    draft.value = fromCampaign(found)
    hashtagsText.value = found.hashtags.join(', ')
    saved.value = false
    error.value = ''
  },
  { immediate: true },
)

// Deep links (?id=) open the sheet; closing it drops the selection so the
// query is cleared and a refresh does not re-open it.
watch(selectedId, (id) => { if (id) editorOpen.value = true })
watch(editorOpen, (open) => { if (!open && selectedId.value) select(null) })

const dirty = computed(() => {
  const current = campaigns.value?.find((c) => c.id === selectedId.value)
  if (!current || !draft.value) return false
  return JSON.stringify(fromCampaign(current)) !== JSON.stringify(draft.value)
})

const linked = computed(() => {
  const ids = new Set(draft.value?.contentIds ?? [])
  return (contentRows.value ?? []).filter((c) => ids.has(c.id))
})

const schedule = computed(() =>
  [...linked.value].sort((a, b) => (a.scheduledFor ?? a.publishedAt ?? '9999').localeCompare(b.scheduledFor ?? b.publishedAt ?? '9999')),
)

function contentDate(c: { scheduledFor: string | null; publishedAt: string | null }): string {
  return c.scheduledFor ?? c.publishedAt ?? '—'
}

function statusLabel(status: CampaignStatus): string {
  return t(`campaign.status.${status}`)
}

function toggleIn(list: 'platforms' | 'pillars', value: string): void {
  const c = draft.value
  if (!c || !canWrite.value) return
  const has = c[list].includes(value)
  c[list] = has ? c[list].filter((v) => v !== value) : [...c[list], value]
}

function toggleContent(id: string): void {
  const c = draft.value
  if (!c || !canWrite.value) return
  c.contentIds = c.contentIds.includes(id)
    ? c.contentIds.filter((v) => v !== id)
    : [...c.contentIds, id]
}

async function newCampaign(): Promise<void> {
  if (!canWrite.value) return
  error.value = ''
  try {
    const created = await create.mutateAsync({
      name: 'New campaign',
      owner: setup.value?.owner ?? '',
      startDate: null,
      endDate: null,
    })
    select(created.id)
  } catch (e) {
    error.value = e instanceof Error ? e.message : String(e)
  }
}

async function save(): Promise<void> {
  const c = draft.value
  if (!c || !canWrite.value || !dirty.value) return
  error.value = ''
  try {
    c.hashtags = hashtagsText.value.split(',').map((v) => v.trim()).filter(Boolean)
    const updated = await update.mutateAsync({ id: c.id, patch: c })
    draft.value = fromCampaign(updated)
    hashtagsText.value = updated.hashtags.join(', ')
    saved.value = true
  } catch (e) {
    error.value = e instanceof Error ? e.message : String(e)
  }
}

async function destroy(): Promise<void> {
  const c = draft.value
  if (!c || !canDelete.value) return
  if (typeof window !== 'undefined' && !window.confirm(`${t('common.delete')}: ${c.name}?`)) return
  error.value = ''
  try {
    await remove.mutateAsync(c.id)
    select(null)
  } catch (e) {
    error.value = e instanceof Error ? e.message : String(e)
  }
}

const totalBudget = computed(() => (visible.value ?? []).reduce((acc, c) => acc + (c.budget || 0), 0))
const activeCount = computed(() => (visible.value ?? []).filter((c) => c.status === 'active').length)
const totalLinked = computed(() => (visible.value ?? []).reduce((acc, c) => acc + (c.contentIds?.length || 0), 0))
</script>

<template>
  <div class="camp-page" ref="root">
    <div class="camp-grid">
      <DockBar label="Campaign controls">
        <div class="flex items-center justify-between w-full gap-3">
          <div class="flex items-center gap-2">
            <MonthToggle :model-value="month" :year="year" @update:model-value="setMonth" @update:year="setYear" />
            <span class="dock-sep" aria-hidden="true" />
            <span class="text-xs font-semibold text-muted tracking-wider uppercase">{{ (visible ?? []).length }} Campaigns</span>
          </div>
          <div class="flex items-center gap-2 ml-auto">
            <button class="btn btn-primary btn-icon w-auto" :disabled="!canWrite || create.isPending.value"
              :title="t('campaign.new')" aria-label="New campaign" @click="newCampaign">{{ create.isPending.value ? '…' : '+' }}</button>
          </div>
        </div>
      </DockBar>

      <div class="camp-main camp-list">
        <!-- List Header -->
        <div class="flex items-center justify-between pb-3 mb-2 border-b border-line/60">
          <div class="flex items-center gap-2">
            <span class="text-xs font-bold uppercase tracking-wider text-muted">Campaigns</span>
            <span class="px-2 py-0.5 rounded-full text-[11px] font-mono font-bold bg-wash text-ink">{{ (visible ?? []).length }}</span>
          </div>
          <button type="button" class="text-xs font-semibold text-accent hover:underline cursor-pointer bg-transparent border-0 p-0"
            :disabled="!canWrite" @click="newCampaign">
            + New
          </button>
        </div>

        <span v-if="saved" class="muted">{{ t('campaign.saved') }}</span>
        <span v-if="error" class="autherr" role="alert">{{ error }}</span>
        <div v-if="isPending" class="muted py-4 text-center">{{ t('common.loading') }}</div>
        <div v-else-if="isError" class="card" role="alert">
          <p class="muted">{{ loadError?.message }}</p>
          <button type="button" class="btn" @click="() => refetch()">{{ t('common.tryAgain') }}</button>
        </div>
        <div v-else-if="!visible || visible.length === 0" class="muted py-8 text-center text-xs">
          {{ t('campaign.empty') }}
        </div>
        <template v-else>
          <button v-for="c in visible" :key="c.id" type="button" class="camp-item"
            :class="{ active: c.id === selectedId }" @click="select(c.id)">
            <div class="camp-item-head">
              <strong class="text-sm font-bold text-ink truncate">{{ c.name }}</strong>
              <span class="camp-status flex-none" :class="c.status">{{ statusLabel(c.status) }}</span>
            </div>
            <div class="flex items-center gap-1.5 text-xs text-muted">
              <span>{{ c.startDate || '—' }} → {{ c.endDate || '—' }}</span>
            </div>
            <div class="flex items-center justify-between text-xs text-muted mt-1 w-full pt-1.5 border-t border-line/30">
              <span class="font-mono text-[11px]">{{ c.contentIds.length }} {{ t('campaign.linked') }}</span>
              <span class="font-mono font-semibold text-ink text-[11px]">{{ c.budget ? `฿${c.budget.toLocaleString()}` : t('campaign.noBudget') }}</span>
            </div>
            <div v-if="c.platforms && c.platforms.length" class="flex items-center gap-1 mt-0.5">
              <PlatformLogo v-for="p in c.platforms" :key="p" :name="p" :size="13" />
            </div>
          </button>
        </template>
      </div>

      <!-- Right Pane Overview when not in editing mode -->
      <div v-if="(deckFull || wide) && !editorOpen" class="camp-detail camp-overview flex-1 flex flex-col gap-5 py-2">
        <div class="flex items-center justify-between pb-3.5 border-b border-line/60">
          <div>
            <h2 class="text-base font-bold text-ink tracking-tight m-0">{{ t('campaign.title') }} Overview</h2>
            <p class="text-xs text-muted mt-0.5 m-0">Schedules, platform targets, and budgets across campaigns</p>
          </div>
          <div class="flex items-center gap-2.5">
            <span class="text-xs font-semibold px-2.5 py-1 rounded-full bg-wash text-ink border border-line/50 font-mono">
              {{ (visible ?? []).length }} Total
            </span>
            <button type="button" class="btn btn-primary text-xs font-bold uppercase tracking-wider inline-flex items-center gap-1.5"
              :disabled="!canWrite" @click="newCampaign">
              <span class="text-sm leading-none">+</span>
              <span>New campaign</span>
            </button>
          </div>
        </div>

        <!-- KPI Cards -->
        <div class="grid grid-cols-1 sm:grid-cols-3 gap-3">
          <div class="p-3.5 bg-surface rounded-xl border border-line/60 shadow-2xs">
            <div class="flex items-center justify-between mb-1.5">
              <span class="text-2xs font-bold uppercase tracking-wider text-muted">Active Campaigns</span>
              <span class="w-2 h-2 rounded-full bg-ink"></span>
            </div>
            <div class="text-2xl font-black text-ink font-mono">{{ activeCount }}</div>
            <span class="text-[11px] text-muted">{{ (visible ?? []).length - activeCount }} draft or paused</span>
          </div>

          <div class="p-3.5 bg-surface rounded-xl border border-line/60 shadow-2xs">
            <div class="flex items-center justify-between mb-1.5">
              <span class="text-2xs font-bold uppercase tracking-wider text-muted">Total Budget</span>
              <span class="text-xs text-muted font-mono">THB</span>
            </div>
            <div class="text-2xl font-black text-ink font-mono">฿{{ totalBudget.toLocaleString('en-US') }}</div>
            <span class="text-[11px] text-muted">Allocated for this window</span>
          </div>

          <div class="p-3.5 bg-surface rounded-xl border border-line/60 shadow-2xs">
            <div class="flex items-center justify-between mb-1.5">
              <span class="text-2xs font-bold uppercase tracking-wider text-muted">Linked Content</span>
              <svg class="w-3.5 h-3.5 text-muted" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M10 13a5 5 0 0 0 7.54.54l3-3a5 5 0 0 0-7.07-7.07l-1.72 1.71"></path><path d="M14 11a5 5 0 0 0-7.54-.54l-3 3a5 5 0 0 0 7.07 7.07l1.71-1.71"></path></svg>
            </div>
            <div class="text-2xl font-black text-ink font-mono">{{ totalLinked }}</div>
            <span class="text-[11px] text-muted">Posts attached to campaigns</span>
          </div>
        </div>

        <!-- Campaign Roster Table -->
        <div class="flex-1 flex flex-col gap-2 mt-1 min-h-0">
          <div class="flex items-center justify-between">
            <h3 class="text-xs font-bold uppercase tracking-wider text-muted m-0">Campaign Roster</h3>
            <span class="text-xs text-muted">{{ t('campaign.pickHint') }}</span>
          </div>

          <div class="rounded-xl border border-line/60 bg-surface shadow-2xs overflow-hidden">
            <table class="w-full text-left border-collapse text-xs">
              <thead>
                <tr class="border-b border-line/60 bg-wash/50 text-muted font-bold text-2xs uppercase tracking-wider">
                  <th class="py-2.5 px-3.5">Campaign</th>
                  <th class="py-2.5 px-3">Status</th>
                  <th class="py-2.5 px-3">Window</th>
                  <th class="py-2.5 px-3">Platforms</th>
                  <th class="py-2.5 px-3 text-right">Budget</th>
                  <th class="py-2.5 px-3 text-center">Linked</th>
                  <th class="py-2.5 px-3 text-right">Action</th>
                </tr>
              </thead>
              <tbody class="divide-y divide-line/40">
                <tr v-for="c in visible" :key="c.id"
                  class="hover:bg-wash/40 cursor-pointer transition-colors group"
                  @click="select(c.id)">
                  <td class="py-3 px-3.5 font-medium text-ink">
                    <div class="font-bold text-sm leading-snug">{{ c.name }}</div>
                    <div v-if="c.objective" class="text-muted text-[11px] truncate max-w-[220px]">{{ c.objective }}</div>
                  </td>
                  <td class="py-3 px-3">
                    <span class="camp-status" :class="c.status">{{ statusLabel(c.status) }}</span>
                  </td>
                  <td class="py-3 px-3 text-muted font-mono text-[11px] whitespace-nowrap">
                    {{ c.startDate || '—' }} → {{ c.endDate || '—' }}
                  </td>
                  <td class="py-3 px-3">
                    <div v-if="c.platforms && c.platforms.length" class="flex items-center gap-1.5">
                      <PlatformLogo v-for="p in c.platforms" :key="p" :name="p" :size="14" />
                    </div>
                    <span v-else class="text-muted text-[11px]">—</span>
                  </td>
                  <td class="py-3 px-3 text-right font-mono font-semibold text-ink whitespace-nowrap">
                    {{ c.budget ? `฿${c.budget.toLocaleString()}` : '—' }}
                  </td>
                  <td class="py-3 px-3 text-center font-mono">
                    <span class="px-2 py-0.5 rounded-full bg-wash text-[11px] font-semibold">{{ c.contentIds.length }}</span>
                  </td>
                  <td class="py-3 px-3 text-right whitespace-nowrap">
                    <button type="button" class="btn text-[11px] py-1 px-2.5 opacity-80 group-hover:opacity-100 hover:border-ink transition-all cursor-pointer">
                      Edit ↗
                    </button>
                  </td>
                </tr>
                <tr v-if="!visible || visible.length === 0">
                  <td colspan="7" class="py-8 text-center text-muted">
                    {{ t('campaign.empty') }}
                  </td>
                </tr>
              </tbody>
            </table>
          </div>
        </div>
      </div>

      <Drawer v-model="editorOpen" :inline="deckFull || wide"
        :side="deckFull ? 'right' : 'bottom'" :width="'100%'" contained
        :title="draft?.name || t('campaign.new')">
    <template v-if="draft">
      <div class="grid2">
        <div>
          <label class="lbl" for="camp-name">{{ t('campaign.name') }}</label>
          <input id="camp-name" class="field" v-model="draft.name" :disabled="!canWrite" maxlength="200" />
        </div>
        <div>
          <label class="lbl" for="camp-status">{{ t('campaign.status') }}</label>
          <select id="camp-status" class="field" v-model="draft.status" :disabled="!canWrite">
            <option v-for="s in STATUSES" :key="s" :value="s">{{ statusLabel(s) }}</option>
          </select>
        </div>
      </div>

      <label class="lbl" for="camp-objective">{{ t('campaign.objective') }}</label>
      <input id="camp-objective" class="field" v-model="draft.objective" :disabled="!canWrite" maxlength="200" />

      <div class="grid2">
        <div>
          <label class="lbl" for="camp-start">{{ t('campaign.start') }}</label>
          <DatePickerPopup id="camp-start" v-model="draft.startDate" :disabled="!canWrite" :title="t('campaign.start')" />
        </div>
        <div>
          <label class="lbl" for="camp-end">{{ t('campaign.end') }}</label>
          <DatePickerPopup id="camp-end" v-model="draft.endDate" :disabled="!canWrite" :title="t('campaign.end')" />
        </div>
      </div>

      <div class="grid2">
        <div>
          <label class="lbl">{{ t('campaign.platforms') }}</label>
          <div class="chiprow">
            <button v-for="p in setup?.platforms ?? []" :key="p" type="button" class="chipbtn inline-flex items-center gap-1.5"
              :class="{ on: draft.platforms.includes(p) }" :disabled="!canWrite" @click="toggleIn('platforms', p)">
              <PlatformLogo :name="p" :size="13" />
              {{ p }}
            </button>
          </div>
        </div>
        <div>
          <label class="lbl">{{ t('campaign.pillars') }}</label>
          <div class="chiprow">
            <button v-for="p in setup?.pillars ?? []" :key="p" type="button" class="chipbtn"
              :class="{ on: draft.pillars.includes(p) }" :disabled="!canWrite" @click="toggleIn('pillars', p)">
              {{ p }}
            </button>
          </div>
        </div>
      </div>

      <div class="grid3">
        <div>
          <label class="lbl" for="camp-budget">{{ t('campaign.budget') }}</label>
          <input id="camp-budget" class="field" type="number" min="0" step="100" v-model.number="draft.budget"
            :disabled="!canWrite" />
        </div>
        <div>
          <label class="lbl" for="camp-metric">{{ t('campaign.metric') }}</label>
          <select id="camp-metric" class="field" v-model="draft.goalMetric" :disabled="!canWrite">
            <option v-for="m in METRICS" :key="m" :value="m">{{ t(`campaign.metric.${m}`) }}</option>
          </select>
        </div>
        <div>
          <label class="lbl" for="camp-target">{{ t('campaign.target') }}</label>
          <input id="camp-target" class="field" type="number" min="0" step="1000" v-model.number="draft.goalTarget"
            :disabled="!canWrite" />
        </div>
      </div>

      <label class="lbl" for="camp-hashtags">{{ t('campaign.hashtags') }}</label>
      <input id="camp-hashtags" class="field" v-model="hashtagsText" :disabled="!canWrite"
        :placeholder="t('campaign.hashtagHint')" />

      <label class="lbl" for="camp-owner">{{ t('campaign.owner') }}</label>
      <input id="camp-owner" class="field" v-model="draft.owner" :disabled="!canWrite" maxlength="200" />

      <label class="lbl" for="camp-notes">{{ t('campaign.notes') }}</label>
      <textarea id="camp-notes" class="field" rows="3" v-model="draft.notes" :disabled="!canWrite"></textarea>

      <h3>{{ t('campaign.schedule') }}<InfoTip :text="t('campaign.scheduleHint')" /></h3>
      <TableFrame v-if="schedule.length">
        <thead>
          <tr><th>{{ t('campaign.content') }}</th><th>{{ t('campaign.date') }}</th><th>{{ t('campaign.status') }}</th></tr>
        </thead>
        <tbody>
          <tr v-for="c in schedule" :key="c.id" style="cursor: default;">
            <td>{{ c.title }}</td>
            <td>{{ contentDate(c) }}</td>
            <td>{{ c.status }}</td>
          </tr>
        </tbody>
      </TableFrame>
      <p v-else class="muted">{{ t('campaign.noContent') }}</p>

      <h3>{{ t('campaign.linkContent') }}</h3>
      <div class="linkgrid">
        <label v-for="c in contentRows ?? []" :key="c.id" class="linkitem">
          <input type="checkbox" :checked="draft.contentIds.includes(c.id)" :disabled="!canWrite"
            @change="toggleContent(c.id)" />
          <span>{{ c.title }}</span>
          <span class="muted">{{ contentDate(c) }}</span>
        </label>
        <p v-if="!(contentRows ?? []).length" class="muted">{{ t('campaign.noContent') }}</p>
      </div>
    </template>
    <template #footer>
      <button class="btn btn-primary" :disabled="!canWrite || !dirty || update.isPending.value" @click="save">
        {{ t('common.save') }}
      </button>
      <button class="btn" :disabled="!canDelete || remove.isPending.value"
        :title="canDelete ? '' : t('auth.noPerm')" @click="destroy">{{ t('common.delete') }}</button>
    </template>
      </Drawer>
    </div>
  </div>
</template>
