<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import {
  usePosts, useUpdatePost, useAddPost, useDeletePost, useSetup, useLockPost, useUnlockPost,
  usePermission,
} from '@/core/queries'
import { currentName } from '@/core/auth'
import { t } from '@/core/i18n'
import { activePlatforms, platformIdBySetupName } from '@/core/platforms'
import { deckFull } from '@/core/deck'
import { activeMonth } from '@/core/navdate'
import { useWidePane } from '@/composables/useWidePane'
import type { Post, Status } from '@/mock/db'
import { guardSuppressed } from '@/core/protected'
import MasterTable from '@/components/tables/MasterTable.vue'
import MonthToggle from '@/components/ui/MonthToggle.vue'
import StatusFunnel from '@/components/ui/StatusFunnel.vue'
import CopyBar from '@/components/ui/CopyBar.vue'
import Drawer from '@/components/overlays/Drawer.vue'
import CommandPalette from '@/components/overlays/CommandPalette.vue'
import ProtectedModal from '@/components/overlays/ProtectedModal.vue'
import Chip from '@/components/ui/Chip.vue'
import IconButton from '@/components/ui/IconButton.vue'

const props = defineProps<{ month?: string; embedded?: boolean }>()
const emit = defineEmits<{ 'update:month': [string] }>()
const route = useRoute()
const router = useRouter()
const root = ref<HTMLElement | null>(null)
const { wide } = useWidePane(root, 'planner-scroll')

function validMonth(value: unknown): number | null {
  const n = Number(value)
  return Number.isInteger(n) && n >= 1 && n <= 12 ? n : null
}

const currentMonth = new Date().getMonth() + 1
const month = ref<number>(
  validMonth(props.month) ?? validMonth(route.params.month) ?? currentMonth,
)
const m = computed(() => month.value)

// The deck works on one month: whatever month any card picks, every other
// card follows. The planner keeps its URL as its own deep-link source of
// truth, so external changes move its content only — never the route.
watch(month, (value) => {
  activeMonth.value = value
})
watch(activeMonth, (value) => {
  if (validMonth(value) !== null && value !== month.value) month.value = value
})

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
// The row editor lives in a popup; closing it drops the selection.
const editorOpen = ref(false)

watch(editorOpen, (open) => {
  if (!open) {
    selectedId.value = null
    draft.value = {}
  }
})

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
    editorOpen.value = false
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

const pillarOptions = computed(() => setup.value?.pillars ?? [])
const formatOptions = computed(() => setup.value?.formats ?? [])
const goalOptions = computed(() => setup.value?.goals ?? [])
const statusOptions = computed<Status[]>(() => (setup.value?.statuses ?? []) as Status[])

function pick(p: Post): void {
  selectedId.value = p.id
  draft.value = { ...p, hashtags: [...p.hashtags], platforms: [...p.platforms] }
  editorOpen.value = true
}

// Topic finder — a search popup opened with Ctrl/⌘ + K (the toolbar button
// shows the same affordance). Picking a result opens that row's editor.
const searchOpen = ref(false)
const searchItems = computed(() =>
  (posts.value ?? []).map((p) => ({ id: p.id, label: p.topic, group: p.date ?? undefined })),
)

function onSearchSelect(id: string): void {
  const post = (posts.value ?? []).find((p) => p.id === id)
  if (post) pick(post)
}

function onSearchKey(e: KeyboardEvent): void {
  // Search is a full-page tool: the Plan hub must be current and expanded.
  if (!route.path.startsWith('/plan') || !deckFull.value) return
  if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === 'k') {
    e.preventDefault()
    searchOpen.value = true
  }
}

watch(deckFull, (value) => {
  if (!value) searchOpen.value = false
})

onMounted(() => window.addEventListener('keydown', onSearchKey))
onBeforeUnmount(() => window.removeEventListener('keydown', onSearchKey))

function goMonth(value: string | number): void {
  const next = validMonth(value)
  if (next === null || next === month.value) return
  month.value = next
  editorOpen.value = false
  selectedId.value = null
  // Embedded in the Plan hub: the parent owns the tab/month and the URL.
  if (props.embedded) emit('update:month', String(next))
  else void router.replace({ path: `/planner/${next}`, query: route.query })
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

// Selecting a card from the calendar deep-links to /plan?…&post=id. The link is
// consumed once: after opening the editor we drop `post` from the URL so a
// refresh (or back/forward) does not re-open the popup.
watch(
  [posts, () => route.query.post],
  ([rows, id]) => {
    if (typeof id !== 'string' || !rows) return
    const found = rows.find((p) => p.id === id)
    if (!found) return
    if (selectedId.value !== found.id) pick(found)
    const query = { ...route.query }
    delete query.post
    void router.replace({ query })
  },
  { immediate: true },
)
</script>

<template>
  <div class="planner-page" ref="root">
  <h1>{{ t('planner.heading') }}</h1>

  <div class="planner-grid">
    <aside class="planner-side">
      <MonthToggle :model-value="m" @update:model-value="goMonth" />
      <button class="btn btn-primary planner-add" :disabled="!canWrite || busy"
        :title="canWrite ? '' : t('auth.noPerm')" @click="addRow">
        {{ addPost.isPending.value ? t('planner.adding') : t('planner.addTopic') }}
      </button>
    </aside>

    <div class="planner-main">
      <div v-if="isPending" class="muted">Loading rows…</div>
      <div v-else-if="isError" class="card">
        <p class="muted">Could not load posts: {{ loadError?.message }}</p>
        <button class="btn" @click="() => refetch()">{{ t('common.tryAgain') }}</button>
      </div>
      <MasterTable v-else-if="(posts ?? []).length" :rows="posts ?? []" @select="pick">
        <template #actions>
          <button v-if="deckFull" type="button" class="ghostbtn searchbtn" title="Search topics (Ctrl/⌘ K)" @click="searchOpen = true">
            <svg viewBox="0 0 16 16" width="14" height="14" fill="none" stroke="currentColor"
              stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
              <circle cx="7" cy="7" r="4.5" />
              <path d="M10.5 10.5 14 14" />
            </svg>
            <kbd>⌘K</kbd>
          </button>
        </template>
      </MasterTable>
      <div v-else class="card planner-empty">
        <p class="muted">{{ t('planner.empty') }}</p>
        <button class="btn btn-primary" :disabled="!canWrite || busy"
          :title="canWrite ? '' : t('auth.noPerm')" @click="addRow">{{ t('planner.addTopic') }}</button>
      </div>

      <p v-if="!selected && actionError" class="autherr" role="alert">{{ actionError }}</p>
    </div>

    <div v-if="(deckFull || wide) && !editorOpen" class="planner-detail">
      <p class="muted">{{ t('planner.pick') }}</p>
      <p class="muted">{{ t('planner.pickHint') }}</p>
    </div>
    <Drawer
      v-model="editorOpen"
      class="editor-sheet"
      :inline="deckFull || wide"
      :side="deckFull ? 'right' : 'bottom'"
      :width="'480px'"
      contained
      :title="draft.topic || t('planner.newTopic')"
    >
    <template v-if="selected">
    <label class="lbl" for="post-topic">{{ t('planner.topic') }}</label>
    <input id="post-topic" class="field" v-model="draft.topic" />
    <div class="grid3">
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
      <div>
        <label class="lbl" for="post-goal">Goal</label>
        <select id="post-goal" class="field" v-model="draft.goal">
          <option v-for="o in goalOptions" :key="o" :value="o">{{ o }}</option>
        </select>
      </div>
    </div>
    <div class="grid3">
      <div>
        <label class="lbl" for="post-date">Date</label>
        <input id="post-date" class="field" type="date" v-model="draft.date" />
      </div>
      <div>
        <label class="lbl" for="post-time">Time</label>
        <input id="post-time" class="field" type="time" v-model="draft.time" />
      </div>
      <div>
        <label class="lbl" for="post-status">Status</label>
        <select id="post-status" class="field" v-model="draft.status">
          <option v-for="s in statusOptions" :key="s" :value="s">{{ s }}</option>
        </select>
      </div>
    </div>
    <div class="grid3">
      <div>
        <label class="lbl" for="post-hook">Hook</label>
        <input id="post-hook" class="field" v-model="draft.hook" />
      </div>
      <div><label class="lbl" for="post-cta">CTA</label><input id="post-cta" class="field" v-model="draft.cta" /></div>
      <div><label class="lbl" for="post-group">Hashtag group</label><input id="post-group" class="field" v-model="draft.hashtagGroup" /></div>
    </div>
    <label class="lbl" for="post-caption">Caption</label>
    <textarea id="post-caption" class="field" rows="2" v-model="draft.caption" />
    <div class="grid2">
      <div>
        <label class="lbl">Workflow</label>
        <div><StatusFunnel :current="(draft.status ?? 'Start') as Status" @advance="(s) => { draft.status = s }" /></div>
      </div>
      <div>
        <label class="lbl">Platforms</label>
        <div>
          <Chip v-for="pl in (setup?.platforms ?? [])" :key="pl" :label="pl"
            :dark="(draft.platforms ?? []).includes(pl)" role="button" tabindex="0"
            :aria-pressed="(draft.platforms ?? []).includes(pl)" style="cursor: pointer;"
            @click="togglePlatform(pl)"
            @keydown.enter.prevent="togglePlatform(pl)" @keydown.space.prevent="togglePlatform(pl)" />
        </div>
      </div>
    </div>
    <div class="mt"><CopyBar :text="assembled" compact /></div>
    <p v-if="actionError" class="autherr" role="alert">{{ actionError }}</p>
    <p v-if="lockedByOther" class="muted">Row is being edited by {{ selected.lockedBy }} — saving is blocked.</p>
    </template>
    <template #footer>
      <IconButton
        tone="primary"
        :label="updatePost.isPending.value ? t('planner.saving') : t('planner.save')"
        :disabled="lockedByOther || busy || !canWrite"
        @click="save"
      >
        <svg viewBox="0 0 16 16" width="16" height="16" fill="none" stroke="currentColor"
          stroke-width="1.7" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
          <path d="M3 8.5 6.5 12 13 4.5" />
        </svg>
      </IconButton>
      <IconButton label="Duplicate" :disabled="lockedByOther || busy || !canWrite" @click="duplicateRow">
        <svg viewBox="0 0 16 16" width="16" height="16" fill="none" stroke="currentColor"
          stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
          <rect x="5.5" y="5.5" width="8" height="8" rx="1.5" />
          <path d="M10.5 5.5V4A1.5 1.5 0 0 0 9 2.5H4A1.5 1.5 0 0 0 2.5 4v5A1.5 1.5 0 0 0 4 10.5h1.5" />
        </svg>
      </IconButton>
      <IconButton tone="danger" label="Delete" :disabled="lockedByOther || busy || !canWrite" @click="removeRow">
        <svg viewBox="0 0 16 16" width="16" height="16" fill="none" stroke="currentColor"
          stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
          <path d="M3 4.5h10M6.5 4.5V3h3v1.5M5 4.5l.6 8.2a1 1 0 0 0 1 .8h2.8a1 1 0 0 0 1-.8l.6-8.2" />
        </svg>
      </IconButton>
      <IconButton
        :label="lockedByMe ? 'Unlock' : (actor ? 'Lock this row while editing' : 'Log in to lock rows')"
        :disabled="!actor || lockedByOther || !canLock || lockPost.isPending.value || unlockPost.isPending.value"
        @click="toggleLock"
      >
        <svg v-if="lockedByMe" viewBox="0 0 16 16" width="16" height="16" fill="none" stroke="currentColor"
          stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
          <rect x="3.5" y="7" width="9" height="6.5" rx="1.5" />
          <path d="M10.5 7V5a2.5 2.5 0 0 0-4.9-.7" />
        </svg>
        <svg v-else viewBox="0 0 16 16" width="16" height="16" fill="none" stroke="currentColor"
          stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
          <rect x="3.5" y="7" width="9" height="6.5" rx="1.5" />
          <path d="M5.5 7V5a2.5 2.5 0 0 1 5 0v2" />
        </svg>
      </IconButton>
      <IconButton label="Request edit of computed field" @click="openGuard">
        <svg viewBox="0 0 16 16" width="16" height="16" fill="none" stroke="currentColor"
          stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
          <path d="M11 2.5 13.5 5 5 13.5H2.5V11z" />
        </svg>
      </IconButton>
    </template>
  </Drawer>
  </div>
  <ProtectedModal :open="guardOpen" field="assembled Copy-Paste"
    @cancel="guardOpen = false" @confirm="guardOpen = false" />
  <CommandPalette v-model="searchOpen" :items="searchItems" placeholder="Search topics…" @select="onSearchSelect" />
  </div>
</template>

<style scoped>
.planner-add { margin-left: 0; }
.searchbtn kbd {
  font-family: var(--mono, ui-monospace, Menlo, monospace);
  font-size: 11px;
  color: var(--muted);
  border: 1px solid var(--line);
  border-radius: 4px;
  padding: 1px 5px;
}
.planner-empty { display: flex; flex-direction: column; align-items: flex-start; gap: 10px; }
.btn.danger { color: var(--danger); border-color: var(--danger); }
.btn.danger:hover { background: var(--danger-wash); }
</style>
