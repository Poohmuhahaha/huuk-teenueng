<script setup lang="ts">
import { computed, ref } from 'vue'
import { useDismiss } from '@/composables/useDismiss'

const model = defineModel<string>()
const props = withDefaults(
  defineProps<{
    options?: { value: string; label: string; disabled?: boolean }[]
    placeholder?: string
    disabled?: boolean
    size?: 'sm' | 'md'
  }>(),
  { options: () => [], placeholder: 'Select…', size: 'md' },
)
const open = ref(false)
const root = ref<HTMLElement | null>(null)
useDismiss(root, () => (open.value = false))
const current = computed(() => props.options.find((o) => o.value === model.value)?.label)
</script>

<template>
  <div ref="root" class="p-select" :class="[`s-${size}`, { open }]">
    <button
      type="button"
      class="p-select-trigger"
      :disabled="disabled"
      :aria-expanded="open"
      aria-haspopup="listbox"
      @click="open = !open"
    >
      <span class="p-select-value" :class="{ placeholder: !current }">{{ current ?? placeholder }}</span>
      <span class="p-select-caret" aria-hidden="true">▾</span>
    </button>
    <div v-if="open" class="p-select-menu" role="listbox">
      <button
        v-for="o in options"
        :key="o.value"
        type="button"
        class="p-select-item"
        :class="{ active: o.value === model }"
        role="option"
        :aria-selected="o.value === model"
        :disabled="o.disabled"
        @click="model = o.value; open = false"
      >
        <span>{{ o.label }}</span>
        <span v-if="o.value === model" aria-hidden="true">✓</span>
      </button>
    </div>
  </div>
</template>

<style scoped>
.p-select { position: relative; width: 100%; }
.p-select-trigger {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 10px;
  width: 100%;
  background: var(--paper, #fff);
  border: 1px solid var(--line, #d9d9d9);
  color: var(--ink, #000);
  font: inherit;
  font-size: 14px;
  padding: 9px 12px;
  cursor: pointer;
  text-align: left;
}
.s-sm .p-select-trigger { padding: 6px 10px; font-size: 13px; }
.p-select-trigger:focus-visible { outline: none; border-color: var(--ink, #000); }
.p-select-trigger:disabled { opacity: 0.5; cursor: not-allowed; }
.p-select-value { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.p-select-value.placeholder { color: var(--faint, #a3a3a3); }
.p-select-caret { color: var(--muted, #6b6b6b); font-size: 11px; }
.p-select-menu {
  position: absolute;
  z-index: 60;
  top: calc(100% + 4px);
  left: 0;
  right: 0;
  background: var(--paper, #fff);
  border: 1px solid var(--ink, #000);
  max-height: 240px;
  overflow-y: auto;
  padding: 4px;
}
.p-select-item {
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
.p-select-item:hover { background: var(--wash, #f6f6f6); }
.p-select-item.active { font-weight: 700; }
.p-select-item:disabled { opacity: 0.4; cursor: not-allowed; }
</style>
