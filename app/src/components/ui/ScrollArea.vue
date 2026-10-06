<script setup lang="ts">
withDefaults(defineProps<{ maxHeight?: string; axis?: 'y' | 'x' | 'both' }>(), {
  maxHeight: '240px',
  axis: 'y',
})
</script>

<template>
  <div class="p-scroll" :class="axis" :style="{ maxHeight: axis === 'x' ? undefined : maxHeight }">
    <slot />
  </div>
</template>

<style scoped>
.p-scroll { width: 100%; overflow: hidden; }
.p-scroll.y { overflow-y: auto; }
.p-scroll.x { overflow-x: auto; }
.p-scroll.both { overflow: auto; }
.p-scroll::-webkit-scrollbar { width: 8px; height: 8px; }
.p-scroll::-webkit-scrollbar-thumb { background: var(--line, #d9d9d9); }
.p-scroll::-webkit-scrollbar-thumb:hover { background: var(--muted, #6b6b6b); }
.p-scroll::-webkit-scrollbar-track { background: transparent; }
</style>
