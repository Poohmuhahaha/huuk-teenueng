<script setup lang="ts">
// Reusable month navigator — compact ‹ Month › toggle (used by every hub card
// that selects a month: Home, Monthly Planner, Performance).
import { monthName } from '@/core/i18n'

const props = defineProps<{ modelValue: number; disabled?: boolean }>()
const emit = defineEmits<{ 'update:modelValue': [number] }>()

function shift(delta: number): void {
  if (props.disabled) return
  const next = props.modelValue + delta
  if (next >= 1 && next <= 12) emit('update:modelValue', next)
}
</script>

<template>
  <div
    class="month-toggle flex items-center justify-between w-full h-[44px] p-1 border border-line rounded-full bg-surface shadow-xs transition-colors"
    :class="disabled ? 'opacity-60 pointer-events-none' : ''"
    role="group"
    aria-label="Month"
  >
    <button
      type="button"
      class="w-[34px] h-[34px] flex-none inline-flex items-center justify-center p-0 border-0 rounded-full bg-transparent text-muted hover:text-ink hover:bg-wash active:scale-95 transition-all cursor-pointer focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ink disabled:opacity-25 disabled:cursor-not-allowed disabled:hover:bg-transparent disabled:hover:text-muted"
      :disabled="disabled || modelValue <= 1"
      aria-label="Previous month"
      :title="modelValue <= 1 ? 'Already at January' : 'Previous month'"
      @click="shift(-1)"
    >
      <svg viewBox="0 0 16 16" class="w-4 h-4" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
        <path d="M10 12L6 8l4-4" />
      </svg>
    </button>
    <span
      class="flex-1 min-w-0 px-2 text-center text-sm font-semibold tracking-wide text-ink select-none truncate"
      aria-live="polite"
    >
      {{ monthName(modelValue) }}
    </span>
    <button
      type="button"
      class="w-[34px] h-[34px] flex-none inline-flex items-center justify-center p-0 border-0 rounded-full bg-transparent text-muted hover:text-ink hover:bg-wash active:scale-95 transition-all cursor-pointer focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ink disabled:opacity-25 disabled:cursor-not-allowed disabled:hover:bg-transparent disabled:hover:text-muted"
      :disabled="disabled || modelValue >= 12"
      aria-label="Next month"
      :title="modelValue >= 12 ? 'Already at December' : 'Next month'"
      @click="shift(1)"
    >
      <svg viewBox="0 0 16 16" class="w-4 h-4" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
        <path d="M6 4l4 4-4 4" />
      </svg>
    </button>
  </div>
</template>
