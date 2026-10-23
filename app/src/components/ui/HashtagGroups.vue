<script setup lang="ts">
import { ref } from 'vue'
import type { HashtagGroup } from '@/mock/db'
import type { TrackedTag } from '@/composables/useHashtagTracker'

const props = defineProps<{
  groups: HashtagGroup[]
  /** Async add handler; the draft clears only when it resolves. */
  add?: (groupId: string, tag: string) => Promise<void>
  /** Tracked stats per tag */
  tagStats?: Map<string, TrackedTag>
  /** Helper to find discovered tags from posts for a group */
  getDiscoveredTags?: (groupTitle: string, existingTags: string[]) => TrackedTag[]
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

function getCount(tag: string): number {
  if (!props.tagStats) return 0
  const normalized = tag.trim().toLowerCase().replace(/^#/, '')
  return props.tagStats.get(normalized)?.count ?? 0
}

function getTagTooltip(tag: string): string {
  const normalized = tag.trim().toLowerCase().replace(/^#/, '')
  const stat = props.tagStats?.get(normalized)
  if (!stat || stat.count === 0) return `Click to copy #${tag} (0 posts)`
  const titles = stat.posts.map((p) => p.title).join(', ')
  return `Click to copy #${tag} · Used in ${stat.count} ${stat.count === 1 ? 'post' : 'posts'} (${titles})`
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

async function addDiscoveredTag(groupId: string, tag: string): Promise<void> {
  if (!props.add || busy.value) return
  busy.value = true
  try {
    await props.add(groupId, tag)
  } catch (e) {
    error.value = e instanceof Error ? e.message : String(e)
  } finally {
    busy.value = false
  }
}
</script>

<template>
  <div class="hashtag-stack flex flex-col gap-6 w-full max-w-4xl mx-auto py-2">
    <!-- Category Cards matching reference Image 7 -->
    <div
      v-for="g in groups"
      :key="g.id"
      class="border border-line rounded-2xl bg-surface p-8 shadow-xs flex flex-col items-center justify-between transition-all hover:border-ink/20"
    >
      <!-- Title: Centered -->
      <h3 class="font-bold text-lg text-ink text-center m-0 tracking-tight">
        {{ g.title }}
      </h3>

      <!-- Tag Pills: Centered with post usage counts -->
      <div class="flex items-center justify-center flex-wrap gap-2.5 py-6 w-full min-h-[58px]">
        <button
          v-for="(t, i) in g.tags"
          :key="`${t}-${i}`"
          type="button"
          class="inline-flex items-center gap-1.5 px-4 py-1.5 rounded-full text-xs font-semibold border transition-all cursor-pointer select-none group"
          :class="[
            copiedKey === `${g.id}-${t}`
              ? '!bg-emerald-500 !text-white !border-emerald-500 shadow-xs'
              : 'border-line bg-surface hover:border-ink hover:bg-wash text-ink shadow-2xs'
          ]"
          :title="getTagTooltip(t)"
          @click="copySingleTag(g.id, t)"
        >
          <span>#{{ t }}</span>
          <!-- Dynamic count tracked from posts -->
          <span
            v-if="getCount(t) > 0"
            class="text-[10px] font-mono px-1.5 py-0.2 rounded-full transition-colors"
            :class="copiedKey === `${g.id}-${t}` ? 'bg-white/20 text-white' : 'bg-wash text-muted group-hover:text-ink border border-faint'"
          >
            {{ getCount(t) }}
          </span>
          <span v-if="copiedKey === `${g.id}-${t}`" class="text-[10px] ml-0.5">✓</span>
        </button>

        <!-- Discovered tags dynamically tracked from fetched posts for this category -->
        <template v-if="getDiscoveredTags">
          <div
            v-for="disc in getDiscoveredTags(g.title, g.tags)"
            :key="disc.tag"
            class="inline-flex items-center gap-1 px-3 py-1 rounded-full text-xs font-medium border border-dashed border-ink/30 bg-wash/50 text-ink/80 hover:border-ink transition-all"
            :title="`Discovered from ${disc.count} post(s). Click '+' to save to ${g.title}.`"
          >
            <span
              class="cursor-pointer hover:underline"
              @click="copySingleTag(g.id, disc.tag)"
            >
              #{{ disc.tag }}
            </span>
            <span class="text-[10px] font-mono opacity-70">({{ disc.count }})</span>
            <button
              v-if="add"
              type="button"
              class="w-4 h-4 rounded-full bg-ink/10 hover:bg-ink hover:text-white flex items-center justify-center text-[10px] font-bold transition-all cursor-pointer border-0 p-0 ml-0.5"
              title="Save to this group"
              @click.stop="addDiscoveredTag(g.id, disc.tag)"
            >
              +
            </button>
          </div>
        </template>

        <span v-if="!g.tags.length" class="text-xs text-muted/60 italic py-1">
          No tags in this category yet.
        </span>
      </div>

      <!-- Underline input + ADD button matching Image 7 -->
      <form
        class="w-full flex items-center gap-4 pt-4 border-t border-transparent"
        @submit.prevent="submit(g.id)"
      >
        <div class="relative flex-1 flex items-center border-b border-ink/80 focus-within:border-ink transition-colors pb-1">
          <input
            :value="drafts[g.id] ?? ''"
            type="text"
            placeholder="new tag"
            :aria-label="`New tag for ${g.title}`"
            :disabled="!add || busy"
            class="w-full bg-transparent border-0 p-0 text-sm text-ink placeholder:text-muted/60 focus:outline-none focus:ring-0 leading-normal"
            @input="drafts[g.id] = ($event.target as HTMLInputElement).value"
          />
        </div>
        <button
          type="submit"
          class="px-6 py-1 rounded-full border border-ink bg-transparent text-xs font-bold uppercase tracking-wider text-ink hover:bg-ink hover:text-white active:scale-95 transition-all cursor-pointer whitespace-nowrap disabled:opacity-40 disabled:cursor-not-allowed shadow-2xs"
          :disabled="!add || busy || !(drafts[g.id] ?? '').trim()"
        >
          {{ busy ? '…' : 'ADD' }}
        </button>
      </form>
    </div>
  </div>

  <p v-if="error" class="autherr mt-3 text-center" role="alert">{{ error }}</p>
</template>
