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
  <div ref="root" class="relative w-full">
    <input
      v-model="query"
      class="w-full bg-surface border border-line text-ink font-sans text-sm p-3 py-[9px] pr-8 rounded-none transition-colors focus:outline-none focus:border-ink placeholder:text-faint"
      :placeholder="placeholder"
      role="combobox"
      :aria-expanded="open"
      @focus="open = true"
      @input="open = true"
    />
    <span class="absolute right-3 top-1/2 -translate-y-1/2 text-muted text-xs pointer-events-none" aria-hidden="true">⌄</span>
    <div
      v-if="open"
      class="absolute z-[60] top-[calc(100%+4px)] left-0 right-0 bg-surface border border-ink max-h-[220px] overflow-y-auto p-1 shadow-md rounded-none"
      role="listbox"
    >
      <button
        v-for="o in filtered"
        :key="o.value"
        type="button"
        class="flex items-center justify-between w-full bg-transparent border-0 text-ink font-sans text-[13px] px-2.5 py-2 cursor-pointer text-left rounded-none hover:bg-wash"
        :class="{ 'font-bold bg-wash': o.value === model }"
        role="option"
        :aria-selected="o.value === model"
        @click="pick(o.value, o.label)"
      >
        <span>{{ o.label }}</span>
        <span v-if="o.value === model" aria-hidden="true">✓</span>
      </button>
      <p v-if="!filtered.length" class="m-1 text-xs text-muted">No matches</p>
    </div>
  </div>
</template>
