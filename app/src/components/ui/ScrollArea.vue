<script setup lang="ts">
withDefaults(defineProps<{ maxHeight?: string; axis?: 'y' | 'x' | 'both' }>(), {
  maxHeight: '240px',
  axis: 'y',
})
</script>

<template>
  <div
    class="w-full"
    :class="{
      'overflow-y-auto': axis === 'y',
      'overflow-x-auto': axis === 'x',
      'overflow-auto': axis === 'both',
    }"
    :style="{ maxHeight: axis === 'x' ? undefined : maxHeight }"
  >
    <slot />
  </div>
</template>
