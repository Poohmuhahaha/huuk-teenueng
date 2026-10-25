<script setup lang="ts">
// Client-facing Content Studio: list + editor for articles/pages/notes.
// Reachable by every account that has the `content.*` permissions (the seeded
// Client role), while staff reach it from the main nav.
import { computed, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useContentList, useCreateContent, usePermission } from '@/core/queries'
import { t } from '@/core/i18n'
import DockBar from '@/components/ui/DockBar.vue'
import DropdownMenu from '@/components/ui/DropdownMenu.vue'
import type { ContentStatus, ContentSummary } from '@/mock/db'
import ContentEditor from '@/components/editor/ContentEditor.vue'
import Drawer from '@/components/overlays/Drawer.vue'
import { useWidePane } from '@/composables/useWidePane'

const props = defineProps<{ id?: string; embedded?: boolean }>()
const emit = defineEmits<{ 'update:id': [string] }>()
const route = useRoute()
const router = useRouter()
const { can, isLoggedIn } = usePermission()

const STATUSES: ContentStatus[] = ['draft', 'review', 'scheduled', 'published', 'archived']
const canWrite = computed(() => can('content.write'))

const status = ref<'' | ContentStatus>('')
const searchQuery = ref('')
const params = computed(() => ({
  status: status.value || undefined,
}))
const listQ = useContentList(params)
const create = useCreateContent()

const root = ref<HTMLElement | null>(null)
useWidePane(root, 'studio-scroll')

const isMobile = ref(typeof window !== 'undefined' ? window.innerWidth < 768 : false)
function updateMobile(): void {
  isMobile.value = window.innerWidth < 768
}
onMounted(() => {
  window.addEventListener('resize', updateMobile)
})
onBeforeUnmount(() => {
  window.removeEventListener('resize', updateMobile)
})

const routeId = computed(() => (typeof route.params.id === 'string' ? route.params.id : ''))
const activeId = computed(() => props.id ?? routeId.value)
const rows = computed<ContentSummary[]>(() => listQ.data.value ?? [])
const selectedRow = computed(() => rows.value.find((r) => r.id === activeId.value) ?? null)

const filteredRows = computed(() => {
  let list = rows.value
  const q = searchQuery.value.trim().toLowerCase()
  if (q) {
    list = list.filter(
      (r) =>
        r.title.toLowerCase().includes(q) ||
        r.excerpt?.toLowerCase().includes(q) ||
        r.tags.some((tag) => tag.toLowerCase().includes(q)) ||
        r.author.toLowerCase().includes(q),
    )
  }
  return list
})

const editorOpen = ref(false)

watch(
  activeId,
  (id) => {
    if (id) {
      editorOpen.value = true
    }
  },
  { immediate: true },
)

watch(editorOpen, (open) => {
  if (open) return
  if (props.embedded) {
    emit('update:id', '')
  } else if (route.params.id) {
    void router.replace('/studio')
  }
})

function select(id: string): void {
  if (props.embedded) emit('update:id', id)
  else void router.push(`/studio/${id}`)
  editorOpen.value = true
}

async function newContent(): Promise<void> {
  if (!canWrite.value || create.isPending.value) return
  try {
    const created = await create.mutateAsync({ title: 'Untitled', kind: 'article' })
    if (props.embedded) emit('update:id', created.id)
    else void router.push(`/studio/${created.id}`)
    editorOpen.value = true
  } catch {
    // handled
  }
}

function statusLabel(s: ContentStatus): string {
  return t(`studio.status.${s}`)
}

function statusBadgeClass(s: ContentStatus): string {
  switch (s) {
    case 'published':
      return 'bg-emerald-500/15 text-emerald-700 dark:text-emerald-400 border border-emerald-500/30'
    case 'scheduled':
      return 'bg-blue-500/15 text-blue-700 dark:text-blue-400 border border-blue-500/30'
    case 'review':
      return 'bg-amber-500/15 text-amber-700 dark:text-amber-400 border border-amber-500/30'
    case 'draft':
      return 'bg-wash text-ink-soft border border-line'
    case 'archived':
      return 'bg-wash/60 text-muted border border-line/50'
    default:
      return 'bg-wash text-muted border border-line'
  }
}

function authorInitial(author: string): string {
  return (author || 'U').charAt(0).toUpperCase()
}

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
      <div class="flex items-center justify-between w-full gap-3 flex-wrap sm:flex-nowrap">
        <div class="flex items-center gap-2 flex-wrap">
          <DropdownMenu :items="statusMenuItems" @select="onStatus">
            <template #trigger>
              <button type="button" class="btn w-auto inline-flex items-center gap-2" :aria-label="t('studio.filter')">
                <span>{{ currentStatusLabel }}</span> <span aria-hidden="true">▾</span>
              </button>
            </template>
          </DropdownMenu>

          <!-- Search Input -->
          <div class="relative w-40 sm:w-52">
            <input
              v-model="searchQuery"
              type="search"
              placeholder="Search pieces..."
              class="w-full text-xs px-3 py-1.5 pl-8 rounded-full border border-line bg-surface focus:outline-none focus:border-ink transition-colors"
            />
            <svg
              class="w-3.5 h-3.5 text-muted absolute left-2.5 top-1/2 -translate-y-1/2 pointer-events-none"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="2"
            >
              <circle cx="11" cy="11" r="8" />
              <line x1="21" y1="21" x2="16.65" y2="16.65" />
            </svg>
          </div>

          <span class="text-xs text-muted font-medium ml-1">
            {{ filteredRows.length }} {{ filteredRows.length === 1 ? 'item' : 'items' }}
          </span>
        </div>

        <div class="flex items-center gap-2 ml-auto">
          <button
            type="button"
            class="btn btn-primary w-auto inline-flex items-center gap-1.5 px-3 py-1.5 text-xs font-semibold"
            :title="t('studio.new')"
            :aria-label="t('studio.new')"
            :disabled="!canWrite || create.isPending.value"
            @click="newContent"
          >
            <span>+</span>
            <span class="hidden sm:inline">New piece</span>
          </button>
        </div>
      </div>
    </DockBar>

    <!-- Main Content Area: Full width Cards Grid -->
    <div class="flex-1 min-h-0 overflow-y-auto mt-3 pr-1">
      <!-- Loading State -->
      <div v-if="listQ.isPending.value" class="muted p-12 text-center">
        {{ t('studio.loading') }}
      </div>

      <!-- Error State -->
      <div v-else-if="listQ.isError.value" class="py-12 grid gap-3 justify-items-center text-center">
        <p class="muted">{{ listQ.error.value?.message }}</p>
        <button class="btn w-auto text-xs" @click="() => listQ.refetch()">{{ t('common.tryAgain') }}</button>
      </div>

      <!-- Empty State -->
      <div
        v-else-if="!filteredRows.length"
        class="p-12 text-center flex flex-col items-center justify-center gap-3 rounded-2xl border border-line bg-surface/50 my-4"
      >
        <div class="w-12 h-12 rounded-2xl bg-wash flex items-center justify-center text-muted">
          <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6">
            <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z" />
            <polyline points="14 2 14 8 20 8" />
            <line x1="16" y1="13" x2="8" y2="13" />
            <line x1="16" y1="17" x2="8" y2="17" />
          </svg>
        </div>
        <div>
          <p class="font-bold text-ink text-sm m-0">No pieces found</p>
          <p class="text-xs text-muted mt-1 max-w-xs">
            {{ searchQuery ? 'Try a different search keyword or status filter.' : t('studio.empty') }}
          </p>
        </div>
        <button
          v-if="!searchQuery && canWrite"
          type="button"
          class="btn btn-primary text-xs w-auto px-4 py-2 mt-2"
          @click="newContent"
        >
          + {{ t('studio.new') }}
        </button>
      </div>

      <!-- Cards Grid -->
      <div v-else class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-3.5 sm:gap-4 pb-6">
        <article
          v-for="row in filteredRows"
          :key="row.id"
          class="card-fade group flex flex-col justify-between p-4 sm:p-5 rounded-2xl border border-line bg-surface hover:border-ink/60 hover:shadow-md transition-all duration-200 cursor-pointer text-left relative focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-accent"
          tabindex="0"
          role="button"
          :aria-label="row.title || t('studio.untitled')"
          @click="select(row.id)"
          @keydown.enter.prevent="select(row.id)"
          @keydown.space.prevent="select(row.id)"
        >
          <div>
            <!-- Status Badge & Reading Time -->
            <div class="flex items-center justify-between gap-2 mb-2.5">
              <span
                class="inline-flex items-center px-2 py-0.5 rounded-full text-[10.5px] font-bold uppercase tracking-wider flex-none"
                :class="statusBadgeClass(row.status)"
              >
                {{ statusLabel(row.status) }}
              </span>
              <span class="text-2xs text-muted font-medium flex items-center gap-1">
                <span>{{ row.wordCount }} {{ t('studio.words') }}</span>
                <span>·</span>
                <span>~{{ Math.max(1, Math.round(row.wordCount / 200)) }} min</span>
              </span>
            </div>

            <!-- Title -->
            <h3 class="text-[15px] font-bold text-ink leading-snug line-clamp-2 m-0 group-hover:text-accent transition-colors">
              {{ row.title || t('studio.untitled') }}
            </h3>

            <!-- Excerpt -->
            <p class="text-xs text-muted leading-relaxed line-clamp-2 mt-2 mb-3">
              {{ row.excerpt || t('studio.noExcerpt') }}
            </p>

            <!-- Tags -->
            <div v-if="row.tags.length" class="flex flex-wrap gap-1.5 mb-4">
              <span
                v-for="tag in row.tags.slice(0, 3)"
                :key="tag"
                class="inline-flex items-center px-2 py-0.5 rounded-md bg-wash text-muted text-2xs font-medium border border-line/60"
              >
                #{{ tag }}
              </span>
              <span v-if="row.tags.length > 3" class="text-2xs text-muted/80 self-center">
                +{{ row.tags.length - 3 }}
              </span>
            </div>
          </div>

          <!-- Footer Row: Author + Date + Edit Affordance -->
          <div class="pt-3 border-t border-line/60 flex items-center justify-between gap-2 text-xs text-muted">
            <div class="flex items-center gap-2 min-w-0">
              <div class="w-5 h-5 rounded-full bg-wash border border-line flex items-center justify-center text-2xs font-bold text-ink flex-none">
                {{ authorInitial(row.author) }}
              </div>
              <span class="truncate text-[11.5px] font-medium text-ink-soft">{{ row.author }}</span>
              <span>·</span>
              <span class="text-[11px] whitespace-nowrap">{{ updatedLabel(row) }}</span>
            </div>

            <span class="inline-flex items-center gap-1 text-[11.5px] font-semibold text-muted group-hover:text-ink transition-colors flex-none">
              <span>Edit</span>
              <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round" class="transition-transform group-hover:translate-x-0.5">
                <path d="M5 12h14M12 5l7 7-7 7" />
              </svg>
            </span>
          </div>
        </article>
      </div>
    </div>

    <!-- Responsive Content Editor Sheet (Bottom Sheet on mobile, Right Drawer on desktop) -->
    <Drawer
      v-model="editorOpen"
      :side="isMobile ? 'bottom' : 'right'"
      :width="'min(92vw, 780px)'"
      :title="selectedRow?.title || t('studio.untitled')"
    >
      <div v-if="activeId" class="p-1 sm:p-2">
        <ContentEditor :key="activeId" :id="activeId" />
      </div>
      <div v-else class="p-8 text-center muted">
        {{ t('studio.loading') }}
      </div>
    </Drawer>
  </div>
</template>
