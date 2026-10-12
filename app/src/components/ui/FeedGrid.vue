<script setup lang="ts">
import { computed } from 'vue'
import type { Post } from '@/mock/db'
import { FEED_META, type FeedPlatformId } from '@/core/platforms'

const props = withDefaults(defineProps<{ posts: Post[]; platform: FeedPlatformId; variant?: number }>(), {
  variant: -1,
})

// Current canvas size: primary feed default, or a variant by index.
// Unknown platform ids fall back to the default feed spec instead of crashing.
const size = computed(() => {
  const feed = (FEED_META[props.platform] ?? FEED_META.meta).feed
  return props.variant >= 0 && props.variant < feed.variants.length ? feed.variants[props.variant] : feed
})
const wide = computed(() => size.value.w > size.value.h)
const platformName = computed(() => (FEED_META[props.platform] ?? FEED_META.meta).name)
const tileStyle = computed(() => ({ aspectRatio: `${size.value.w} / ${size.value.h}` }))
</script>

<template>
  <div class="feed" :class="{ wide }">
    <div class="grid gap-2 grid-cols-[repeat(auto-fill,minmax(120px,1fr))] [grid-auto-flow:dense]" role="list">
      <div v-for="p in posts" :key="p.id" role="listitem" tabindex="0" :aria-label="p.topic" class="cell bg-surface-2 border border-faint text-muted text-center rounded-none flex justify-center items-center p-1 text-xl leading-[1.6] font-bold overflow-hidden relative flex-col gap-[2px]" :class="wide ? 'col-span-2' : ''" :style="tileStyle" :title="p.topic">
        <span class="absolute top-1 left-1 px-1.5 py-[1px] rounded-full bg-ink/60 text-accent-ink text-[9.5px] font-bold tracking-[0.02em]">{{ platformName }}</span>
        <span class="uppercase">{{ (p.topic || '?').slice(0, 1).toUpperCase() }}</span>
        <span class="text-[10px] font-semibold text-muted">{{ size.w }}×{{ size.h }}</span>
        <span v-if="p.date" class="text-[10px] font-semibold text-muted">{{ p.date }}</span>
      </div>
    </div>
  </div>
  <p v-if="!posts.length" role="status" class="text-muted text-sm">Nothing scheduled in range.</p>
</template>
