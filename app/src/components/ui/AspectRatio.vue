<script setup lang="ts">
withDefaults(defineProps<{ ratio?: number }>(), { ratio: 16 / 9 })
</script>

<template>
  <div class="p-ar" :style="{ aspectRatio: String(ratio) }">
    <slot />
  </div>
</template>

<style scoped>
.p-ar { width: 100%; overflow: hidden; background: var(--wash, #f6f6f6); }
.p-ar :deep(img),
.p-ar :deep(video),
.p-ar :deep(canvas) { width: 100%; height: 100%; object-fit: cover; display: block; }
</style>
