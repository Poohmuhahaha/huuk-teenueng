<script setup lang="ts">
import { ref } from 'vue'
import type { HashtagGroup } from '@/mock/db'
import type { TrackedTag } from '@/composables/useHashtagTracker'

const props = defineProps<{
  groups: HashtagGroup[]
  /** Async add handler; the draft clears only when it resolves. */
  add?: (groupId: string, tag: string) => Promise<void>
  /** Tracked stats per tag (used for tooltips) */
  tagStats?: Map<string, TrackedTag>
}>()

const drafts = ref<Record<string, string>>({})
const copiedKey = ref<string>('')
const error = ref('')
const busy = ref(false)

async function copyText(key: string, text: string): Promise<void> {
  if (typeof navigator !== 'undefined' && navigator.clipboard) {
    try {
      await navigator.clipboard.writeText(text)
      copiedKey.value = key
      setTimeout(() => {
        if (copiedKey.value === key) copiedKey.value = ''
      }, 1800)
    } catch {
      // ignore clipboard error
    }
  }
}

function copySingleTag(gId: string, t: string): void {
  const tag = `#${t.replace(/^#/, '')}`
  void copyText(`${gId}-${t}`, tag)
}

function getTagTooltip(tag: string): string {
  const normalized = tag.trim().toLowerCase().replace(/^#/, '')
  const stat = props.tagStats?.get(normalized)
  if (!stat || stat.count === 0) return `Click to copy #${tag}`
  const titles = stat.posts.map((p) => p.title).join(', ')
  return `Click to copy #${tag} · Tracked in ${stat.count} ${stat.count === 1 ? 'post' : 'posts'} (${titles})`
}

async function submit(groupId: string): Promise<void> {
  const value = (drafts.value[groupId] ?? '').trim().replace(/^#/, '')
  if (!value || !props.add || busy.value) return
  error.value = ''
  busy.value = true
  try {
    await props.add(groupId, value)
    drafts.value[groupId] = ''
  } catch (e) {
    error.value = e instanceof Error ? e.message : String(e)
  } finally {
    busy.value = false
  }
}
</script>

<template>
  <!-- Sleek, tight 2x2 Grid without excessive gap or bloated padding -->
  <div class="hashtag-grid grid grid-cols-1 md:grid-cols-2 gap-2.5 w-full flex-1 min-h-0 overflow-y-auto">
    <div
      v-for="g in groups"
      :key="g.id"
      class="border border-line rounded-xl bg-surface p-4 sm:p-4.5 shadow-2xs flex flex-col justify-between transition-all hover:border-ink/25"
    >
      <!-- Title: Centered, Title Case, clean sans-serif font matching Image 7 -->
      <div class="!font-sans font-bold text-sm text-ink text-center !normal-case tracking-tight select-none flex-none mb-1">
        {{ g.title }}
      </div>

      <!-- Tag Pills: Centered, compact gap and padding -->
      <div class="flex items-center justify-center flex-wrap gap-1.5 py-2 w-full flex-1 content-center">
        <button
          v-for="(t, i) in g.tags"
          :key="`${t}-${i}`"
          type="button"
          class="inline-flex items-center gap-1.5 px-3 py-1 rounded-full text-xs font-medium border transition-all cursor-pointer select-none group"
          :class="[
            copiedKey === `${g.id}-${t}`
              ? '!bg-emerald-500 !text-white !border-emerald-500 shadow-xs'
              : 'border-line bg-surface hover:border-ink hover:bg-wash text-ink shadow-2xs'
          ]"
          :title="getTagTooltip(t)"
          @click="copySingleTag(g.id, t)"
        >
          <span>#{{ t }}</span>
          <span v-if="copiedKey === `${g.id}-${t}`" class="text-[10px]">✓</span>
        </button>

        <span v-if="!g.tags.length" class="text-xs text-muted/60 italic py-1">
          No tags in this category yet.
        </span>
      </div>

      <!-- Underline input + ADD button matching Image 7 with single clean line -->
      <form
        class="w-full flex items-center gap-2.5 pt-2 flex-none"
        @submit.prevent="submit(g.id)"
      >
        <div class="relative flex-1 flex items-center">
          <input
            :value="drafts[g.id] ?? ''"
            type="text"
            placeholder="new tag"
            :aria-label="`New tag for ${g.title}`"
            :disabled="!add || busy"
            class="tag-input w-full bg-transparent p-0 text-xs text-ink placeholder:text-muted/60 focus:outline-none"
            @input="drafts[g.id] = ($event.target as HTMLInputElement).value"
          />
        </div>
        <button
          type="submit"
          class="px-4 py-0.5 rounded-full border border-ink bg-surface text-[11px] font-bold uppercase tracking-wider text-ink hover:bg-ink hover:text-white active:scale-95 transition-all cursor-pointer whitespace-nowrap disabled:opacity-40 disabled:cursor-not-allowed shadow-2xs"
          :disabled="!add || busy || !(drafts[g.id] ?? '').trim()"
        >
          {{ busy ? '…' : 'ADD' }}
        </button>
      </form>
    </div>
  </div>

  <p v-if="error" class="autherr mt-1.5 text-center flex-none" role="alert">{{ error }}</p>
</template>

<style scoped>
.tag-input {
  border: 0 !important;
  border-bottom: 1.2px solid var(--ink) !important;
  border-radius: 0 !important;
  padding: 5px 2px !important;
  background: transparent !important;
}
.tag-input:focus {
  outline: none !important;
  box-shadow: none !important;
  border-bottom-color: var(--ink) !important;
}
</style>
