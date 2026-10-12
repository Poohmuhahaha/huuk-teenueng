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
  <div ref="root" class="relative w-full">
    <button
      type="button"
      class="flex items-center justify-between gap-2.5 w-full bg-surface border border-line text-ink font-sans cursor-pointer text-left transition-colors rounded-none focus-visible:outline-none focus-visible:border-ink disabled:opacity-50 disabled:cursor-not-allowed"
      :class="size === 'sm' ? 'px-2.5 py-1.5 text-[13px]' : 'px-3 py-[9px] text-sm'"
      :disabled="disabled"
      :aria-expanded="open"
      aria-haspopup="listbox"
      @click="open = !open"
    >
      <span class="overflow-hidden text-ellipsis whitespace-nowrap" :class="{ 'text-faint': !current }">{{ current ?? placeholder }}</span>
      <span class="text-muted text-[11px]" aria-hidden="true">▾</span>
    </button>
    <div v-if="open" class="absolute z-[60] top-[calc(100%+4px)] left-0 right-0 bg-surface border border-ink max-h-[240px] overflow-y-auto p-1 shadow-md rounded-none" role="listbox">
      <button
        v-for="o in options"
        :key="o.value"
        type="button"
        class="flex items-center justify-between w-full bg-transparent border-0 text-ink font-sans text-[13px] px-2.5 py-2 cursor-pointer text-left rounded-none hover:bg-wash disabled:opacity-40 disabled:cursor-not-allowed"
        :class="{ 'font-bold bg-wash': o.value === model }"
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
