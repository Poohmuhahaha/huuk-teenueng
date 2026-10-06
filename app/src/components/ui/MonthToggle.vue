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
  <div class="month-toggle" :class="{ disabled }" role="group" aria-label="Month">
    <button type="button" class="mt-btn" :disabled="disabled || modelValue <= 1" aria-label="Previous month" @click="shift(-1)">‹</button>
    <span class="mt-label" aria-live="polite">{{ monthName(modelValue) }}</span>
    <button type="button" class="mt-btn" :disabled="disabled || modelValue >= 12" aria-label="Next month" @click="shift(1)">›</button>
  </div>
</template>

<style scoped>
/* Compact pill: circular ghost arrows hugging a fixed-width label so the
   control never jumps as month names grow/shrink. */
.month-toggle {
  display: inline-flex;
  align-items: center;
  gap: 2px;
  padding: 4px;
  margin: 2px 0 20px;
  border: 1px solid var(--line, #e4e4e4);
  border-radius: var(--radius-pill, 999px);
  background: var(--surface, #fff);
}
.mt-btn {
  width: 34px;
  height: 34px;
  flex: 0 0 auto;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  padding: 0;
  border: 0;
  border-radius: 50%;
  background: transparent;
  color: var(--muted, #6b6b6b);
  font: inherit;
  font-size: 18px;
  line-height: 1;
  cursor: pointer;
  transition: background 0.15s, color 0.15s;
}
.mt-btn:hover:not(:disabled) {
  background: var(--wash, #f6f6f6);
  color: var(--ink, #000);
}
.mt-btn:disabled {
  opacity: 0.3;
  cursor: default;
}
.mt-label {
  min-width: 128px;
  padding: 0 8px;
  text-align: center;
  font-size: 14px;
  font-weight: 600;
  letter-spacing: 0.01em;
  color: var(--ink, #000);
}
.month-toggle.disabled {
  opacity: 0.6;
}
</style>
