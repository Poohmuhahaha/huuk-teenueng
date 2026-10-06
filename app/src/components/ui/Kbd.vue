<script setup lang="ts">
defineProps<{ size?: 'sm' | 'md' }>()
</script>

<template>
  <kbd class="p-kbd" :class="size ?? 'md'"><slot /></kbd>
</template>

<style scoped>
.p-kbd {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  min-width: 20px;
  border: 1px solid var(--line, #d9d9d9);
  border-bottom-width: 2px;
  background: var(--wash, #f6f6f6);
  color: var(--ink, #000);
  font-family: var(--mono, ui-monospace, Menlo, monospace);
  font-size: 11px;
  padding: 2px 6px;
}
.p-kbd.sm { font-size: 10px; padding: 1px 4px; }
</style>
