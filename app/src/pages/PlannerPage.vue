<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import {
  usePosts, useUpdatePost, useAddPost, useDeletePost, useSetup, useLockPost, useUnlockPost,
  usePermission,
} from '@/core/queries'
import { currentName } from '@/core/auth'
import { t } from '@/core/i18n'
import { MONTHS } from '@/mock/db'
import { activePlatforms, platformIdBySetupName } from '@/core/platforms'
import type { Post, Status } from '@/mock/db'
import { guardSuppressed } from '@/core/protected'
import CarouselTabs from '@/components/ui/CarouselTabs.vue'
import MasterTable from '@/components/tables/MasterTable.vue'
import StatusFunnel from '@/components/ui/StatusFunnel.vue'
import CopyBar from '@/components/ui/CopyBar.vue'
import ProtectedModal from '@/components/overlays/ProtectedModal.vue'
import Chip from '@/components/ui/Chip.vue'

const props = defineProps<{ month?: string }>()
const route = useRoute()
const router = useRouter()

function validMonth(value: unknown): number | null {
  const n = Number(value)
  return Number.isInteger(n) && n >= 1 && n <= 12 ? n : null
}

const currentMonth = new Date().getMonth() + 1
const month = ref<number>(
  validMonth(props.month) ?? validMonth(route.params.month) ?? currentMonth,
)
const m = computed(() => month.value)

// Keep the route param and the selected month in sync (deep links + tab clicks).
watch(
  () => props.month,
  (value) => {
    const n = validMonth(value)
    if (n !== null && n !== month.value) month.value = n
  },
)

const { data: posts, isPending, isError, error: loadError, refetch } = usePosts(m)
const { data: setup } = useSetup()
const updatePost = useUpdatePost(m)
const addPost = useAddPost(m)
const deletePost = useDeletePost(m)
const lockPost = useLockPost(m)
const unlockPost = useUnlockPost(m)
const { can } = usePermission()

const canWrite = computed(() => can('posts.write'))
const canLock = computed(() => can('posts.lock'))

// In demo mode (auth off) there is no signed-in account: use the workspace
// owner as the actor so locking/editing stays usable.
const actor = computed(() => currentName.value ?? setup.value?.owner ?? '')

const selectedId = ref<string | null>(null)
const selected = computed(() => (posts.value ?? []).find((p) => p.id === selectedId.value) ?? null)
const draft = ref<Partial<Post>>({})
const guardOpen = ref(false)

const lockedByOther = computed(() =>
  Boolean(selected.value?.lockedBy && selected.value.lockedBy !== actor.value),
)
const lockedByMe = computed(() =>
  Boolean(selected.value?.lockedBy && selected.value.lockedBy === actor.value),
)
const actionError = computed(() => {
  const e = lockPost.error.value ?? unlockPost.error.value ?? updatePost.error.value
    ?? addPost.error.value ?? deletePost.error.value
  return e ? (e instanceof Error ? e.message : String(e)) : ''
})

const busy = computed(() =>
  updatePost.isPending.value || addPost.isPending.value || deletePost.isPending.value,
)

/** Setup platform names for the platforms the workspace actually uses. */
function defaultPlatforms(): string[] {
  const names = setup.value?.platforms ?? []
  // Instagram rides on the Meta connection, so it counts as active too.
  const active = names.filter((name) => {
    const id = platformIdBySetupName(name)
    return id === 'instagram' || (id !== undefined && activePlatforms.value.includes(id))
  })
  return active.length ? active : [...names]
}

/** Sensible defaults so a fresh month can be set up in one click. */
function blankRow(): Omit<Post, 'id'> {
  const setupRow = setup.value
  const year = setupRow?.year ?? new Date().getFullYear()
  const now = new Date()
  const inMonth = now.getFullYear() === year && now.getMonth() + 1 === month.value
  const pad = (n: number): string => String(n).padStart(2, '0')
  const date = `${year}-${pad(month.value)}-${pad(inMonth ? now.getDate() : 1)}`
  return {
    month: month.value,
    topic: t('planner.defaultTopic'),
    pillar: setupRow?.pillars[0] ?? '',
    format: setupRow?.formats[0] ?? '',
    goal: setupRow?.goals[0] ?? '',
    date,
    time: '18:00',
    status: (setupRow?.statuses[0] ?? 'Start') as Status,
    hook: '',
    caption: '',
    cta: '',
    hashtagGroup: '',
    hashtags: [],
    imageUrl: '',
    note: '',
    done: false,
    platforms: defaultPlatforms(),
  }
}

async function addRow(): Promise<void> {
  if (!canWrite.value) return
  try {
    const created = await addPost.mutateAsync(blankRow())
    pick(created)
  } catch {
    // surfaced through actionError
  }
}

async function duplicateRow(): Promise<void> {
  if (!canWrite.value || !selected.value) return
  try {
    const created = await addPost.mutateAsync({
      ...blankRow(),
      ...selected.value,
      topic: `${selected.value.topic} (copy)`,
      hashtags: [...selected.value.hashtags],
      platforms: [...selected.value.platforms],
    } as Omit<Post, 'id'>)
    pick(created)
  } catch {
    // surfaced through actionError
  }
}

async function removeRow(): Promise<void> {
  if (!canWrite.value || !selected.value) return
  if (typeof window !== 'undefined' && !window.confirm(t('planner.deleteConfirm'))) return
  try {
    await deletePost.mutateAsync(selected.value.id)
    selectedId.value = null
    draft.value = {}
  } catch {
    // surfaced through actionError
  }
}

function toggleLock(): void {
  if (!canLock.value || !selected.value || !actor.value) return
  if (lockedByMe.value) unlockPost.mutate(selected.value.id)
  else lockPost.mutate(selected.value.id)
}

function localToday(): string {
  const d = new Date()
  const pad = (n: number): string => String(n).padStart(2, '0')
  return `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())}`
}

const todayStr = localToday()
const todayCount = computed(() => (posts.value ?? []).filter((p) => p.date === todayStr).length)

const pillarOptions = computed(() => setup.value?.pillars ?? [])
const formatOptions = computed(() => setup.value?.formats ?? [])
const goalOptions = computed(() => setup.value?.goals ?? [])
const statusOptions = computed<Status[]>(() => (setup.value?.statuses ?? []) as Status[])

function pick(p: Post): void {
  selectedId.value = p.id
  draft.value = { ...p, hashtags: [...p.hashtags], platforms: [...p.platforms] }
}

function goMonth(value: string | number): void {
  const next = validMonth(value)
  if (next === null || next === month.value) return
  month.value = next
  selectedId.value = null
  void router.replace({ path: `/planner/${next}`, query: route.query })
}

const assembled = computed(() => {
  const c = (draft.value.caption ?? '').trim()
  const a = (draft.value.cta ?? '').trim()
  const tags = (draft.value.hashtags ?? []).join(' ')
  return [c, a, tags].filter(Boolean).join('\n')
})

// Only these fields are user-editable — never round-trip id/month/lockedBy/done.
const EDITABLE: (keyof Post)[] = [
  'topic', 'pillar', 'format', 'goal', 'date', 'time', 'status', 'hook', 'caption',
  'cta', 'hashtagGroup', 'hashtags', 'imageUrl', 'note', 'platforms',
]

function save(): void {
  if (!canWrite.value || !selected.value) return
  const patch: Partial<Post> & { user?: string } = { user: actor.value || undefined }
  for (const key of EDITABLE) {
    const value = draft.value[key]
    if (value !== undefined) (patch as Record<string, unknown>)[key] = value
  }
  updatePost.mutate({ id: selected.value.id, patch })
}

function togglePlatform(platform: string): void {
  const current = [...(draft.value.platforms ?? [])]
  const index = current.indexOf(platform)
  if (index >= 0) current.splice(index, 1)
  else current.push(platform)
  draft.value.platforms = current
}

function openGuard(): void {
  if (guardSuppressed()) return
  guardOpen.value = true
}

// Selecting a card from the Monthly page deep-links to /planner/{month}?post=id.
watch(
  [posts, () => route.query.post],
  ([rows, id]) => {
    if (typeof id !== 'string' || !rows) return
    const found = rows.find((p) => p.id === id)
    if (found && selectedId.value !== found.id) pick(found)
  },
  { immediate: true },
)
</script>

<template>
  <h1>{{ t('planner.title').replace('{month}', String(m).padStart(2, '0')) }}</h1>
  <CarouselTabs :items="MONTHS" :model-value="m" @update:model-value="goMonth" />
  <div class="row planner-bar">
    <p class="muted">{{ (posts?.length ?? 0) }} {{ t('planner.topics') }} · {{ todayCount }} {{ t('planner.today') }}</p>
    <button class="btn btn-primary planner-add" :disabled="!canWrite || busy"
      :title="canWrite ? '' : t('auth.noPerm')" @click="addRow">
      {{ addPost.isPending.value ? t('planner.adding') : t('planner.addTopic') }}
    </button>
  </div>

  <div v-if="isPending" class="muted">Loading rows…</div>
  <div v-else-if="isError" class="card">
    <p class="muted">Could not load posts: {{ loadError?.message }}</p>
    <button class="btn" @click="() => refetch()">{{ t('common.tryAgain') }}</button>
  </div>
  <MasterTable v-else-if="(posts ?? []).length" :rows="posts ?? []" @select="pick" />
  <div v-else class="card planner-empty">
    <p class="muted">{{ t('planner.empty') }}</p>
    <button class="btn btn-primary" :disabled="!canWrite || busy"
      :title="canWrite ? '' : t('auth.noPerm')" @click="addRow">{{ t('planner.addTopic') }}</button>
  </div>

  <p v-if="!selected && actionError" class="autherr" role="alert">{{ actionError }}</p>
  <div v-if="selected" class="card mt">
    <h2>{{ selected.topic || t('planner.newTopic') }}<span v-if="selected.lockedBy" class="lockchip">Locked by {{ selected.lockedBy }}</span></h2>
    <div class="grid3">
      <div>
        <label class="lbl" for="post-topic">{{ t('planner.topic') }}</label>
        <input id="post-topic" class="field" v-model="draft.topic" />
      </div>
      <div>
        <label class="lbl" for="post-pillar">Pillar</label>
        <select id="post-pillar" class="field" v-model="draft.pillar">
          <option v-for="o in pillarOptions" :key="o" :value="o">{{ o }}</option>
        </select>
      </div>
      <div>
        <label class="lbl" for="post-format">Format</label>
        <select id="post-format" class="field" v-model="draft.format">
          <option v-for="o in formatOptions" :key="o" :value="o">{{ o }}</option>
        </select>
      </div>
    </div>
    <div class="grid3">
      <div>
        <label class="lbl" for="post-goal">Goal</label>
        <select id="post-goal" class="field" v-model="draft.goal">
          <option v-for="o in goalOptions" :key="o" :value="o">{{ o }}</option>
        </select>
      </div>
      <div>
        <label class="lbl" for="post-date">Date</label>
        <input id="post-date" class="field" type="date" v-model="draft.date" />
      </div>
      <div>
        <label class="lbl" for="post-time">Time</label>
        <input id="post-time" class="field" type="time" v-model="draft.time" />
      </div>
    </div>
    <label class="lbl" for="post-status">Status</label>
    <select id="post-status" class="field" v-model="draft.status">
      <option v-for="s in statusOptions" :key="s" :value="s">{{ s }}</option>
    </select>
    <label class="lbl" for="post-hook">Hook</label>
    <input id="post-hook" class="field" v-model="draft.hook" />
    <label class="lbl" for="post-caption">Caption</label>
    <textarea id="post-caption" class="field" rows="2" v-model="draft.caption" />
    <div class="grid2">
      <div><label class="lbl" for="post-cta">CTA</label><input id="post-cta" class="field" v-model="draft.cta" /></div>
      <div><label class="lbl" for="post-group">Hashtag group</label><input id="post-group" class="field" v-model="draft.hashtagGroup" /></div>
    </div>
    <label class="lbl">Workflow</label>
    <div><StatusFunnel :current="(draft.status ?? 'Start') as Status" @advance="(s) => { draft.status = s }" /></div>
    <label class="lbl">Platforms</label>
    <div>
      <Chip v-for="pl in (setup?.platforms ?? [])" :key="pl" :label="pl"
        :dark="(draft.platforms ?? []).includes(pl)" role="button" tabindex="0"
        :aria-pressed="(draft.platforms ?? []).includes(pl)" style="cursor: pointer;"
        @click="togglePlatform(pl)"
        @keydown.enter.prevent="togglePlatform(pl)" @keydown.space.prevent="togglePlatform(pl)" />
    </div>
    <div class="mt"><CopyBar :text="assembled" /></div>
    <p v-if="actionError" class="autherr" role="alert">{{ actionError }}</p>
    <div class="row mt">
      <button class="btn btn-primary" :disabled="lockedByOther || busy || !canWrite"
        :title="canWrite ? '' : t('auth.noPerm')" @click="save">
        {{ updatePost.isPending.value ? t('planner.saving') : t('planner.save') }}
      </button>
      <button class="btn" :disabled="lockedByOther || busy || !canWrite"
        :title="canWrite ? '' : t('auth.noPerm')" @click="duplicateRow">{{ t('planner.duplicate') }}</button>
      <button class="btn danger" :disabled="lockedByOther || busy || !canWrite"
        :title="canWrite ? '' : t('auth.noPerm')" @click="removeRow">{{ t('planner.delete') }}</button>
      <button class="btn" :disabled="!actor || lockedByOther || !canLock || lockPost.isPending.value || unlockPost.isPending.value"
        :title="!canLock ? t('auth.noPerm') : (actor ? 'Lock this row while editing' : 'Log in to lock rows')" @click="toggleLock">
        {{ lockedByMe ? 'Unlock' : 'Lock' }}
      </button>
      <button class="btn" @click="openGuard">Request edit of computed field</button>
    </div>
    <p v-if="lockedByOther" class="muted">Row is being edited by {{ selected.lockedBy }} — saving is blocked.</p>
  </div>
  <ProtectedModal :open="guardOpen" field="assembled Copy-Paste"
    @cancel="guardOpen = false" @confirm="guardOpen = false" />
</template>

<style scoped>
.planner-bar { justify-content: space-between; }
.planner-add { margin-left: auto; }
.planner-empty { display: flex; flex-direction: column; align-items: flex-start; gap: 10px; }
.btn.danger { color: var(--danger); border-color: var(--danger); }
.btn.danger:hover { background: var(--danger-wash); }
.lockchip {
  margin-left: 8px;
  padding: 2px 8px;
  border: 1px solid var(--faint);
  border-radius: 999px;
  font-size: 12px;
  font-weight: 400;
  vertical-align: middle;
}
</style>
