<script setup lang="ts">
// Reusable month navigator — compact ‹ Month › toggle (used by every hub card
// that selects a month: Home, Monthly Planner, Performance).
// The center label is a button that opens a 12-month grid plus a year stepper,
// so jumping from e.g. July 2026 back to January 2025 takes two taps instead
// of a year of ‹ presses.
import { onUnmounted, ref, watch } from 'vue'
import { monthName, t } from '@/core/i18n'

const props = withDefaults(defineProps<{ modelValue: number; year?: number; disabled?: boolean }>(), {
  year: () => new Date().getFullYear(),
})
const emit = defineEmits<{ 'update:modelValue': [number]; 'update:year': [number] }>()

const MONTHS = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12]
const open = ref(false)

// Slide-up sheet animation; instant when the OS asks for reduced motion.
const animOK =
  typeof window === 'undefined' ? true : !(window.matchMedia?.('(prefers-reduced-motion: reduce)').matches ?? false)

function stepYear(delta: number): void {
  if (props.disabled) return
  const next = props.year + delta
  if (next >= 1970 && next <= 9999) emit('update:year', next)
}

function toggle(): void {
  if (props.disabled) return
  open.value = !open.value
}

function close(): void {
  open.value = false
}

function pick(month: number): void {
  if (props.disabled) return
  emit('update:modelValue', month)
  close()
}

function onKey(e: KeyboardEvent): void {
  if (e.key === 'Escape') close()
}

watch(open, (v) => {
  if (v) window.addEventListener('keydown', onKey)
  else window.removeEventListener('keydown', onKey)
})
onUnmounted(() => window.removeEventListener('keydown', onKey))
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
      class="month-label w-full flex-1 min-w-0 px-2 relative inline-flex items-center justify-center text-center text-sm font-semibold tracking-wide text-ink select-none truncate bg-transparent border-0 rounded-full cursor-pointer hover:bg-wash active:scale-95 transition-all focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ink disabled:cursor-not-allowed disabled:hover:bg-transparent"
      :disabled="disabled"
      aria-haspopup="dialog"
      :aria-expanded="open"
      title="Choose month and year"
      style="position:relative;display:flex;align-items:center;justify-content:center;text-align:center;width:100%"
      @click="toggle"
    >
      <span class="min-w-0 truncate" aria-live="polite">{{ monthName(modelValue) }} {{ year }}</span>
      <span class="text-muted text-xs" aria-hidden="true" style="margin-left:4px;flex:none">▾</span>
    </button>
  </div>
  <Teleport to="body">
    <Transition name="p-drawer-anim" :css="animOK">
      <div v-if="open" class="backdrop p-drawer-side-bottom fixed inset-0 z-50 flex flex-col justify-end items-center bg-backdrop backdrop-blur-[4px]" @click.self="close">
        <div class="month-modal p-drawer w-full max-w-[480px] bg-surface border border-line border-b-0 shadow-[0_-12px_40px_rgba(0,0,0,0.22)] p-5 pb-[calc(1.25rem+env(safe-area-inset-bottom,0px))]" role="dialog" aria-modal="true" aria-label="Choose month and year">
          <div class="w-10 h-1 bg-muted/40 rounded-full mx-auto mb-3 flex-none" aria-hidden="true" />
          <strong class="block text-center text-base">Choose month</strong>
        <div class="mt-3 flex items-center justify-between gap-2" role="group" aria-label="Year">
          <button
            type="button"
            class="w-auto flex-none inline-flex items-center justify-center px-3 py-2 border-0 rounded-full bg-transparent text-muted text-sm font-semibold cursor-pointer hover:text-ink hover:bg-wash active:scale-95 transition-all focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ink"
            aria-label="Previous year"
            @click="stepYear(-1)"
          >
            ‹
          </button>
          <span class="text-sm font-semibold tracking-wide text-ink tabular-nums" aria-live="polite">{{ year }}</span>
          <button
            type="button"
            class="w-auto flex-none inline-flex items-center justify-center px-3 py-2 border-0 rounded-full bg-transparent text-muted text-sm font-semibold cursor-pointer hover:text-ink hover:bg-wash active:scale-95 transition-all focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ink"
            aria-label="Next year"
            @click="stepYear(1)"
          >
            ›
          </button>
        </div>
        <div class="grid grid-cols-3 gap-1 mt-3">
          <button
            v-for="m in MONTHS"
            :key="m"
            type="button"
            class="month-cell w-full min-w-0 px-2 py-2.5 text-sm border rounded-full cursor-pointer hover:bg-wash"
            :class="m === modelValue ? 'bg-ink text-white border-ink font-semibold' : 'border-transparent bg-transparent text-ink'"
            :aria-pressed="m === modelValue"
            :aria-label="monthName(m)"
            @click="pick(m)"
          >
            {{ monthName(m) }}
          </button>
        </div>
        <div class="actions flex justify-end gap-2 mt-4">
          <button type="button" class="btn flex-1 min-w-0 font-sans uppercase tracking-[0.08em] rounded-full px-5.5 py-3 text-[12.5px] font-semibold border-[1.5px] border-ink bg-ink text-white cursor-pointer shadow-none hover:bg-accent-hover hover:border-accent-hover hover:text-white active:transform-none disabled:opacity-50 disabled:cursor-default" @click="close">{{ t('common.close') }}</button>
        </div>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>
