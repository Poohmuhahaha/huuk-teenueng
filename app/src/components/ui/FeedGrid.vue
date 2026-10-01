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
    <div v-for="p in posts" :key="p.id" class="cell" :style="tileStyle" :title="p.topic">
      <span class="badge">{{ platformName }}</span>
      <span class="letter">{{ (p.topic || '?').slice(0, 1).toUpperCase() }}</span>
      <span class="size">{{ size.w }}×{{ size.h }}</span>
      <span v-if="p.date" class="size">{{ p.date }}</span>
    </div>
  </div>
  <p v-if="!posts.length" class="muted">Nothing scheduled in range.</p>
</template>

<style scoped>
/* Adapt the global square .feed grid to per-platform canvas ratios. */
.feed {
  grid-template-columns: repeat(auto-fill, minmax(120px, 1fr));
  grid-auto-flow: dense;
}
.feed.wide .cell {
  grid-column: span 2;
}
.cell {
  flex-direction: column;
  gap: 2px;
}
.cell .badge {
  position: absolute; top: 4px; left: 4px; padding: 1px 6px;
  border-radius: 999px; background: rgba(0, 0, 0, 0.55); color: #fff; font-size: 9.5px;
  font-weight: 700; letter-spacing: 0.02em;
}
.cell .size {
  font-size: 10px;
  font-weight: 600;
  color: var(--muted);
}
</style>
