<script setup lang="ts">
// Skeleton — loading placeholder (line or block).
withDefaults(defineProps<{ variant?: 'line' | 'block'; width?: string }>(), { variant: 'line' })
</script>

<template>
  <div
    class="bg-wash w-full animate-pulse rounded-none"
    :class="variant === 'block' ? 'h-10' : 'h-2.5'"
    :style="width ? `width:${width}` : ''"
  />
</template>
