<script setup lang="ts">
const model = defineModel<boolean>({ default: false })
defineProps<{ label?: string; disabled?: boolean; size?: 'sm' | 'md' }>()
</script>

<template>
  <button
    type="button"
    class="p-toggle"
    :class="[size ?? 'md', { on: model }]"
    :aria-pressed="model"
    :disabled="disabled"
    @click="model = !model"
  >
    <slot>{{ label }}</slot>
  </button>
</template>

<style scoped>
.p-toggle {
  width: auto;
  display: inline-flex;
  align-items: center;
  gap: 6px;
  background: transparent;
  border: 1px solid var(--line, #d9d9d9);
  color: var(--muted, #6b6b6b);
  font: inherit;
  font-size: 13px;
  font-weight: 600;
  padding: 6px 12px;
  cursor: pointer;
  transition: all 0.12s;
}
.p-toggle.sm { padding: 4px 9px; font-size: 12px; }
.p-toggle:hover { color: var(--ink, #000); border-color: var(--ink, #000); }
.p-toggle.on { background: var(--ink, #000); color: var(--paper, #fff); border-color: var(--ink, #000); }
.p-toggle:disabled { opacity: 0.4; cursor: not-allowed; }
</style>
