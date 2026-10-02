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
import InfoTip from '@/components/ui/InfoTip.vue'
import { t } from '@/core/i18n'

const route = useRoute()
const router = useRouter()
const { data: campaigns, isPending } = useCampaigns()
const { data: setup } = useSetup()
const { data: contentRows } = useContentList({})
const create = useCreateCampaign()
const update = useUpdateCampaign()
const remove = useDeleteCampaign()
const { can } = usePermission()

const canWrite = computed(() => can('campaigns.write'))
const canDelete = computed(() => can('campaigns.delete'))

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

const STATUSES: CampaignStatus[] = ['draft', 'active', 'paused', 'completed']
const METRICS: CampaignMetric[] = ['views', 'likes', 'reach', 'posts']

function select(id: string | null): void {
  selectedId.value = id
  if (props.embedded) {
    emit('update:id', id ?? '')
    return
  }
  void router.replace(id ? `/campaigns/${id}` : '/campaigns')
}

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
  <h1>{{ t('campaign.title') }}<InfoTip :text="t('campaign.hint')" /></h1>

  <div class="row mt">
    <button class="btn btn-primary" :disabled="!canWrite || create.isPending.value" @click="newCampaign">
      {{ t('campaign.new') }}
    </button>
    <span v-if="saved" class="muted">{{ t('campaign.saved') }}</span>
    <span v-if="error" class="autherr" role="alert">{{ error }}</span>
  </div>

  <div class="camp-grid mt">
    <aside class="camp-list card">
      <div v-if="isPending" class="muted">{{ t('common.loading') }}</div>
      <button v-for="c in campaigns" v-else :key="c.id" type="button" class="camp-item"
        :class="{ active: c.id === selectedId }" @click="select(c.id)">
        <span class="camp-item-head">
          <strong>{{ c.name }}</strong>
          <span class="camp-status" :class="c.status">{{ statusLabel(c.status) }}</span>
        </span>
        <span class="muted">{{ c.startDate || '—' }} → {{ c.endDate || '—' }}</span>
        <span class="muted">
          {{ c.contentIds.length }} {{ t('campaign.linked') }} ·
          {{ c.budget ? `฿${c.budget.toLocaleString()}` : t('campaign.noBudget') }}
        </span>
      </button>
      <p v-if="campaigns && !campaigns.length" class="muted">{{ t('campaign.empty') }}</p>
    </aside>

    <section v-if="draft" class="card camp-editor">
      <div class="row">
        <h2 style="margin: 0;">{{ draft.name || t('campaign.new') }}</h2>
        <span style="flex: 1;" />
        <button class="btn btn-primary" :disabled="!canWrite || !dirty || update.isPending.value" @click="save">
          {{ t('common.save') }}
        </button>
        <button class="btn" :disabled="!canDelete || remove.isPending.value"
          :title="canDelete ? '' : t('auth.noPerm')" @click="destroy">{{ t('common.delete') }}</button>
      </div>

      <div class="grid2 mt">
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
            <button v-for="p in setup?.platforms ?? []" :key="p" type="button" class="chipbtn"
              :class="{ on: draft.platforms.includes(p) }" :disabled="!canWrite" @click="toggleIn('platforms', p)">
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
      <div v-if="schedule.length" class="tblwrap">
        <table class="tbl">
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
        </table>
      </div>
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
    </section>
  </div>
</template>
