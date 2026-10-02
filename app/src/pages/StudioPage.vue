<script setup lang="ts">
// Client-facing Content Studio: list + editor for articles/pages/notes.
// Reachable by every account that has the `content.*` permissions (the seeded
// Client role), while staff reach it from the main nav.
import { computed, onBeforeUnmount, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useContentList, useCreateContent, usePermission } from '@/core/queries'
import { t } from '@/core/i18n'
import InfoTip from '@/components/ui/InfoTip.vue'
import type { ContentStatus, ContentSummary } from '@/mock/db'
import ContentEditor from '@/components/editor/ContentEditor.vue'

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

const selectedId = computed(() =>
  props.id ?? (typeof route.params.id === 'string' ? route.params.id : ''),
)
const rows = computed<ContentSummary[]>(() => listQ.data.value ?? [])

function select(id: string): void {
  if (id === selectedId.value) return
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

  <div v-else class="studio">
    <aside class="studio-side">
      <div class="studio-side-head">
        <div>
          <h1>{{ t('studio.title') }}<InfoTip :text="t('studio.subtitle')" /></h1>
        </div>
        <button class="btn btn-primary" :disabled="!canWrite || create.isPending.value"
          :title="canWrite ? '' : t('auth.noPerm')" @click="newContent">
          + {{ t('studio.new') }}
        </button>
      </div>

      <input class="field" v-model="search" type="search" :placeholder="t('studio.search')"
        :aria-label="t('studio.search')" />
      <div class="studio-filters" role="tablist" :aria-label="t('studio.filter')">
        <button class="tab" :class="{ active: status === '' }" role="tab" :aria-selected="status === ''"
          @click="status = ''">{{ t('studio.all') }}</button>
        <button v-for="s in STATUSES" :key="s" class="tab" :class="{ active: status === s }" role="tab"
          :aria-selected="status === s" @click="status = s">{{ statusLabel(s) }}</button>
      </div>

      <div v-if="listQ.isPending.value" class="muted">{{ t('studio.loading') }}</div>
      <div v-else-if="listQ.isError.value" class="card">
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
    </aside>

    <main class="studio-main">
      <ContentEditor v-if="selectedId" :key="selectedId" :id="selectedId" />
      <div v-else class="card mt">
        <h2 style="margin-top: 0;">{{ t('studio.pick') }}<InfoTip :text="t('studio.pickHint')" /></h2>
      </div>
    </main>
  </div>
</template>

<style scoped>
.studio {
  display: grid;
  grid-template-columns: minmax(260px, 340px) 1fr;
  gap: 16px;
  align-items: start;
}
.studio-side {
  border: 1.5px solid var(--line);
  border-radius: var(--radius);
  padding: 12px;
  position: sticky;
  top: 12px;
  max-height: calc(100vh - 120px);
  overflow-y: auto;
}
.studio-side-head {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  gap: 8px;
  margin-bottom: 10px;
}
.studio-side h1 { font-size: 20px; margin: 0; }
.studio-filters { display: flex; flex-wrap: wrap; gap: 6px; margin: 8px 0; }
.studio-list { list-style: none; margin: 0; padding: 0; display: grid; gap: 6px; }
.studio-item {
  width: 100%;
  text-align: left;
  font: inherit;
  background: var(--surface);
  border: 1px solid var(--faint);
  border-radius: var(--radius-sm);
  padding: 8px 10px;
  cursor: pointer;
  display: grid;
  gap: 3px;
}
.studio-item:hover { background: var(--wash); }
.studio-item.on { border-color: var(--ink); box-shadow: 0 0 0 1px var(--ink) inset; }
.studio-item-top { display: flex; justify-content: space-between; gap: 8px; align-items: baseline; }
.studio-excerpt { font-size: 12px; display: -webkit-box; -webkit-line-clamp: 2; -webkit-box-orient: vertical; overflow: hidden; }
.studio-meta { font-size: 11px; color: var(--muted); }
@media (max-width: 900px) {
  .studio { grid-template-columns: 1fr; }
  .studio-side { position: static; max-height: none; }
}
</style>
