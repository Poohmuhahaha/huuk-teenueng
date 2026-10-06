<script setup lang="ts">
// StatCards — responsive grid of label/value tiles (ported from Pugu ui/StatCards).
defineProps<{ items?: { label: string; value: string | number }[] }>()
</script>

<template>
  <div class="p-stats">
    <div v-for="s in items ?? []" :key="s.label" class="p-stat">
      <span class="p-stat-label">{{ s.label }}</span>
      <span class="p-stat-value">{{ s.value }}</span>
    </div>
  </div>
</template>

<style scoped>
.p-stats { display: grid; grid-template-columns: repeat(auto-fit, minmax(120px, 1fr)); gap: 12px; width: 100%; }
.p-stat { border: 1px solid var(--line, #e4e4e4); padding: 12px; display: flex; flex-direction: column; gap: 4px; }
.p-stat-label { font-size: 11px; letter-spacing: 0.1em; text-transform: uppercase; color: var(--muted, #6b6b6b); }
.p-stat-value { font-family: var(--font-display, Georgia, serif); font-size: 20px; font-weight: 700; color: var(--ink, #000); }
</style>
