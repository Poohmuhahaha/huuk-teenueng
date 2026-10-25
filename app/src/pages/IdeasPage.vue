<script setup lang="ts">
import { computed, ref } from 'vue'
import { useIdeas, useAddIdea, useToggleIdea, usePromoteIdea, usePermission } from '@/core/queries'
import type { Idea } from '@/mock/db'
import { t } from '@/core/i18n'
import { activeMonth, activeYear } from '@/core/navdate'
import MonthToggle from '@/components/ui/MonthToggle.vue'
import TableFrame from '@/components/tables/TableFrame.vue'
import DockBar from '@/components/ui/DockBar.vue'
import Drawer from '@/components/overlays/Drawer.vue'
import { deckFull } from '@/core/deck'
import { useWidePane } from '@/composables/useWidePane'

const root = ref<HTMLElement | null>(null)
const { wide } = useWidePane(root, 'ideas-scroll')
const isWide = computed(() => deckFull.value || wide.value)

const { data: ideas, isPending, isError, error: loadError, refetch } = useIdeas()
const isEmpty = computed(() => !ideas.value || ideas.value.length === 0)

const pageIndex = ref(0)
const pageSize = ref(10)
const totalIdeas = computed(() => ideas.value?.length ?? 0)
const totalPages = computed(() => Math.max(1, Math.ceil(totalIdeas.value / pageSize.value)))
const rangeStart = computed(() => totalIdeas.value === 0 ? 0 : pageIndex.value * pageSize.value + 1)
const rangeEnd = computed(() => Math.min(totalIdeas.value, (pageIndex.value + 1) * pageSize.value))
const visibleIdeas = computed(() => {
  const all = ideas.value ?? []
  return all.slice(pageIndex.value * pageSize.value, (pageIndex.value + 1) * pageSize.value)
})
const addIdea = useAddIdea()
const toggleIdea = useToggleIdea()
const promoteMonth = activeMonth
const promoteYear = activeYear
const promote = usePromoteIdea(promoteMonth, promoteYear)
const { can } = usePermission()
const canWrite = computed(() => can('ideas.write'))

const drawerOpen = ref(false)
const topic = ref('')
const format = ref('Short-video')
const idea = ref('')
const link = ref('')
const addError = ref('')
const actionError = ref('')
const notice = ref('')

function openNew(): void {
  topic.value = ''
  format.value = 'Short-video'
  idea.value = ''
  link.value = ''
  addError.value = ''
  drawerOpen.value = true
}

async function add(): Promise<void> {
  if (!canWrite.value || addIdea.isPending.value || !topic.value.trim()) return
  addError.value = ''
  try {
    await addIdea.mutateAsync({
      topic: topic.value.trim(),
      format: format.value.trim() || 'Short-video',
      idea: idea.value.trim(),
      link: link.value.trim(),
      done: false,
    })
    topic.value = ''
    format.value = 'Short-video'
    idea.value = ''
    link.value = ''
    drawerOpen.value = false
  } catch (e) {
    addError.value = e instanceof Error ? e.message : String(e)
  }
}

async function onToggle(it: Idea, event: Event): Promise<void> {
  if (toggleIdea.isPending.value) return
  actionError.value = ''
  try {
    await toggleIdea.mutateAsync(it.id)
  } catch (e) {
    // put the checkbox back to the authoritative value
    ;(event.target as HTMLInputElement).checked = it.done
    actionError.value = e instanceof Error ? e.message : String(e)
  }
}

async function onPromote(it: Idea): Promise<void> {
  if (promote.isPending.value) return
  actionError.value = ''
  notice.value = ''
  try {
    await promote.mutateAsync(it.id)
    notice.value = `Promoted “${it.topic}” to ${promoteMonth.value}/${promoteYear.value}.`
  } catch (e) {
    actionError.value = e instanceof Error ? e.message : String(e)
  }
}
</script>

<template>
  <div ref="root" class="ideas-page w-full flex-1 flex flex-col min-h-0 text-left">
    <!-- Notice / Alert notifications -->
    <div v-if="notice" class="m-3 px-3.5 py-2 rounded-lg bg-emerald-500/10 border border-emerald-500/20 text-emerald-700 text-xs font-medium flex items-center justify-between" role="status">
      <span>{{ notice }}</span>
      <button type="button" class="text-xs opacity-60 hover:opacity-100 cursor-pointer" @click="notice = ''">✕</button>
    </div>
    <div v-if="actionError" class="m-3 px-3.5 py-2 rounded-lg bg-rose-500/10 border border-rose-500/20 text-rose-700 text-xs font-medium flex items-center justify-between" role="alert">
      <span>{{ actionError }}</span>
      <button type="button" class="text-xs opacity-60 hover:opacity-100 cursor-pointer" @click="actionError = ''">✕</button>
    </div>

    <!-- Table content -->
    <div class="flex-1 min-h-0 overflow-y-auto">
      <div v-if="isPending" class="muted py-8 text-center">Loading ideas…</div>
      <div v-else-if="isError" class="card m-4 py-8 text-center" role="alert">
        <p class="muted mb-3">Could not load ideas: {{ loadError?.message }}</p>
        <button type="button" class="btn" @click="() => refetch()">{{ t('common.tryAgain') }}</button>
      </div>
      <div v-else-if="isEmpty" class="card m-4 py-12 text-center flex flex-col items-center justify-center gap-3">
        <p class="muted">No ideas captured yet. Capture your first idea to get started!</p>
        <button type="button" class="btn btn-primary" :disabled="!canWrite" @click="openNew">
          + New Idea
        </button>
      </div>
      <TableFrame v-else class="h-full">
        <thead>
          <tr>
            <th class="w-[30%]">Topic</th>
            <th class="w-[15%]">Format</th>
            <th class="w-[35%]">Idea</th>
            <th class="w-[10%]">Link</th>
            <th class="w-[48px] text-center">Done</th>
            <th class="w-[100px] text-right"></th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="it in visibleIdeas" :key="it.id" class="hover:bg-wash/60 transition-colors">
            <td>
              <strong class="font-semibold text-ink text-sm">{{ it.topic }}</strong>
            </td>
            <td>
              <span class="inline-flex px-2 py-0.5 rounded text-2xs font-semibold bg-wash text-muted border border-faint">
                {{ it.format || 'General' }}
              </span>
            </td>
            <td>
              <span class="text-sm text-ink/80 line-clamp-2">{{ it.idea || '—' }}</span>
            </td>
            <td>
              <a v-if="it.link && (it.link.startsWith('http://') || it.link.startsWith('https://'))"
                :href="it.link" target="_blank" rel="noopener noreferrer"
                class="text-xs text-ink underline truncate block max-w-[120px]" :title="it.link">
                Link ↗
              </a>
              <span v-else class="muted text-xs">{{ it.link || '—' }}</span>
            </td>
            <td class="text-center">
              <input
                type="checkbox"
                :checked="it.done"
                :disabled="!canWrite || toggleIdea.isPending.value"
                :aria-label="`Mark ${it.topic} done`"
                :title="canWrite ? '' : t('auth.noPerm')"
                class="cursor-pointer"
                @change="onToggle(it, $event)"
              />
            </td>
            <td class="text-right">
              <button
                type="button"
                class="px-3 py-1 rounded-full text-xs font-semibold border border-line bg-surface hover:bg-ink hover:text-white hover:border-ink transition-all cursor-pointer shadow-2xs"
                :disabled="!canWrite || promote.isPending.value"
                :title="canWrite ? '' : t('auth.noPerm')"
                @click="onPromote(it)"
              >
                Promote
              </button>
            </td>
          </tr>
        </tbody>
      </TableFrame>

      <!-- Pagination bar -->
      <div
        v-if="totalIdeas > 0"
        class="flex flex-wrap items-center justify-between gap-3 px-4 py-2.5 mt-2 bg-surface rounded-xl border border-line/60 text-xs text-muted"
      >
        <span>Showing <strong class="font-semibold text-ink">{{ rangeStart }}–{{ rangeEnd }}</strong> of <strong class="font-semibold text-ink">{{ totalIdeas }}</strong> ideas</span>
        <div v-if="totalPages > 1" class="flex items-center gap-1.5 ml-auto">
          <button
            type="button"
            class="px-2.5 py-1 rounded-lg border border-line bg-surface hover:bg-wash disabled:opacity-40 disabled:cursor-not-allowed font-medium text-xs text-ink transition-colors cursor-pointer"
            :disabled="pageIndex === 0"
            aria-label="Previous page"
            @click="pageIndex--"
          >
            ‹ Prev
          </button>
          <span class="font-mono text-xs px-1 text-ink">
            {{ pageIndex + 1 }} / {{ totalPages }}
          </span>
          <button
            type="button"
            class="px-2.5 py-1 rounded-lg border border-line bg-surface hover:bg-wash disabled:opacity-40 disabled:cursor-not-allowed font-medium text-xs text-ink transition-colors cursor-pointer"
            :disabled="pageIndex >= totalPages - 1"
            aria-label="Next page"
            @click="pageIndex++"
          >
            Next ›
          </button>
        </div>
      </div>
    </div>

    <!-- Standard Slide DockBar -->
    <DockBar label="Ideas controls">
      <div class="flex items-center justify-between w-full gap-3">
        <div class="flex items-center gap-2">
          <span class="text-xs font-semibold text-muted tracking-wider uppercase font-mono">
            {{ ideas?.length ?? 0 }} Ideas
          </span>
        </div>
        <div class="flex items-center gap-2 ml-auto">
          <button
            type="button"
            class="btn btn-primary btn-icon w-auto"
            :disabled="!canWrite"
            :title="canWrite ? 'Add new idea' : t('auth.noPerm')"
            aria-label="Add new idea"
            @click="openNew"
          >
            +
          </button>
        </div>
      </div>
    </DockBar>

    <!-- Slide-over Drawer for Capturing New Idea -->
    <Drawer
      v-model="drawerOpen"
      class="editor-sheet"
      :inline="false"
      :side="isWide ? 'right' : 'bottom'"
      :width="'460px'"
      contained
      title="Capture New Idea"
    >
      <div class="p-6 flex flex-col gap-4 text-left">
        <div>
          <label class="lbl block text-xs font-semibold mb-1" for="idea-topic">Topic *</label>
          <input
            id="idea-topic"
            class="field w-full"
            v-model="topic"
            placeholder="e.g. Rainy Day Reads"
            :disabled="!canWrite"
            @keydown.enter="add"
          />
        </div>

        <div class="grid grid-cols-2 gap-3">
          <div>
            <label class="lbl block text-xs font-semibold mb-1" for="idea-format">Format</label>
            <input
              id="idea-format"
              class="field w-full"
              v-model="format"
              placeholder="e.g. Short-video"
              :disabled="!canWrite"
            />
          </div>
          <div>
            <label class="lbl block text-xs font-semibold mb-1">Promote to Month</label>
            <MonthToggle v-model="promoteMonth" v-model:year="promoteYear" :disabled="!canWrite" />
          </div>
        </div>

        <div>
          <label class="lbl block text-xs font-semibold mb-1" for="idea-idea">Idea / Angle</label>
          <textarea
            id="idea-idea"
            class="field w-full"
            rows="3"
            v-model="idea"
            placeholder="Cozy reading vlog, lighting setup, 3 key takeaways..."
            :disabled="!canWrite"
          />
        </div>

        <div>
          <label class="lbl block text-xs font-semibold mb-1" for="idea-link">Reference Link</label>
          <input
            id="idea-link"
            class="field w-full"
            v-model="link"
            placeholder="https://instagram.com/p/... or reference URL"
            :disabled="!canWrite"
          />
        </div>

        <p v-if="addError" class="autherr text-xs text-rose-600" role="alert">{{ addError }}</p>

        <div class="flex items-center justify-end gap-2.5 pt-3 border-t border-line mt-2">
          <button type="button" class="btn text-xs py-2 px-4 cursor-pointer" @click="drawerOpen = false">
            Cancel
          </button>
          <button
            type="button"
            class="btn btn-primary text-xs py-2 px-5 font-semibold cursor-pointer shadow-xs"
            :disabled="!canWrite || addIdea.isPending.value || !topic.trim()"
            @click="add"
          >
            {{ addIdea.isPending.value ? 'Saving…' : 'Save Idea' }}
          </button>
        </div>
      </div>
    </Drawer>
  </div>
</template>
