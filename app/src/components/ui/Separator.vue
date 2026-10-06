<script setup lang="ts">
withDefaults(defineProps<{ orientation?: 'horizontal' | 'vertical'; label?: string }>(), {
  orientation: 'horizontal',
})
</script>

<template>
  <div v-if="label" class="p-sep-label">
    <span class="p-sep-line" />
    <span class="p-sep-text">{{ label }}</span>
    <span class="p-sep-line" />
  </div>
  <div v-else class="p-sep" :class="orientation" role="separator" />
</template>

<style scoped>
.p-sep { background: var(--line, #d9d9d9); }
.p-sep.horizontal { height: 1px; width: 100%; }
.p-sep.vertical { width: 1px; align-self: stretch; min-height: 16px; }
.p-sep-label { display: flex; align-items: center; gap: 10px; width: 100%; }
.p-sep-line { flex: 1; height: 1px; background: var(--line, #d9d9d9); }
.p-sep-text { font-size: 11px; letter-spacing: 0.08em; text-transform: uppercase; color: var(--muted, #6b6b6b); }
</style>
