<script setup lang="ts">
withDefaults(defineProps<{ width?: string }>(), { width: '280px' })
</script>

<template>
  <span class="group relative inline-flex">
    <span class="inline-flex"><slot name="trigger" /></span>
    <span
      class="absolute left-0 top-[calc(100%+8px)] z-[60] bg-surface border border-ink p-3.5 opacity-0 invisible transition-all duration-150 rounded-none shadow-md group-hover:opacity-100 group-hover:visible group-focus-within:opacity-100 group-focus-within:visible"
      :style="{ width }"
    >
      <slot />
    </span>
  </span>
</template>
