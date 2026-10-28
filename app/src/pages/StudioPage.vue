<script setup lang="ts">
// Client-facing Content Studio: list + editor for articles/pages/notes.
// Reachable by every account that has the `content.*` permissions (the seeded
// Client role), while staff reach it from the main nav.
import { computed, onMounted, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useContentList, useCreateContent, usePermission } from '@/core/queries'
import { t } from '@/core/i18n'
import DockBar from '@/components/ui/DockBar.vue'
import DropdownMenu from '@/components/ui/DropdownMenu.vue'
import type { ContentStatus, ContentSummary } from '@/mock/db'
import ContentEditor from '@/components/editor/ContentEditor.vue'
import Drawer from '@/components/overlays/Drawer.vue'
import { deckFull } from '@/core/deck'
import { useWidePane } from '@/composables/useWidePane'

const props = defineProps<{ id?: string; embedded?: boolean }>()
const emit = defineEmits<{ 'update:id': [string] }>()
const route = useRoute()
const router = useRouter()
const { can, isLoggedIn } = usePermission()

const STATUSES: ContentStatus[] = ['draft', 'review', 'scheduled', 'published', 'archived']
const canWrite = computed(() => can('content.write'))

const status = ref<'' | ContentStatus>('')
const params = computed(() => ({
  status: status.value || undefined,
}))
const listQ = useContentList(params)
const create = useCreateContent()

// The studio's wide layout (configs | items | detail) follows the card's own
// width, so it aligns progressively while the card expands — not in one snap
// when the deck flips to full.
const root = ref<HTMLElement | null>(null)
const { wide } = useWidePane(root, 'studio-scroll')

const selectedId = computed(() =>
  props.id ?? (typeof route.params.id === 'string' ? route.params.id : ''),
)
const rows = computed<ContentSummary[]>(() => listQ.data.value ?? [])
const selectedRow = computed(() => rows.value.find((r) => r.id === selectedId.value) ?? null)

// Auto-select first item on desktop so right pane is immediately active and ready
watch(
  [() => rows.value, () => wide.value],
  ([list, isWide]) => {
    if (isWide && !selectedId.value && list && list.length > 0) {
      if (props.embedded) emit('update:id', list[0].id)
      else void router.replace(`/studio/${list[0].id}`)
    }
  },
  { immediate: true },
)

// Mobile-only bottom drawer: only active when card is narrow
const editorOpen = ref(false)
let mounted = false
onMounted(() => { mounted = true })
watch(selectedId, (id) => {
  if (id && mounted && !wide.value && !deckFull.value) {
    editorOpen.value = true
  }
})
watch(editorOpen, (open) => {
  if (open || !selectedId.value) return
  if (!wide.value && !deckFull.value) {
    if (props.embedded) emit('update:id', '')
    else void router.replace('/studio')
  }
})

function select(id: string): void {
  if (props.embedded) emit('update:id', id)
  else void router.push(`/studio/${id}`)
  if (!wide.value && !deckFull.value) {
    editorOpen.value = true
  }
}

async function newContent(): Promise<void> {
  if (!canWrite.value || create.isPending.value) return
  try {
    const created = await create.mutateAsync({ title: 'Untitled', kind: 'article' })
    if (props.embedded) emit('update:id', created.id)
    else void router.push(`/studio/${created.id}`)
    if (!wide.value && !deckFull.value) {
      editorOpen.value = true
    }
  } catch {
    // the list surfaces mutation errors; creating is best-effort here
  }
}

function statusLabel(s: ContentStatus): string {
  return t(`studio.status.${s}`)
}

function statusBadgeClass(_s: ContentStatus): string {
  return 'bg-ink text-white'
}

// Status filter as a toggle button: press to open the status menu.
const statusMenuItems = computed(() => [
  { id: '', label: t('studio.all') },
  ...STATUSES.map((s) => ({ id: s, label: statusLabel(s) })),
])
const currentStatusLabel = computed(() =>
  status.value === '' ? t('studio.all') : statusLabel(status.value as ContentStatus),
)

function onStatus(value: string): void {
  status.value = value as '' | ContentStatus
}

function updatedLabel(row: ContentSummary): string {
  const stamp = row.publishedAt && row.status === 'published' ? row.publishedAt : row.updatedAt
  const date = new Date(stamp)
  if (Number.isNaN(date.getTime())) return ''
  return date.toLocaleDateString(undefined, { month: 'short', day: 'numeric', year: 'numeric' })
}
</script>

<template>
  <div v-if="!isLoggedIn" class="card mt" style="max-width: 560px; margin: 40px auto;">
    <p class="muted">{{ t('studio.signInHint') }}</p>
  </div>

  <div v-else ref="root" class="studio-page h-full flex flex-col min-h-0 text-left">
    <DockBar label="Studio controls">
      <div class="flex items-center justify-between w-full gap-3">
        <div class="flex items-center gap-2">
          <DropdownMenu :items="statusMenuItems" @select="onStatus">
            <template #trigger>
              <button type="button" class="btn w-auto inline-flex items-center gap-2" :aria-label="t('studio.filter')">
                <span>{{ currentStatusLabel }}</span> <span aria-hidden="true">▾</span>
              </button>
            </template>
          </DropdownMenu>
          <span class="text-xs text-muted font-medium">{{ rows.length }} items</span>
        </div>
        <button type="button" class="btn btn-primary btn-icon w-auto" :title="t('studio.new')" :aria-label="t('studio.new')"
          :disabled="!canWrite || create.isPending.value" @click="newContent">
          {{ create.isPending.value ? '…' : '+' }}
        </button>
      </div>
    </DockBar>

    <div class="flex-1 min-h-0 grid grid-cols-1 md:grid-cols-12 gap-0 rounded-2xl bg-surface overflow-hidden mt-2">
      <!-- Left pane: Article List -->
      <div class="md:col-span-5 border-r border-line/40 flex flex-col min-h-0 overflow-y-auto p-3 gap-2">
        <div v-if="listQ.isPending.value" class="muted p-4 text-center">{{ t('studio.loading') }}</div>
        <div v-else-if="listQ.isError.value" class="py-4 grid gap-2 justify-items-center text-center">
          <p class="muted">{{ listQ.error.value?.message }}</p>
          <button class="btn w-auto" @click="() => listQ.refetch()">{{ t('common.tryAgain') }}</button>
        </div>
        <p v-else-if="!rows.length" class="muted p-6 text-center">{{ t('studio.empty') }}</p>
        <template v-else>
          <button v-for="row in rows" :key="row.id"
            class="w-full text-left p-3.5 rounded-xl border transition-all cursor-pointer flex flex-col gap-1.5"
            :class="row.id === selectedId ? 'border-ink bg-wash/60 shadow-xs' : 'border-line/60 bg-surface hover:border-line hover:bg-wash/30'"
            @click="select(row.id)">
            <div class="flex items-start justify-between gap-2">
              <strong class="text-sm font-bold text-ink leading-snug line-clamp-1">{{ row.title || t('studio.untitled') }}</strong>
              <span class="inline-flex items-center px-2 py-0.5 rounded-full text-[10.5px] font-bold uppercase tracking-wider flex-none"
                :class="statusBadgeClass(row.status)">
                {{ statusLabel(row.status) }}
              </span>
            </div>
            <span class="muted text-xs line-clamp-2 leading-relaxed">{{ row.excerpt || t('studio.noExcerpt') }}</span>
            <div class="text-[11px] text-muted flex items-center gap-1.5 mt-0.5">
              <span>{{ row.author }}</span>
              <span>·</span>
              <span>{{ updatedLabel(row) }}</span>
              <span>·</span>
              <span>{{ row.wordCount }} {{ t('studio.words') }}</span>
              <template v-if="row.tags.length">
                <span>·</span>
                <span class="truncate">{{ row.tags.join(', ') }}</span>
              </template>
            </div>
          </button>
        </template>
      </div>

      <!-- Right pane: In-Place Content Editor -->
      <div class="md:col-span-7 flex flex-col min-h-0 overflow-y-auto p-4 md:p-6 bg-surface">
        <ContentEditor v-if="selectedRow" :key="selectedRow.id" :id="selectedRow.id" />
        <div v-else class="h-full flex flex-col items-center justify-center p-8 text-center text-muted gap-2 min-h-[300px]">
          <svg width="40" height="40" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" class="opacity-30 mb-2">
            <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z" />
            <polyline points="14 2 14 8 20 8" />
            <line x1="16" y1="13" x2="8" y2="13" />
            <line x1="16" y1="17" x2="8" y2="17" />
          </svg>
          <p class="font-bold text-ink text-sm">{{ t('studio.pick') }}</p>
          <p class="text-xs max-w-xs">{{ t('studio.pickHint') }}</p>
        </div>
      </div>
    </div>

    <!-- Mobile-only contained bottom drawer (never leaks to desktop deck) -->
    <Drawer v-if="!deckFull && !wide" v-model="editorOpen" side="bottom" contained
      :title="selectedRow?.title || t('studio.untitled')">
      <ContentEditor v-if="selectedRow" :key="selectedRow.id" :id="selectedRow.id" />
    </Drawer>
  </div>
</template>
