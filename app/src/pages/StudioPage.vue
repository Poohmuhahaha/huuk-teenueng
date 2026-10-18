<script setup lang="ts">
// Client-facing Content Studio: list + editor for articles/pages/notes.
// Reachable by every account that has the `content.*` permissions (the seeded
// Client role), while staff reach it from the main nav.
import { computed, onMounted, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useContentList, useCreateContent, usePermission } from '@/core/queries'
import { t } from '@/core/i18n'
import InfoTip from '@/components/ui/InfoTip.vue'
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

    <div class="grid grid-single gap-4 items-start">
    <DockBar label="Studio controls">
      <div class="flex-1 min-w-0 m-0 [&>div]:w-full [&>div>button]:w-full">
        <DropdownMenu :items="statusMenuItems" @select="onStatus">
          <template #trigger>
            <span class="btn inline-flex items-center justify-center gap-2 w-full h-touch" :aria-label="t('studio.filter')">
              {{ currentStatusLabel }} <span aria-hidden="true">▾</span>
            </span>
          </template>
        </DropdownMenu>
      </div>
      <button type="button" class="btn btn-primary btn-icon" :title="t('studio.new')" :aria-label="t('studio.new')"
        :disabled="!canWrite || create.isPending.value" @click="newContent">
        {{ create.isPending.value ? '…' : '+' }}
      </button>
    </DockBar>

    <div class="studio-main">
      <div v-if="listQ.isPending.value" class="muted">{{ t('studio.loading') }}</div>
      <div v-else-if="listQ.isError.value" class="py-2 grid gap-2 justify-items-center">
        <p class="muted">{{ listQ.error.value?.message }}</p>
        <button class="btn" @click="() => listQ.refetch()">{{ t('common.tryAgain') }}</button>
      </div>
      <p v-else-if="!rows.length" class="muted">{{ t('studio.empty') }}</p>
      <ul v-else class="list-none m-0 p-0 grid gap-1.5">
        <li v-for="row in rows" :key="row.id">
          <button class="w-full text-center justify-items-center bg-transparent border-0 border-b border-faint rounded-none py-2.5 px-1 cursor-pointer grid gap-tight hover:bg-wash transition"
            :class="row.id === selectedId ? 'bg-wash shadow-selected-bar' : ''"
            @click="select(row.id)">
            <span class="flex justify-center gap-2 items-baseline">
              <strong>{{ row.title || t('studio.untitled') }}</strong>
              <span class="statuschip" :class="`st-${row.status}`">{{ statusLabel(row.status) }}</span>
            </span>
            <span class="muted text-xs line-clamp-2">{{ row.excerpt || t('studio.noExcerpt') }}</span>
            <span class="text-2xs text-muted">
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

    <Drawer v-if="!deckFull" v-model="editorOpen" side="bottom" contained
      :title="selectedRow?.title || t('studio.untitled')">
      <ContentEditor v-if="selectedId" :key="selectedId" :id="selectedId" />
    </Drawer>
  </div>
</template>
