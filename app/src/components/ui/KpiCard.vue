<script setup lang="ts">
import { computed } from 'vue'
import Sparkline from './Sparkline.vue'

const props = defineProps<{
  label: string
  value: string | number
  hint?: string
  trend?: string | number
  info?: string
  to?: string
  interactive?: boolean
  spark?: number[]
}>()

const emit = defineEmits<{ select: [] }>()

function pick(): void {
  emit('select')
}

const isPositiveTrend = computed(() => {
  if (props.trend === undefined || props.trend === null) return true
  const str = String(props.trend).trim()
  if (str.startsWith('-')) return false
  return true
})

const trendFormatted = computed(() => {
  if (props.trend === undefined || props.trend === null) return ''
  const str = String(props.trend).trim()
  if (typeof props.trend === 'number') {
    return props.trend > 0 ? `+${props.trend}%` : `${props.trend}%`
  }
  return str
})
</script>

<template>
  <component
    :is="to ? 'RouterLink' : interactive ? 'div' : 'article'"
    :to="to"
    :role="interactive ? 'button' : undefined"
    :tabindex="interactive ? 0 : undefined"
    class="card-fade group border border-line rounded-[20px] sm:rounded-[24px] bg-surface shadow-xs hover:shadow-md hover:border-ink transition-all relative flex flex-col justify-between p-4 sm:p-5 text-left no-underline text-inherit focus-visible:border-ink focus-visible:ring-2 focus-visible:ring-accent overflow-hidden"
    :class="{ 'cursor-pointer': interactive || to }"
    @click="interactive ? pick() : undefined"
    @keydown.enter.prevent="interactive ? pick() : undefined"
    @keydown.space.prevent="interactive ? pick() : undefined"
  >
    <div class="relative z-10">
      <!-- Header row: Label + Trend Pill / Action -->
      <div class="flex items-center justify-between gap-2">
        <h3 class="flex items-center gap-1.5 text-[12.5px] font-medium text-muted tracking-tight m-0 leading-none">
          <span>{{ label }}</span>
          <button
            v-if="info"
            type="button"
            class="inline-flex h-6 w-6 items-center justify-center rounded-full text-muted/60 transition hover:bg-wash hover:text-ink focus-visible:outline-none"
            :title="info"
            :aria-label="info"
            @click.stop
          >
            <svg
              xmlns="http://www.w3.org/2000/svg"
              width="14"
              height="14"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="2"
              stroke-linecap="round"
              stroke-linejoin="round"
              class="opacity-70 hover:opacity-100"
              aria-hidden="true"
            >
              <circle cx="12" cy="12" r="10" />
              <path d="M12 16v-4" />
              <path d="M12 8h.01" />
            </svg>
          </button>
        </h3>

        <!-- Trend Pill -->
        <span
          v-if="trendFormatted"
          class="inline-flex items-center gap-0.5 rounded-full px-2 py-0.5 text-[11px] tracking-tight bg-black text-white font-semibold"
        >
          <svg
            v-if="isPositiveTrend"
            xmlns="http://www.w3.org/2000/svg"
            width="12"
            height="12"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2.2"
            stroke-linecap="round"
            stroke-linejoin="round"
            aria-hidden="true"
          >
            <path d="M7 7h10v10" />
            <path d="M7 17 17 7" />
          </svg>
          <svg
            v-else
            xmlns="http://www.w3.org/2000/svg"
            width="12"
            height="12"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2.2"
            stroke-linecap="round"
            stroke-linejoin="round"
            aria-hidden="true"
          >
            <path d="M7 17h10V7" />
            <path d="M7 7 17 17" />
          </svg>
          {{ trendFormatted }}
        </span>

        <!-- Alternative: Alert pill when hint specifies alert (e.g. over budget) -->
        <span
          v-else-if="hint && hint.toLowerCase().includes('over')"
          class="inline-flex items-center px-2 py-0.5 rounded-full text-[10.5px] font-semibold bg-rose-50 text-rose-600 border border-rose-200"
        >
          {{ hint }}
        </span>

        <!-- Navigation Arrow if linkable and no trend -->
        <svg
          v-else-if="to"
          class="opacity-0 transition-all transform group-hover:opacity-60 group-hover:translate-x-0.5 group-hover:-translate-y-0.5 text-muted"
          viewBox="0 0 16 16"
          width="13"
          height="13"
          fill="none"
          stroke="currentColor"
          stroke-width="1.8"
          stroke-linecap="round"
          stroke-linejoin="round"
          aria-hidden="true"
        >
          <path d="M4 12 12 4M6 4h6v6" />
        </svg>
      </div>

      <!-- Value row: Metric + Subtitle -->
      <div class="mt-2.5 flex items-baseline justify-between gap-2">
        <p class="tabular text-[24px] sm:text-[27px] font-bold leading-tight tracking-[-0.03em] text-ink font-mono m-0">
          {{ value }}
        </p>
        <p
          v-if="hint && !hint.toLowerCase().includes('over')"
          class="truncate text-[11px] font-medium text-muted m-0"
        >
          {{ hint }}
        </p>
      </div>
    </div>

    <!-- Full-width smooth bezier sparkline spanning across card base -->
    <div
      v-if="spark && spark.length"
      class="relative -mx-4 -mb-4 sm:-mx-5 sm:-mb-5 mt-3.5 overflow-hidden rounded-b-[20px] sm:rounded-b-[24px]"
    >
      <Sparkline :values="spark" :height="36" />
    </div>
  </component>
</template>
