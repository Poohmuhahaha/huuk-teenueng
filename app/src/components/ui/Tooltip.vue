<script setup lang="ts">
withDefaults(defineProps<{ text: string; side?: 'top' | 'bottom' }>(), { side: 'top' })
</script>

<template>
  <span class="group relative inline-flex" tabindex="0">
    <slot />
    <span
      class="absolute left-1/2 -translate-x-1/2 z-[60] bg-ink text-surface text-[11px] font-medium px-2 py-1 whitespace-nowrap opacity-0 pointer-events-none transition-opacity duration-100 rounded-none group-hover:opacity-100 group-focus-visible:opacity-100"
      :class="side === 'bottom' ? 'top-[calc(100%+6px)]' : 'bottom-[calc(100%+6px)]'"
      role="tooltip"
    >{{ text }}</span>
  </span>
</template>
