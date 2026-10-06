<script setup lang="ts">
// Compact IA: a hub page groups related screens as tabs instead of separate
// deck cards. Purely presentational — the parent owns the active tab.
export interface HubTab {
  id: string
  label: string
}

defineProps<{ tabs: HubTab[]; modelValue: string }>()
const emit = defineEmits<{ 'update:modelValue': [string] }>()
</script>

<template>
  <div class="hubtabs" role="tablist">
    <button
      v-for="t in tabs"
      :key="t.id"
      type="button"
      role="tab"
      class="hubtab"
      :class="{ active: modelValue === t.id }"
      :aria-selected="modelValue === t.id"
      @click="emit('update:modelValue', t.id)"
    >
      {{ t.label }}
    </button>
  </div>
</template>

<style scoped>
.hubtabs {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  margin: 4px 0 16px;
}
.hubtab {
  width: auto;
  flex: 0 0 auto;
  font: inherit;
  font-size: 13px;
  font-weight: 600;
  letter-spacing: 0.02em;
  padding: 8px 16px;
  border: 1px solid var(--line, #e4e4e4);
  border-radius: var(--radius-pill, 999px);
  background: transparent;
  color: var(--muted, #6b6b6b);
  cursor: pointer;
}
.hubtab:hover {
  color: var(--ink, #000);
  border-color: var(--ink, #000);
}
.hubtab.active {
  background: var(--ink, #000);
  border-color: var(--ink, #000);
  color: var(--paper, #fff);
}
</style>
