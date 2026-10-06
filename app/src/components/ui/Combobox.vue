<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { useDismiss } from '@/composables/useDismiss'

const model = defineModel<string>({ default: '' })
const props = withDefaults(
  defineProps<{ options?: { value: string; label: string }[]; placeholder?: string }>(),
  { options: () => [], placeholder: 'Search…' },
)
const root = ref<HTMLElement | null>(null)
const open = ref(false)
const query = ref('')
useDismiss(root, () => (open.value = false))
const selected = computed(() => props.options.find((o) => o.value === model.value))
watch(
  selected,
  (value) => {
    if (!open.value) query.value = value?.label ?? ''
  },
  { immediate: true },
)
const filtered = computed(() => {
  const q = open.value ? query.value.trim().toLowerCase() : ''
  if (!q) return props.options
  return props.options.filter((o) => o.label.toLowerCase().includes(q))
})

function pick(value: string, label: string): void {
  model.value = value
  query.value = label
  open.value = false
}
</script>

<template>
  <div ref="root" class="p-combo">
    <input
      v-model="query"
      class="p-combo-input"
      :placeholder="placeholder"
      role="combobox"
      :aria-expanded="open"
      @focus="open = true"
      @input="open = true"
    />
    <span class="p-combo-caret" aria-hidden="true">⌄</span>
    <div v-if="open" class="p-combo-menu" role="listbox">
      <button
        v-for="o in filtered"
        :key="o.value"
        type="button"
        class="p-combo-item"
        :class="{ active: o.value === model }"
        role="option"
        :aria-selected="o.value === model"
        @click="pick(o.value, o.label)"
      >
        <span>{{ o.label }}</span>
        <span v-if="o.value === model" aria-hidden="true">✓</span>
      </button>
      <p v-if="!filtered.length" class="p-combo-empty">No matches</p>
    </div>
  </div>
</template>

<style scoped>
.p-combo { position: relative; width: 100%; }
.p-combo-input {
  width: 100%;
  background: var(--paper, #fff);
  border: 1px solid var(--line, #d9d9d9);
  color: var(--ink, #000);
  font: inherit;
  font-size: 14px;
  padding: 9px 32px 9px 12px;
}
.p-combo-input:focus { outline: none; border-color: var(--ink, #000); }
.p-combo-caret { position: absolute; right: 12px; top: 50%; transform: translateY(-50%); color: var(--muted, #6b6b6b); font-size: 12px; pointer-events: none; }
.p-combo-menu {
  position: absolute;
  z-index: 60;
  top: calc(100% + 4px);
  left: 0;
  right: 0;
  background: var(--paper, #fff);
  border: 1px solid var(--ink, #000);
  max-height: 220px;
  overflow-y: auto;
  padding: 4px;
}
.p-combo-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  width: 100%;
  background: transparent;
  border: 0;
  color: var(--ink, #000);
  font: inherit;
  font-size: 13px;
  padding: 8px 10px;
  cursor: pointer;
  text-align: left;
}
.p-combo-item:hover { background: var(--wash, #f6f6f6); }
.p-combo-item.active { font-weight: 700; }
.p-combo-empty { margin: 4px; font-size: 12px; color: var(--muted, #6b6b6b); }
</style>
