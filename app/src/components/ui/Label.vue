<script setup lang="ts">
defineProps<{ htmlFor?: string; required?: boolean }>()
</script>

<template>
  <label class="inline-flex items-center gap-1 text-xs font-semibold tracking-[0.04em] uppercase text-muted w-full" :for="htmlFor">
    <slot />
    <span v-if="required" class="text-ink" aria-hidden="true">*</span>
  </label>
</template>
