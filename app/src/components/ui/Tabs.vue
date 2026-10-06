<script setup lang="ts">
const model = defineModel<string>()
defineProps<{ tabs: { id: string; label: string; badge?: string | number; disabled?: boolean }[] }>()
</script>

<template>
  <div class="p-tabs" role="tablist">
    <button
      v-for="t in tabs"
      :key="t.id"
      type="button"
      class="p-tab"
      :class="{ active: model === t.id }"
      role="tab"
      :aria-selected="model === t.id"
      :disabled="t.disabled"
      @click="model = t.id"
    >
      <span>{{ t.label }}</span>
      <span v-if="t.badge !== undefined" class="p-tab-badge">{{ t.badge }}</span>
    </button>
  </div>
</template>

<style scoped>
.p-tabs { display: flex; gap: 4px; border-bottom: 1px solid var(--line, #d9d9d9); width: 100%; overflow-x: auto; }
.p-tab {
  width: auto;
  display: inline-flex;
  align-items: center;
  gap: 6px;
  background: transparent;
  border: 0;
  border-bottom: 2px solid transparent;
  color: var(--muted, #6b6b6b);
  font: inherit;
  font-size: 14px;
  font-weight: 600;
  padding: 8px 12px;
  cursor: pointer;
  white-space: nowrap;
  margin-bottom: -1px;
}
.p-tab:hover { color: var(--ink, #000); }
.p-tab.active { color: var(--ink, #000); border-bottom-color: var(--ink, #000); }
.p-tab:disabled { opacity: 0.4; cursor: not-allowed; }
.p-tab-badge {
  font-size: 10px;
  font-weight: 700;
  border: 1px solid currentColor;
  padding: 0 5px;
  line-height: 1.5;
}
</style>
