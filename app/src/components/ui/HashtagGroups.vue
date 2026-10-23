<script setup lang="ts">
import { ref } from 'vue'
import type { HashtagGroup } from '@/mock/db'

const props = defineProps<{
  groups: HashtagGroup[]
  /** Async add handler; the draft clears only when it resolves. */
  add?: (groupId: string, tag: string) => Promise<void>
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

function copyGroup(g: HashtagGroup): void {
  const text = g.tags.map((t) => `#${t.replace(/^#/, '')}`).join(' ')
  void copyText(g.id, text)
}

function copySingleTag(gId: string, t: string): void {
  const tag = `#${t.replace(/^#/, '')}`
  void copyText(`${gId}-${t}`, tag)
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
  <div class="hashtag-grid grid grid-cols-1 md:grid-cols-2 gap-4 w-full flex-1">
    <div
      v-for="g in groups"
      :key="g.id"
      class="border border-line rounded-2xl bg-surface p-5 shadow-xs flex flex-col justify-between hover:border-ink/25 transition-all"
    >
      <!-- Group Header -->
      <div class="flex items-center justify-between pb-3 border-b border-line/60">
        <div class="flex items-center gap-2.5">
          <h3 class="font-bold text-base text-ink m-0 tracking-tight">{{ g.title }}</h3>
          <span class="text-2xs font-semibold px-2 py-0.5 rounded-full bg-wash text-muted border border-faint font-mono">
            {{ g.tags.length }} {{ g.tags.length === 1 ? 'tag' : 'tags' }}
          </span>
        </div>

        <button
          type="button"
          class="px-3 py-1 rounded-full text-xs font-semibold transition-all border border-line cursor-pointer select-none"
          :class="copiedKey === g.id ? 'bg-emerald-500 text-white border-emerald-500' : 'bg-surface text-muted hover:text-ink hover:bg-wash hover:border-ink shadow-2xs'"
          title="Copy all tags in this category"
          @click="copyGroup(g)"
        >
          <span v-if="copiedKey === g.id">Copied ✓</span>
          <span v-else>Copy all</span>
        </button>
      </div>

      <!-- Tags Flow -->
      <div class="flex flex-wrap gap-1.5 py-4 flex-1 content-start min-h-[64px]">
        <button
          v-for="(t, i) in g.tags"
          :key="`${t}-${i}`"
          type="button"
          class="inline-flex items-center gap-1 px-3 py-1 rounded-full text-xs font-medium border border-line text-ink transition-all cursor-pointer select-none group"
          :class="copiedKey === `${g.id}-${t}` ? 'bg-emerald-500 !text-white border-emerald-500' : 'bg-surface hover:bg-wash hover:border-ink shadow-2xs'"
          :title="`Click to copy #${t}`"
          @click="copySingleTag(g.id, t)"
        >
          <span>#{{ t }}</span>
          <span v-if="copiedKey === `${g.id}-${t}`" class="text-[10px]">✓</span>
        </button>
        <span v-if="!g.tags.length" class="text-xs text-muted/60 italic py-2">
          No tags in this group yet.
        </span>
      </div>

      <!-- Quick Add Tag Input Bar -->
      <div class="flex items-center gap-2 pt-3 border-t border-faint">
        <div class="relative flex-1">
          <span class="absolute left-3 top-1/2 -translate-y-1/2 text-muted/60 font-mono text-xs select-none pointer-events-none">#</span>
          <input
            class="w-full pl-7 pr-3 py-1.5 rounded-full border border-line bg-surface text-xs text-ink placeholder:text-muted/50 focus:outline-none focus:border-ink transition-colors shadow-2xs"
            :value="drafts[g.id] ?? ''"
            placeholder="add tag..."
            :aria-label="`New tag for ${g.title}`"
            :disabled="!add || busy"
            @input="drafts[g.id] = ($event.target as HTMLInputElement).value"
            @keyup.enter="submit(g.id)"
          />
        </div>
        <button
          type="button"
          class="btn px-3.5 py-1.5 rounded-full bg-ink text-white text-xs font-semibold hover:bg-ink/90 active:scale-95 transition-all cursor-pointer disabled:opacity-50 disabled:cursor-not-allowed shadow-2xs"
          :disabled="!add || busy || !(drafts[g.id] ?? '').trim()"
          @click="submit(g.id)"
        >
          {{ busy ? '…' : 'Add' }}
        </button>
      </div>
    </div>
  </div>

  <p v-if="error" class="autherr mt-3" role="alert">{{ error }}</p>
</template>
