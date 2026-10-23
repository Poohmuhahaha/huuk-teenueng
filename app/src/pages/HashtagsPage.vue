<script setup lang="ts">
import { computed, ref } from 'vue'
import { useTags, useAddTag, usePermission } from '@/core/queries'
import { t } from '@/core/i18n'
import HashtagGroups from '@/components/ui/HashtagGroups.vue'
import DockBar from '@/components/ui/DockBar.vue'
import { useWidePane } from '@/composables/useWidePane'

const root = ref<HTMLElement | null>(null)
useWidePane(root, 'tags-scroll')

const { data: groups, isPending, isError, error: loadError, refetch } = useTags()
const addTag = useAddTag()
const { can } = usePermission()
const canWrite = computed(() => can('hashtags.write'))

const copyAllNotice = ref(false)

const totalTags = computed(() => {
  return (groups.value ?? []).reduce((acc, g) => acc + g.tags.length, 0)
})

async function onAdd(groupId: string, tag: string): Promise<void> {
  if (!canWrite.value) throw new Error(t('auth.noPerm'))
  await addTag.mutateAsync({ groupId, tag })
}

async function copyAllBundled(): Promise<void> {
  const allTags = (groups.value ?? []).flatMap((g) => g.tags.map((t) => `#${t.replace(/^#/, '')}`))
  if (!allTags.length || typeof navigator === 'undefined' || !navigator.clipboard) return
  try {
    await navigator.clipboard.writeText(allTags.join(' '))
    copyAllNotice.value = true
    setTimeout(() => { copyAllNotice.value = false }, 2000)
  } catch {
    // ignore
  }
}
</script>

<template>
  <div ref="root" class="tags-page w-full flex-1 flex flex-col min-h-0 text-left">
    <div v-if="isPending" class="muted py-8 text-center">Loading hashtag groups…</div>
    <div v-else-if="isError" class="card py-8 text-center">
      <p class="muted mb-3">Could not load hashtag groups: {{ loadError?.message }}</p>
      <button class="btn" @click="() => refetch()">{{ t('common.tryAgain') }}</button>
    </div>
    <fieldset v-else class="border-0 p-0 m-0 min-w-0 flex-1 flex flex-col min-h-0" :disabled="!canWrite" :title="canWrite ? '' : t('auth.noPerm')">
      <HashtagGroups :groups="groups ?? []" :add="onAdd" />
    </fieldset>

    <!-- Standard Slide DockBar -->
    <DockBar label="Hashtags controls">
      <div class="flex items-center justify-between w-full gap-3">
        <div class="flex items-center gap-2">
          <span class="text-xs font-semibold text-muted tracking-wider uppercase font-mono">
            {{ totalTags }} Tags · {{ groups?.length ?? 0 }} Categories
          </span>
        </div>
        <div class="flex items-center gap-2 ml-auto">
          <button
            type="button"
            class="px-4 py-2 rounded-full border border-line bg-surface text-xs font-semibold text-ink hover:border-ink hover:bg-wash transition-all cursor-pointer shadow-2xs"
            title="Copy all hashtags across all groups"
            @click="copyAllBundled"
          >
            {{ copyAllNotice ? 'All Copied ✓' : 'Copy All #' }}
          </button>
        </div>
      </div>
    </DockBar>
  </div>
</template>
