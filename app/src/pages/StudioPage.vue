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

// Auto-select the first item on desktop so the right pane is immediately active,
// and recover when the current id belongs to another hub (or no longer exists).
watch(
  [() => rows.value, () => wide.value, () => selectedId.value],
  ([list, isWide, id]) => {
    if (!isWide || !list?.length) return
    if (id && list.some((row) => row.id === id)) return
    if (props.embedded) emit('update:id', list[0].id)
    else void router.replace(`/studio/${list[0].id}`)
  },
  { immediate: true },
)

// Mobile-only bottom drawer: only active when card is narrow
const editorOpen = ref(false)
let mounted = false
onMounted(() => {
  mounted = true
  if (selectedId.value && !wide.value && !deckFull.value) editorOpen.value = true
})
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
watch([wide, deckFull], ([isWide, full]) => {
  if (isWide || full) editorOpen.value = false
  else if (selectedId.value) editorOpen.value = true
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

    <div class="studio-shell flex-1 min-h-0 grid grid-cols-1 md:grid-cols-[340px_minmax(0,1fr)] gap-0 rounded-2xl bg-surface overflow-hidden mt-2">
      <!-- Left pane: Article List -->
      <div class="border-r border-line/40 flex flex-col min-h-0 overflow-y-auto p-3 md:px-0 md:py-3">
        <div v-if="listQ.isPending.value" class="muted p-4 text-center">{{ t('studio.loading') }}</div>
        <div v-else-if="listQ.isError.value" class="py-4 grid gap-2 justify-items-center text-center">
          <p class="muted">{{ listQ.error.value?.message }}</p>
          <button class="btn w-auto" @click="() => listQ.refetch()">{{ t('common.tryAgain') }}</button>
        </div>
        <p v-else-if="!rows.length" class="muted p-6 text-center">{{ t('studio.empty') }}</p>
        <template v-else>
          <button v-for="row in rows" :key="row.id" type="button" class="camp-item card-fade group"
            :class="{ active: row.id === selectedId }" @click="select(row.id)">
            <div class="camp-item-head">
              <div class="min-w-0 flex-1 pr-2">
                <strong class="block text-[16.8px] font-bold text-ink tracking-tight truncate">{{ row.title || t('studio.untitled') }}</strong>
                <p class="text-[13.2px] text-muted truncate m-0 mt-0.5">{{ row.excerpt || t('studio.noExcerpt') }}</p>
              </div>
              <span class="camp-status flex-none" :class="row.status">{{ statusLabel(row.status) }}</span>
            </div>

            <div class="flex items-center gap-1.5 text-[13.8px] text-ink-soft font-mono mt-0.5">
              <svg class="w-3.5 h-3.5 text-muted flex-none" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
                <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"></path>
                <polyline points="14 2 14 8 20 8"></polyline>
                <line x1="16" y1="13" x2="8" y2="13"></line>
                <line x1="16" y1="17" x2="8" y2="17"></line>
              </svg>
              <span class="truncate">{{ row.author }} · {{ updatedLabel(row) }}</span>
            </div>

            <div class="flex items-center justify-between gap-2 text-xs text-muted mt-1.5 w-full pt-2.5 border-t border-black/[0.06]">
              <div class="flex items-center gap-2 min-w-0">
                <span class="inline-flex items-center gap-1 font-mono text-[13.2px] text-muted min-w-0">
                  <svg class="w-3.5 h-3.5 flex-none" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
                    <path d="M20.59 13.41 12 22l-9-9V4a1 1 0 0 1 1-1h9l7.59 7.59a2 2 0 0 1 0 2.82z"></path>
                    <circle cx="7.5" cy="7.5" r="1.5"></circle>
                  </svg>
                  <span class="truncate">{{ row.tags.length ? row.tags.join(', ') : '—' }}</span>
                </span>
              </div>
              <span class="font-mono font-bold text-ink text-[15.6px] tracking-tight tabular-nums flex-none">
                {{ row.wordCount }} {{ t('studio.words') }}
              </span>
            </div>
          </button>
        </template>

      </div>

      <!-- Right pane: In-Place Content Editor (wide layouts only; narrow uses the drawer) -->
      <div v-if="wide || deckFull" class="flex flex-col min-h-0 overflow-y-auto p-4 md:p-6 bg-surface">
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

<style scoped>
.studio-page > .studio-shell {
  flex: 1 1 0%;
  min-height: 0;
}

.studio-shell > * {
  min-height: 0;
}
</style>
