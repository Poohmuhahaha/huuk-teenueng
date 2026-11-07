<script setup lang="ts">
import { computed, ref } from 'vue'
import PlatformLogo from './PlatformLogo.vue'
import CountUpNumber from './CountUpNumber.vue'

export interface DonutItem {
  label: string
  value: number
  total?: number
  sublabel?: string
  meta?: string
  posted?: number
  pending?: number
  wip?: number
  hideSublabel?: boolean
  /** Optional brand color; falls back to the monochrome palette. */
  color?: string
  /** Optional platform id/name so the legend shows the brand glyph. */
  platform?: string
}

const props = withDefaults(
  defineProps<{ items: DonutItem[]; card?: boolean }>(),
  { card: true },
)

const emit = defineEmits<{ (e: 'select-item', item: DonutItem): void }>()

const SIZE = 152
const CX = 76
const R = 55
const STROKE = 16
const C = 2 * Math.PI * R

function num(v: unknown): number {
  return typeof v === 'number' && Number.isFinite(v) ? Math.max(0, v) : 0
}

const palette = ['#0a0a0a', '#404040', '#666666', '#8c8c8c', '#262626', '#b3b3b3']
function colorOf(i: number, custom?: string): string {
  return custom ?? palette[i % palette.length]
}

const total = computed(() => props.items.reduce((sum, item) => sum + num(item.value), 0))
const summary = computed(() => props.items.map((item) => `${item.label}:${num(item.value)}`).join(', '))
const hovered = ref<number | null>(null)

const segs = computed(() => {
  let acc = 0
  return props.items.map((item, i) => {
    const value = num(item.value)
    const frac = total.value ? value / total.value : 0
    const len = frac * C
    const seg = {
      ...item,
      value,
      i,
      len,
      offset: -acc,
      pct: Math.round(frac * 100),
      color: colorOf(i, item.color),
    }
    acc += len
    return seg
  })
})

function select(item: DonutItem): void {
  emit('select-item', item)
}
</script>

<template>
  <div
    :class="card ? 'card-fade p-4 sm:p-5 shadow-xs transition-all' : 'w-full'"
  >
    <div v-if="!items.length || !total" class="text-muted text-sm py-8 text-center">No data yet.</div>
    <div v-else class="donut-layout flex flex-wrap items-center gap-4 min-w-0">
      <svg class="donut-svg w-[152px] h-[152px] flex-none" :viewBox="`0 0 ${SIZE} ${SIZE}`" role="img" :aria-label="summary">
        <g :transform="`rotate(-90 ${CX} ${CX})`">
          <circle :cx="CX" :cy="CX" :r="R" fill="none" stroke="var(--faint)" :stroke-width="STROKE" />
          <circle
            v-for="s in segs"
            :key="`${s.label}-${s.i}`"
            :cx="CX"
            :cy="CX"
            :r="R"
            fill="none"
            :stroke="s.color"
            :stroke-width="hovered === s.i ? STROKE + 4 : STROKE"
            :stroke-dasharray="`${s.len} ${C - s.len}`"
            :stroke-dashoffset="s.offset"
            class="cursor-pointer transition-all duration-200"
            :opacity="hovered === null || hovered === s.i ? 1 : 0.45"
            @click="select(s)"
            @mouseenter="hovered = s.i"
            @mouseleave="hovered = null"
          >
            <title>{{ s.label }}: {{ s.value.toLocaleString('en-US') }} ({{ s.pct }}%)</title>
          </circle>
        </g>
        <text :x="CX" :y="CX - 4" text-anchor="middle" class="fill-ink text-[26px] font-extrabold">
          {{ total.toLocaleString('en-US') }}
        </text>
        <text :x="CX" :y="CX + 16" text-anchor="middle" class="fill-muted text-[10.8px] tracking-[0.14em]">TOTAL</text>
      </svg>

      <ul class="donut-legend list-none m-0 p-0 grid gap-0.5 flex-1 min-w-0">
        <li v-for="s in segs" :key="`${s.label}-${s.i}`">
          <button
            type="button"
            class="group w-full min-w-0 flex flex-col items-stretch rounded-lg px-2 py-1.5 text-left transition-colors cursor-pointer hover:bg-wash focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-accent"
            :aria-label="`${s.label}: ${s.value}`"
            @click="select(s)"
            @mouseenter="hovered = s.i"
            @mouseleave="hovered = null"
          >
            <span class="flex w-full min-w-0 items-center gap-2">
              <span v-if="s.platform" class="flex-none" :style="{ color: s.color }">
                <PlatformLogo :name="s.platform" :size="16" />
              </span>
              <span v-else class="w-2.5 h-2.5 rounded-full flex-none" :style="{ background: s.color }" />

              <span class="min-w-0 flex-1">
                <span class="block text-[15px] font-bold text-ink truncate">{{ s.label }}</span>
                <span v-if="!s.hideSublabel && s.sublabel" class="block text-[13.2px] text-muted leading-snug">{{ s.sublabel }}</span>
              </span>

              <span class="text-sm font-bold text-ink font-mono flex-none">
                <CountUpNumber :value="s.value.toLocaleString('en-US')" />
              </span>
              <span class="text-[13.2px] font-semibold text-muted bg-wash border border-line px-1.5 py-0.5 rounded-full flex-none">
                <CountUpNumber :value="`${s.pct}%`" />
              </span>
              <svg
                class="w-3.5 h-3.5 text-muted opacity-0 -translate-x-1 transition-all group-hover:opacity-100 group-hover:translate-x-0 flex-none"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                stroke-width="2.2"
                stroke-linecap="round"
                stroke-linejoin="round"
                aria-hidden="true"
              >
                <path d="m9 18 6-6-6-6" />
              </svg>
            </span>
          </button>
        </li>
      </ul>
    </div>
  </div>
</template>
