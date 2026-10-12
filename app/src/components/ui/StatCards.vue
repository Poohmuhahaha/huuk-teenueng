<script setup lang="ts">
// StatCards — responsive grid of label/value tiles (ported from Pugu ui/StatCards).
defineProps<{ items?: { label: string; value: string | number }[] }>()
</script>

<template>
  <div class="grid grid-cols-[repeat(auto-fit,minmax(120px,1fr))] gap-3 w-full">
    <div v-for="s in items ?? []" :key="s.label" class="border border-line p-3 flex flex-col gap-1 rounded-none bg-surface">
      <span class="text-[11px] tracking-[0.1em] uppercase text-muted">{{ s.label }}</span>
      <span class="font-display text-xl font-bold text-ink">{{ s.value }}</span>
    </div>
  </div>
</template>
