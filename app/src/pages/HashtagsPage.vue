<script setup lang="ts">
import { computed, ref } from 'vue'
import { useTags, useAddTag, usePermission, useAllPosts, useLive } from '@/core/queries'
import { t } from '@/core/i18n'
import HashtagGroups from '@/components/ui/HashtagGroups.vue'
import DockBar from '@/components/ui/DockBar.vue'
import { useWidePane } from '@/composables/useWidePane'
import { useHashtagTracker } from '@/composables/useHashtagTracker'

const root = ref<HTMLElement | null>(null)
useWidePane(root, 'tags-scroll')

const { data: groups, isPending, isError, error: loadError, refetch } = useTags()
const { data: allPosts } = useAllPosts()
const { data: liveData } = useLive()

const addTag = useAddTag()
const { can } = usePermission()
const canWrite = computed(() => can('hashtags.write'))

// Dynamic tracker that extracts and tracks hashtags from all planned and live posts
const {
  trackedMap,
  trackedList,
  totalTrackedTags,
  totalPostsAnalyzed,
} = useHashtagTracker(allPosts, liveData)

// View switcher: 'categories' (Image 7 design) or 'tracked' (post discovery feed)
const viewMode = ref<'categories' | 'tracked'>('categories')

const copyNotice = ref('')

const totalTagsInGroups = computed(() => {
  return (groups.value ?? []).reduce((acc, g) => acc + g.tags.length, 0)
})

async function onAdd(groupId: string, tag: string): Promise<void> {
  if (!canWrite.value) throw new Error(t('auth.noPerm'))
  await addTag.mutateAsync({ groupId, tag })
}

async function copyAllBundled(): Promise<void> {
  // Collect all unique hashtags from both groups and tracked posts
  const tagsSet = new Set<string>()
  for (const g of groups.value ?? []) {
    for (const t of g.tags) tagsSet.add(`#${t.replace(/^#/, '')}`)
  }
  for (const t of trackedList.value) {
    tagsSet.add(`#${t.tag}`)
  }

  const allTags = Array.from(tagsSet)
  if (!allTags.length || typeof navigator === 'undefined' || !navigator.clipboard) return
  try {
    await navigator.clipboard.writeText(allTags.join(' '))
    copyNotice.value = 'all'
    setTimeout(() => { copyNotice.value = '' }, 2000)
  } catch {
    // ignore
  }
}

async function copySingle(tag: string): Promise<void> {
  if (typeof navigator === 'undefined' || !navigator.clipboard) return
  try {
    await navigator.clipboard.writeText(`#${tag.replace(/^#/, '')}`)
    copyNotice.value = tag
    setTimeout(() => {
      if (copyNotice.value === tag) copyNotice.value = ''
    }, 1800)
  } catch {
    // ignore
  }
}

async function quickAssignToGroup(groupId: string, tag: string): Promise<void> {
  if (!canWrite.value) return
  try {
    await onAdd(groupId, tag)
  } catch {
    // handled
  }
}
</script>

<template>
  <div ref="root" class="tags-page w-full flex-1 flex flex-col min-h-0 text-left">
    <div v-if="isPending" class="muted py-8 text-center">Loading hashtags…</div>
    <div v-else-if="isError" class="card py-8 text-center">
      <p class="muted mb-3">Could not load hashtag groups: {{ loadError?.message }}</p>
      <button class="btn" @click="() => refetch()">{{ t('common.tryAgain') }}</button>
    </div>

    <!-- VIEW 1: CATEGORIES STACK (Matching Reference Image 7) -->
    <div v-else-if="viewMode === 'categories'" class="w-full flex-1 overflow-y-auto px-1">
      <fieldset
        class="border-0 p-0 m-0 min-w-0 w-full"
        :disabled="!canWrite"
        :title="canWrite ? '' : t('auth.noPerm')"
      >
        <HashtagGroups
          :groups="groups ?? []"
          :add="onAdd"
          :tag-stats="trackedMap"
        />
      </fieldset>
    </div>

    <!-- VIEW 2: TRACKED FROM POSTS FEED -->
    <div v-else class="w-full flex-1 overflow-y-auto px-2 py-3 max-w-4xl mx-auto">
      <div class="mb-5 flex items-center justify-between">
        <div>
          <h3 class="text-base font-bold text-ink m-0 tracking-tight">Hashtags Tracked from Posts</h3>
          <p class="text-xs text-muted m-0 mt-0.5">
            Automatically extracted across {{ totalPostsAnalyzed }} planned and live posts
          </p>
        </div>
      </div>

      <!-- Table / Cards of Tracked Hashtags -->
      <div class="border border-line rounded-2xl bg-surface overflow-hidden shadow-xs">
        <div class="overflow-x-auto">
          <table class="w-full text-left text-xs border-collapse">
            <thead>
              <tr class="border-b border-line bg-wash/50 text-muted font-semibold uppercase tracking-wider text-[11px]">
                <th class="py-3 px-4">Hashtag</th>
                <th class="py-3 px-4">Usage</th>
                <th class="py-3 px-4">Posts</th>
                <th class="py-3 px-4">Category</th>
                <th class="py-3 px-4 text-right">Action</th>
              </tr>
            </thead>
            <tbody class="divide-y divide-line/60">
              <tr
                v-for="stat in trackedList"
                :key="stat.tag"
                class="hover:bg-wash/40 transition-colors"
              >
                <!-- Hashtag Name -->
                <td class="py-3.5 px-4 font-semibold text-ink">
                  <button
                    type="button"
                    class="inline-flex items-center gap-1.5 px-3 py-1 rounded-full border text-xs font-semibold cursor-pointer transition-all"
                    :class="[
                      copyNotice === stat.tag
                        ? 'bg-emerald-500 text-white border-emerald-500'
                        : 'bg-surface border-line hover:border-ink text-ink shadow-2xs'
                    ]"
                    :title="`Click to copy #${stat.tag}`"
                    @click="copySingle(stat.tag)"
                  >
                    <span>#{{ stat.tag }}</span>
                    <span v-if="copyNotice === stat.tag" class="text-[10px]">✓</span>
                  </button>
                </td>

                <!-- Usage count -->
                <td class="py-3.5 px-4 font-mono font-medium text-ink">
                  <span class="inline-flex items-center gap-1 px-2 py-0.5 rounded-full bg-wash border border-faint text-[11px]">
                    {{ stat.count }} {{ stat.count === 1 ? 'post' : 'posts' }}
                  </span>
                </td>

                <!-- Associated Posts -->
                <td class="py-3.5 px-4 text-muted max-w-[280px] truncate" :title="stat.posts.map(p => p.title).join(', ')">
                  <span v-for="(p, pi) in stat.posts.slice(0, 2)" :key="p.id">
                    <span class="text-ink font-medium">{{ p.title }}</span>
                    <span v-if="pi < Math.min(stat.posts.length, 2) - 1">, </span>
                  </span>
                  <span v-if="stat.posts.length > 2" class="text-[10px] text-muted ml-1">
                    +{{ stat.posts.length - 2 }} more
                  </span>
                </td>

                <!-- Category -->
                <td class="py-3.5 px-4">
                  <span
                    v-if="stat.group"
                    class="px-2.5 py-0.5 rounded-full text-[11px] font-semibold bg-wash border border-line text-ink"
                  >
                    {{ stat.group }}
                  </span>
                  <span v-else class="text-muted/60 italic">—</span>
                </td>

                <!-- Actions -->
                <td class="py-3.5 px-4 text-right">
                  <div class="inline-flex items-center gap-1.5">
                    <!-- Add to category dropdown/buttons -->
                    <select
                      v-if="canWrite"
                      class="px-2 py-1 rounded-full text-[11px] border border-line bg-surface text-ink cursor-pointer hover:border-ink focus:outline-none"
                      aria-label="Add to group"
                      @change="(e) => {
                        const target = e.target as HTMLSelectElement
                        if (target.value) {
                          void quickAssignToGroup(target.value, stat.tag)
                          target.value = ''
                        }
                      }"
                    >
                      <option value="">+ Add to Category</option>
                      <option v-for="g in groups ?? []" :key="g.id" :value="g.id">
                        {{ g.title }}
                      </option>
                    </select>

                    <button
                      type="button"
                      class="px-2.5 py-1 rounded-full border border-line bg-surface text-[11px] font-semibold hover:border-ink hover:bg-wash transition-all cursor-pointer"
                      @click="copySingle(stat.tag)"
                    >
                      {{ copyNotice === stat.tag ? 'Copied' : 'Copy' }}
                    </button>
                  </div>
                </td>
              </tr>

              <tr v-if="!trackedList.length">
                <td colspan="5" class="py-8 text-center text-muted italic">
                  No hashtags found in posts yet. As you plan or fetch posts, they will appear here.
                </td>
              </tr>
            </tbody>
          </table>
        </div>
      </div>
    </div>

    <!-- Standard Slide DockBar -->
    <DockBar label="Hashtags controls">
      <div class="flex items-center justify-between w-full gap-3">
        <div class="flex items-center gap-2">
          <!-- View switcher capsule -->
          <div class="inline-flex items-center gap-1 p-1 bg-wash border border-line rounded-full shadow-2xs">
            <button
              type="button"
              class="px-3.5 py-1 rounded-full text-xs font-semibold transition-all cursor-pointer select-none whitespace-nowrap"
              :class="viewMode === 'categories' ? '!bg-ink !text-white shadow-xs font-bold' : 'text-muted hover:text-ink hover:bg-surface/60'"
              @click="viewMode = 'categories'"
            >
              Categories ({{ groups?.length ?? 0 }})
            </button>
            <button
              type="button"
              class="px-3.5 py-1 rounded-full text-xs font-semibold transition-all cursor-pointer select-none whitespace-nowrap"
              :class="viewMode === 'tracked' ? '!bg-ink !text-white shadow-xs font-bold' : 'text-muted hover:text-ink hover:bg-surface/60'"
              @click="viewMode = 'tracked'"
            >
              Tracked Posts ({{ totalTrackedTags }})
            </button>
          </div>

          <span class="dock-sep" aria-hidden="true" />

          <span class="text-xs font-semibold text-muted tracking-wider uppercase font-mono">
            {{ totalTagsInGroups }} Group Tags · {{ totalPostsAnalyzed }} Posts Tracked
          </span>
        </div>

        <div class="flex items-center gap-2 ml-auto">
          <button
            type="button"
            class="px-4 py-2 rounded-full border border-line bg-surface text-xs font-semibold text-ink hover:border-ink hover:bg-wash transition-all cursor-pointer shadow-2xs"
            title="Copy all hashtags across all groups and tracked posts"
            @click="copyAllBundled"
          >
            {{ copyNotice === 'all' ? 'All Copied ✓' : 'Copy All #' }}
          </button>
        </div>
      </div>
    </DockBar>
  </div>
</template>
