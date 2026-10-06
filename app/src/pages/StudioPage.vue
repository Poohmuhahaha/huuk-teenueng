<script setup lang="ts">
// Client-facing Content Studio: list + editor for articles/pages/notes.
// Reachable by every account that has the `content.*` permissions (the seeded
// Client role), while staff reach it from the main nav.
import { computed, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useContentList, useCreateContent, usePermission } from '@/core/queries'
import { t } from '@/core/i18n'
import InfoTip from '@/components/ui/InfoTip.vue'
import DropdownMenu from '@/components/ui/DropdownMenu.vue'
import IconButton from '@/components/ui/IconButton.vue'
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

const search = ref('')
const debouncedSearch = ref('')
let searchTimer: number | undefined
watch(search, (value) => {
  window.clearTimeout(searchTimer)
  searchTimer = window.setTimeout(() => { debouncedSearch.value = value }, 250)
})
onBeforeUnmount(() => window.clearTimeout(searchTimer))

const status = ref<'' | ContentStatus>('')
const params = computed(() => ({
  status: status.value || undefined,
  q: debouncedSearch.value.trim() || undefined,
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

// The editor is a popup: picking an item opens it, closing drops the id from
// the URL. A refresh restores the selection but never pops the sheet open.
const editorOpen = ref(false)
let mounted = false
onMounted(() => { mounted = true })
watch(selectedId, (id) => { if (id && mounted) editorOpen.value = true })
watch(editorOpen, (open) => {
  if (open || !selectedId.value) return
  if (props.embedded) emit('update:id', '')
  else void router.replace('/studio')
})

function select(id: string): void {
  if (id === selectedId.value) {
    editorOpen.value = true
    return
  }
  if (props.embedded) emit('update:id', id)
  else void router.push(`/studio/${id}`)
}

async function newContent(): Promise<void> {
  if (!canWrite.value || create.isPending.value) return
  try {
    const created = await create.mutateAsync({ title: 'Untitled', kind: 'article' })
    if (props.embedded) emit('update:id', created.id)
    else void router.push(`/studio/${created.id}`)
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
    <h1>{{ t('studio.title') }}</h1>
    <p class="muted">{{ t('studio.signInHint') }}</p>
  </div>

  <div v-else ref="root" class="studio-page">
    <h1>{{ t('studio.title') }}<InfoTip :text="t('studio.subtitle')" /></h1>

    <div class="studio">
    <aside class="studio-side">
      <div class="studio-searchbar">
        <input class="field studio-search" v-model="search" type="search" :placeholder="t('studio.search')"
          :aria-label="t('studio.search')" />
        <IconButton tone="primary" :label="t('studio.new')" :disabled="!canWrite || create.isPending.value"
          @click="newContent">+</IconButton>
      </div>
      <div class="studio-filters">
        <DropdownMenu :items="statusMenuItems" @select="onStatus">
          <template #trigger>
            <button type="button" class="btn studio-filter-btn" :aria-label="t('studio.filter')">
              {{ currentStatusLabel }} <span aria-hidden="true">▾</span>
            </button>
          </template>
        </DropdownMenu>
      </div>
    </aside>

    <div class="studio-main">
      <div v-if="listQ.isPending.value" class="muted">{{ t('studio.loading') }}</div>
      <div v-else-if="listQ.isError.value" class="studio-error">
        <p class="muted">{{ listQ.error.value?.message }}</p>
        <button class="btn" @click="() => listQ.refetch()">{{ t('common.tryAgain') }}</button>
      </div>
      <p v-else-if="!rows.length" class="muted">{{ t('studio.empty') }}</p>
      <ul v-else class="studio-list">
        <li v-for="row in rows" :key="row.id">
          <button class="studio-item" :class="{ on: row.id === selectedId }" @click="select(row.id)">
            <span class="studio-item-top">
              <strong>{{ row.title || t('studio.untitled') }}</strong>
              <span class="statuschip" :class="`st-${row.status}`">{{ statusLabel(row.status) }}</span>
            </span>
            <span class="muted studio-excerpt">{{ row.excerpt || t('studio.noExcerpt') }}</span>
            <span class="studio-meta">
              {{ row.author }} · {{ updatedLabel(row) }} · {{ row.wordCount }} {{ t('studio.words') }}
              <template v-if="row.tags.length"> · {{ row.tags.join(', ') }}</template>
            </span>
          </button>
        </li>
      </ul>
    </div>

    <section v-if="deckFull || wide" class="studio-detail">
      <ContentEditor v-if="selectedId" :key="selectedId" :id="selectedId" />
      <template v-else>
        <p class="muted">{{ t('studio.pick') }}</p>
        <p class="muted">{{ t('studio.pickHint') }}</p>
      </template>
    </section>
    </div>

    <Drawer v-if="!deckFull" v-model="editorOpen" class="editor-sheet" side="bottom" contained
      :title="selectedRow?.title || t('studio.untitled')">
      <ContentEditor v-if="selectedId" :key="selectedId" :id="selectedId" />
    </Drawer>
  </div>
</template>

<style scoped>
.studio {
  display: grid;
  grid-template-columns: minmax(0, 1fr);
  gap: 16px;
  align-items: start;
}
/* Search + new: one row at an 80/20 split (gap aside). */
.studio-searchbar { display: flex; align-items: center; gap: 8px; }
.studio-searchbar .studio-search { flex: 4 1 0; min-width: 0; }
.studio-searchbar :deep(.iconbtn) {
  flex: 1 1 0;
  width: 100%;
  height: 38px;
  border-radius: 10px;
  font-size: 18px;
  line-height: 1;
  transition: transform 0.15s, background 0.15s, border-color 0.15s;
}
/* Hover contracts the + a touch. */
.studio-searchbar :deep(.iconbtn:hover:not(:disabled)) { transform: scale(0.94); }
/* No nested cards: the list sits directly on the slide card. */
.studio-side {
  padding: 0;
  display: grid;
  gap: 10px;
}
.studio-filters { display: block; margin: 0; }
/* Full-width filter toggle: trigger and menu match the column width. */
.studio-filters :deep(.p-menu) { width: 100%; }
.studio-filters :deep(.p-menu-trigger) { width: 100%; }
.studio-filters :deep(.p-menu-panel) { width: 100%; min-width: 0; }
.studio-filter-btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  width: 100%;
  height: 38px;
}
/* Redesigned search: white bar, ink border, magnifier, soft shadow. */
.studio-side .studio-search {
  background-color: var(--surface, #fff);
  border: 1.5px solid var(--ink, #0a0a0a);
  border-radius: 10px;
  height: 38px;
  padding: 0 14px 0 40px;
  background-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='16' height='16' viewBox='0 0 16 16' fill='none' stroke='%230a0a0a' stroke-width='1.6' stroke-linecap='round'%3E%3Ccircle cx='7' cy='7' r='4.5'/%3E%3Cpath d='M10.5 10.5 14 14'/%3E%3C/svg%3E");
  background-repeat: no-repeat;
  background-position: 13px center;
  background-size: 16px 16px;
}
.studio-side .studio-search:focus {
  border-color: var(--ink, #000);
  box-shadow: 0 0 0 3px rgba(0, 0, 0, 0.12);
}
.studio-side .studio-search::placeholder { color: var(--muted, #6b6b6b); }
.studio-list { list-style: none; margin: 0; padding: 0; display: grid; gap: 6px; }
.studio-item {
  width: 100%;
  text-align: center;
  justify-items: center;
  font: inherit;
  background: transparent;
  border: 0;
  border-bottom: 1px solid var(--faint);
  border-radius: 0;
  padding: 10px 4px;
  cursor: pointer;
  display: grid;
  gap: 3px;
}
.studio-item:hover { background: var(--wash); }
.studio-item.on { background: var(--wash); box-shadow: inset 2px 0 0 var(--ink); }
.studio-error { padding: 8px 0; display: grid; gap: 8px; justify-items: center; }
.studio-item-top { display: flex; justify-content: center; gap: 8px; align-items: baseline; }
.studio-excerpt { font-size: 12px; display: -webkit-box; -webkit-line-clamp: 2; -webkit-box-orient: vertical; overflow: hidden; }
.studio-meta { font-size: 11px; color: var(--muted); }

/* Editor sheet: slightly tighter chrome than the default drawer. */
.editor-sheet :deep(.p-drawer-head) { padding: 12px 16px; }
.editor-sheet :deep(.p-drawer-title) { font-size: 18px; }
.editor-sheet :deep(.p-drawer-body) { padding: 14px 18px; }
</style>
