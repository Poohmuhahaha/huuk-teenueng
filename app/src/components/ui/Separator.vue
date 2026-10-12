<script setup lang="ts">
withDefaults(defineProps<{ orientation?: 'horizontal' | 'vertical'; label?: string }>(), {
  orientation: 'horizontal',
})
</script>

<template>
  <div v-if="label" class="flex items-center gap-2.5 w-full">
    <span class="flex-1 h-px bg-line" />
    <span class="text-[11px] tracking-[0.08em] uppercase text-muted">{{ label }}</span>
    <span class="flex-1 h-px bg-line" />
  </div>
  <div
    v-else
    class="bg-line shrink-0"
    :class="orientation === 'vertical' ? 'w-px self-stretch min-h-4' : 'h-px w-full'"
    role="separator"
  />
</template>
