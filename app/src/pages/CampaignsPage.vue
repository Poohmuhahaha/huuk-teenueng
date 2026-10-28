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

// Auto-select first campaign on desktop when loaded or when month filter changes
watch(
  [() => visible.value, () => wide.value, () => deckFull.value],
  ([list, isWide, isFull]) => {
    if ((isWide || isFull) && list && list.length > 0) {
      if (!selectedId.value || !list.some((c) => c.id === selectedId.value)) {
        selectedId.value = list[0].id
        editorOpen.value = true
        if (props.embedded) {
          emit('update:id', list[0].id)
        }
      }
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
        <div class="flex items-center justify-between pb-3 mb-2.5 px-1 border-b border-black/[0.06]">
          <div class="flex items-center gap-2">
            <span class="text-xs font-bold uppercase tracking-wider text-muted">Campaigns</span>
            <span class="px-2 py-0.5 rounded-full text-[11px] font-mono font-bold bg-wash text-ink">{{ (visible ?? []).length }}</span>
          </div>
          <button type="button" class="w-auto inline-flex items-center gap-1 px-2.5 py-1 rounded-full text-xs font-semibold text-ink bg-wash hover:bg-ink hover:text-white transition-colors cursor-pointer border-0"
            :disabled="!canWrite" @click="newCampaign">
            <span>+ New</span>
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
          <button v-for="c in visible" :key="c.id" type="button" class="camp-item card-fade group"
            :class="{ active: c.id === selectedId }" @click="select(c.id)">
            <div class="camp-item-head">
              <div class="min-w-0 flex-1 pr-2">
                <strong class="block text-[14px] font-bold text-ink tracking-tight truncate">{{ c.name }}</strong>
                <p v-if="c.objective" class="text-[11px] text-muted truncate m-0 mt-0.5">{{ c.objective }}</p>
              </div>
              <span class="camp-status flex-none" :class="c.status">{{ statusLabel(c.status) }}</span>
            </div>

            <div class="flex items-center gap-1.5 text-[11.5px] text-ink-soft font-mono mt-0.5">
              <svg class="w-3.5 h-3.5 text-muted flex-none" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
                <rect x="3" y="4" width="18" height="18" rx="2" ry="2"></rect>
                <line x1="16" y1="2" x2="16" y2="6"></line>
                <line x1="8" y1="2" x2="8" y2="6"></line>
                <line x1="3" y1="10" x2="21" y2="10"></line>
              </svg>
              <span>{{ c.startDate || '—' }} → {{ c.endDate || '—' }}</span>
            </div>

            <div class="flex items-center justify-between gap-2 text-xs text-muted mt-1.5 w-full pt-2.5 border-t border-black/[0.06]">
              <div class="flex items-center gap-2 min-w-0">
                <div v-if="c.platforms && c.platforms.length" class="flex items-center gap-1.5 pr-2 border-r border-black/[0.08]">
                  <PlatformLogo v-for="p in c.platforms" :key="p" :name="p" :size="13" />
                </div>
                <span class="inline-flex items-center gap-1 font-mono text-[11px] text-muted">
                  {{ c.contentIds.length }} {{ t('campaign.linked') }}
                </span>
              </div>
              <span class="font-mono font-bold text-ink text-[13px] tracking-tight tabular-nums">
                {{ c.budget ? `฿${c.budget.toLocaleString()}` : t('campaign.noBudget') }}
              </span>
            </div>
          </button>
        </template>
      </div>

      <Drawer :model-value="editorOpen || ((deckFull || wide) && Boolean(draft))"
        @update:model-value="editorOpen = $event"
        :inline="deckFull || wide"
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
            <button v-for="p in setup?.platforms ?? []" :key="p" type="button"
              class="chipbtn flex flex-col items-center justify-center w-[88px] h-[88px] aspect-square gap-2 rounded-[20px] lg:rounded-[24px] transition bg-white border border-[#ded9d1] hover:border-[#111] text-black/80"
              :class="{ on: draft.platforms.includes(p) }" :disabled="!canWrite" @click="toggleIn('platforms', p)">
              <PlatformLogo :name="p" :size="24" />
              <span class="text-xs font-semibold tracking-tight leading-none">{{ p }}</span>
            </button>
          </div>
        </div>
        <div>
          <label class="lbl">{{ t('campaign.pillars') }}</label>
          <div class="chiprow">
            <button v-for="p in setup?.pillars ?? []" :key="p" type="button"
              class="chipbtn flex flex-col items-center justify-center w-[88px] h-[88px] aspect-square rounded-[20px] lg:rounded-[24px] transition bg-white border border-[#ded9d1] hover:border-[#111] text-black/80"
              :class="{ on: draft.pillars.includes(p) }" :disabled="!canWrite" @click="toggleIn('pillars', p)">
              <span class="text-xs font-semibold tracking-tight leading-tight">{{ p }}</span>
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
