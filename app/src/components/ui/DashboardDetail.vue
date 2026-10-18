<script setup lang="ts">
// Dashboard in-place detail — the same system as the Content Studio: inline
// in the wide/full card, inside a bottom sheet on the compact card. Three
// kinds: a new-topic form, the metrics import, and KPI drill-downs. Nothing
// here navigates away; cross-links are explicit at the bottom.
import { computed, ref, watch } from 'vue'
import {
  useSetup, useAddPost, useImportMetrics, usePosts, useMetrics, usePermission,
} from '@/core/queries'
import { activeMonth, activeYear } from '@/core/navdate'
import { monthName, t } from '@/core/i18n'
import { activePlatforms, platformIdBySetupName } from '@/core/platforms'
import type { Status } from '@/mock/db'

const props = defineProps<{ kind: '' | 'new' | 'import' | 'kpi'; kpi?: string }>()
const emit = defineEmits<{ close: []; done: [msg: string] }>()

const month = activeMonth
const year = activeYear
const { data: setup } = useSetup()
const { data: posts } = usePosts(month, year)
const { data: metrics } = useMetrics()
const { can } = usePermission()
const canWrite = computed(() => can('posts.write'))
const canImport = computed(() => can('metrics.import'))

// ---- new topic ----
const addPost = useAddPost(month)
const nTopic = ref('')
const nPillar = ref('')
const nDate = ref('')
const nTime = ref('18:00')
const nStatus = ref('')
const nPlatforms = ref<string[]>([])
const nError = ref('')
const nNotice = ref('')

function pad(n: number): string {
  return String(n).padStart(2, '0')
}

function defaultPlatforms(): string[] {
  const names = setup.value?.platforms ?? []
  const active = names.filter((name) => {
    const id = platformIdBySetupName(name)
    return id === 'instagram' || (id !== undefined && activePlatforms.value.includes(id))
  })
  return active.length ? active : [...names]
}

watch(
  [() => props.kind, () => props.kpi, setup, month, year],
  () => {
    const s = setup.value
    if (!s) return
    if (!nPillar.value) nPillar.value = s.pillars[0] ?? ''
    if (!nStatus.value) nStatus.value = s.statuses[0] ?? 'Start'
    if (!nPlatforms.value.length) nPlatforms.value = defaultPlatforms()
    if (!nDate.value) nDate.value = `${year.value}-${pad(month.value)}-01`
  },
  { immediate: true },
)

function togglePlatform(name: string): void {
  const set = new Set(nPlatforms.value)
  if (set.has(name)) set.delete(name)
  else set.add(name)
  nPlatforms.value = [...set]
}

async function addTopic(): Promise<void> {
  const topic = nTopic.value.trim()
  if (!topic || !canWrite.value || addPost.isPending.value) return
  nError.value = ''
  nNotice.value = ''
  try {
    await addPost.mutateAsync({
      month: month.value,
      year: year.value,
      topic,
      pillar: nPillar.value,
      format: setup.value?.formats[0] ?? '',
      goal: setup.value?.goals[0] ?? '',
      date: nDate.value || `${year.value}-${pad(month.value)}-01`,
      time: nTime.value || '18:00',
      status: (nStatus.value || 'Start') as Status,
      hook: '',
      caption: '',
      cta: '',
      hashtagGroup: '',
      hashtags: [],
      imageUrl: '',
      note: '',
      done: false,
      platforms: [...nPlatforms.value],
    })
    nNotice.value = t('dash.added', { topic, month: monthName(month.value) })
    nTopic.value = ''
    emit('done', nNotice.value)
  } catch (e) {
    nError.value = e instanceof Error ? e.message : String(e)
  }
}

// ---- import metrics ----
const imp = useImportMetrics()
const iPlatform = ref('')
watch(
  setup,
  (s) => {
    if (s && !iPlatform.value) iPlatform.value = s.platforms[0] ?? ''
  },
  { immediate: true },
)

// ---- KPI drill-down ----
const KPI_LABELS: Record<string, string> = {
  total: 'Total', posted: 'Posted', pending: 'Pending', wip: 'WIP', views: 'Views', likes: 'Likes',
}
const kpiTitle = computed(() => KPI_LABELS[props.kpi ?? ''] ?? 'Details')

const kpiRows = computed(() => {
  const rows = posts.value ?? []
  const key = props.kpi ?? ''
  if (key === 'total') return rows.map((p) => ({ id: p.id, topic: p.topic, date: p.date, status: p.status, views: 0, likes: 0 }))
  if (key === 'posted') return rows.filter((p) => p.status === 'Done').map((p) => ({ id: p.id, topic: p.topic, date: p.date, status: p.status, views: 0, likes: 0 }))
  if (key === 'pending') return rows.filter((p) => p.status === 'Start').map((p) => ({ id: p.id, topic: p.topic, date: p.date, status: p.status, views: 0, likes: 0 }))
  if (key === 'wip') return rows.filter((p) => p.status === 'Design' || p.status === 'Dev').map((p) => ({ id: p.id, topic: p.topic, date: p.date, status: p.status, views: 0, likes: 0 }))
  if (key !== 'views' && key !== 'likes') return []
  const mets = metrics.value ?? []
  return rows
    .map((p) => {
      const mine = mets.filter((m) => m.postId === p.id)
      return {
        id: p.id,
        topic: p.topic,
        date: p.date,
        status: p.status,
        views: mine.reduce((s, m) => s + m.views, 0),
        likes: mine.reduce((s, m) => s + m.likes, 0),
      }
    })
    .sort((a, b) => (key === 'likes' ? b.likes - a.likes : b.views - a.views))
})

const showMetrics = computed(() => props.kpi === 'views' || props.kpi === 'likes')
const crossLink = computed(() =>
  showMetrics.value
    ? { to: '/analyze?tab=performance', label: t('dash.openPerf') }
    : { to: '/plan?tab=monthly', label: t('dash.openPlanner') },
)
</script>

<template>
  <!-- New topic -->
  <div v-if="kind === 'new'" class="grid gap-3 text-left">
    <div class="grid gap-1">
      <label class="block text-muted text-sm font-[650] m-0" for="dd-topic">{{ t('dash.topic') }}</label>
      <input id="dd-topic" v-model="nTopic" :disabled="!canWrite"
        :title="canWrite ? '' : t('auth.noPerm')" @keyup.enter="addTopic" />
    </div>
    <div class="grid grid-cols-2 gap-3">
      <div class="grid gap-1">
        <label class="block text-muted text-sm font-[650] m-0" for="dd-pillar">{{ t('setup.list.pillars') }}</label>
        <select id="dd-pillar" v-model="nPillar" :disabled="!canWrite">
          <option v-for="o in setup?.pillars ?? []" :key="o" :value="o">{{ o }}</option>
        </select>
      </div>
      <div class="grid gap-1">
        <label class="block text-muted text-sm font-[650] m-0" for="dd-status">Status</label>
        <select id="dd-status" v-model="nStatus" :disabled="!canWrite">
          <option v-for="o in setup?.statuses ?? []" :key="o" :value="o">{{ o }}</option>
        </select>
      </div>
    </div>
    <div class="grid grid-cols-2 gap-3">
      <div class="grid gap-1">
        <label class="block text-muted text-sm font-[650] m-0" for="dd-date">Date</label>
        <input id="dd-date" type="date" v-model="nDate" :disabled="!canWrite" />
      </div>
      <div class="grid gap-1">
        <label class="block text-muted text-sm font-[650] m-0" for="dd-time">Time</label>
        <input id="dd-time" type="time" v-model="nTime" :disabled="!canWrite" />
      </div>
    </div>
    <div class="grid gap-1">
      <span class="block text-muted text-sm font-[650] m-0">{{ t('setup.list.platforms') }}</span>
      <div class="flex flex-wrap gap-1.5">
        <label v-for="o in setup?.platforms ?? []" :key="o" class="inline-flex items-center gap-1.5 rounded-full border px-3 py-[5px] text-sm cursor-pointer"
          :class="nPlatforms.includes(o) ? 'on bg-ink text-white border-ink' : 'bg-transparent text-ink border-line'">
          <input type="checkbox" :checked="nPlatforms.includes(o)" :disabled="!canWrite"
            @change="togglePlatform(o)" />
          {{ o }}
        </label>
      </div>
    </div>
    <p v-if="nError" class="text-danger text-[13.5px] font-semibold mt-2.5" role="alert">{{ nError }}</p>
    <p v-if="nNotice" class="text-muted text-sm" role="status">{{ nNotice }}</p>
    <div class="flex gap-2">
      <button class="w-auto flex-none font-sans uppercase tracking-[0.08em] rounded-full px-5.5 py-3 text-[12.5px] font-semibold border-[1.5px] border-ink bg-ink text-white cursor-pointer shadow-none hover:bg-accent-hover hover:border-accent-hover hover:text-white active:transform-none disabled:opacity-50 disabled:cursor-default" :disabled="!canWrite || addPost.isPending.value || !nTopic.trim()"
        :title="canWrite ? '' : t('auth.noPerm')" @click="addTopic">
        {{ t('dash.newTopic') }}
      </button>
    </div>
  </div>

  <!-- Import metrics (same import as Analyze ▸ Performance) -->
  <div v-else-if="kind === 'import'" class="grid gap-3 text-left">
    <div class="grid gap-1">
      <label class="block text-muted text-sm font-[650] m-0" for="dd-platform">Platform</label>
      <select id="dd-platform" v-model="iPlatform" :disabled="!canImport">
        <option v-for="o in setup?.platforms ?? []" :key="o" :value="o">{{ o }}</option>
      </select>
    </div>
    <div class="flex gap-2">
      <button class="w-auto flex-none font-sans uppercase tracking-[0.08em] rounded-full px-5.5 py-3 text-[12.5px] font-semibold border-[1.5px] border-ink bg-ink text-white cursor-pointer shadow-none hover:bg-accent-hover hover:border-accent-hover hover:text-white active:transform-none disabled:opacity-50 disabled:cursor-default" :disabled="!iPlatform || imp.isPending.value || !canImport"
        :title="canImport ? '' : t('auth.noPerm')" @click="imp.mutate({ platform: iPlatform, month, year })">
        {{ imp.isPending.value ? t('common.loading') : t('dash.import') }}
      </button>
    </div>
    <p v-if="imp.data.value" class="text-muted text-sm" role="status">
      {{ t('dash.imported', { count: imp.data.value.imported, platform: iPlatform }) }}
    </p>
    <p v-if="imp.error.value" class="text-danger text-[13.5px] font-semibold mt-2.5" role="alert">
      {{ imp.error.value instanceof Error ? imp.error.value.message : imp.error.value }}
    </p>
  </div>

  <!-- KPI drill-down -->
  <div v-else-if="kind === 'kpi'" class="grid gap-3 text-left">
    <h3 class="m-0 font-display text-2xl leading-[1.2] uppercase tracking-[-0.01em]">{{ kpiTitle }} ({{ kpiRows.length }})</h3>
    <ul v-if="kpiRows.length" class="list-none m-0 p-0 grid gap-0.5 max-h-[320px] overflow-auto">
      <li v-for="r in kpiRows" :key="r.id" class="grid gap-0.5 px-0.5 py-2 border-b border-faint">
        <span class="text-md font-semibold">{{ r.topic }}</span>
        <span class="text-muted text-[12.5px]">
          {{ r.date ?? '—' }} · {{ r.status }}
          <template v-if="showMetrics"> · {{ r.views.toLocaleString('en-US') }} views · {{ r.likes.toLocaleString('en-US') }} likes</template>
        </span>
      </li>
    </ul>
    <p v-else class="text-muted text-sm">{{ t('dash.noRows') }}</p>
    <div class="flex gap-2">
      <RouterLink class="w-auto flex-none font-sans uppercase tracking-[0.08em] rounded-full px-5.5 py-3 text-[12.5px] font-semibold border-[1.5px] border-ink bg-transparent text-ink cursor-pointer shadow-none hover:bg-ink hover:text-white hover:border-ink active:transform-none disabled:opacity-50 disabled:cursor-default text-center no-underline" :to="crossLink.to">{{ crossLink.label }}</RouterLink>
      <button class="w-auto flex-none font-sans uppercase tracking-[0.08em] rounded-full px-5.5 py-3 text-[12.5px] font-semibold border-[1.5px] border-line bg-transparent text-muted cursor-pointer shadow-none hover:border-ink hover:text-ink" @click="$emit('close')">{{ t('common.close') }}</button>
    </div>
  </div>

  <!-- Empty state (wide card, nothing picked yet) -->
  <div v-else class="grid gap-3 text-left">
    <p class="text-muted text-sm mt-0">{{ t('dash.detailHint') }}</p>
  </div>
</template>
