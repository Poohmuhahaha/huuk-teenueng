<script setup lang="ts">
// Skeleton — loading placeholder (line or block).
withDefaults(defineProps<{ variant?: 'line' | 'block'; width?: string }>(), { variant: 'line' })
</script>

<template>
  <div class="p-skel" :class="{ block: variant === 'block' }" :style="width ? `width:${width}` : ''" />
</template>

<style scoped>
.p-skel { background: var(--wash, #f6f6f6); height: 10px; width: 100%; }
.p-skel.block { height: 40px; }
</style>
