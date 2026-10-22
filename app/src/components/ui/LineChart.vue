<script setup lang="ts">
import { computed } from 'vue'

const props = withDefaults(
  defineProps<{
    labels: string[]
    series: { label: string; values: number[]; area?: boolean }[]
    height?: number
  }>(),
  { height: 140 },
)

const W = 600

function num(v: unknown): number {
  return typeof v === 'number' && Number.isFinite(v) ? Math.max(0, v) : 0
}

const max = computed(() => Math.max(1, ...props.series.flatMap((s) => s.values.map(num))))

const summary = computed(() =>
  props.series.map((s) => `${s.label}:${s.values.map(num).reduce((a, b) => a + b, 0)}`).join(', '),
)

function x(i: number, n: number): number {
  if (n <= 1) return W / 2
  return (i / (n - 1)) * (W - 12) + 6
}
function y(v: number): number {
  return props.height - 8 - (num(v) / max.value) * (props.height - 16)
}
function linePoints(values: number[]): string {
  return values.map((v, i) => `${x(i, values.length).toFixed(1)},${y(v).toFixed(1)}`).join(' ')
}
function areaPoints(values: number[]): string {
  if (!values.length) return ''
  const last = x(values.length - 1, values.length).toFixed(1)
  return `6,${props.height - 8} ${linePoints(values)} ${last},${props.height - 8}`
}
// Strictly monochrome palette — solid black and charcoal shades
const palette = ['#0a0a0a', '#666666', '#a3a3a3', '#404040']
function color(i: number): string {
  return palette[i % palette.length]
}

const gridLines = computed(() =>
  [0.25, 0.5, 0.75].map((f) => props.height - 8 - f * (props.height - 16)),
)
const labelStep = computed(() => Math.max(1, Math.ceil(props.labels.length / 10)))
const shownLabels = computed(() => props.labels.map((l, i) => ({ l, i })).filter((o) => o.i % labelStep.value === 0))
const totals = computed(() => props.series.map((s) => s.values.map(num).reduce((a, b) => a + b, 0)))

// An all-zero series draws a heavy flat line pinned to the baseline; hide it
// (the legend still reports the 0 total).
function hasData(s: { values: number[] }): boolean {
  return s.values.some((v) => num(v) > 0)
}
</script>

<template>
  <div class="border border-line rounded-xl bg-surface p-4.5 shadow-xs transition-all">
    <div v-if="!labels.length || !series.length" class="text-muted text-sm">No data yet.</div>
    <template v-else>
      <div class="relative">
        <svg class="block w-full h-auto" :viewBox="`0 0 ${W} ${height}`" role="img" :aria-label="summary">
          <line v-for="(gy, i) in gridLines" :key="i" :x1="0" :x2="W" :y1="gy" :y2="gy" class="stroke-faint" stroke-width="1" />
          <template v-for="(s, si) in series" :key="`${s.label}-${si}`">
            <template v-if="hasData(s)">
              <polygon v-if="s.area" :points="areaPoints(s.values)" :fill="color(si)" opacity="0.08" />
              <polyline :points="linePoints(s.values)" fill="none" :stroke="color(si)"
                :stroke-width="si % 2 ? 1.5 : 2" :stroke-opacity="si % 2 ? 0.7 : 1"
                :stroke-dasharray="si % 2 ? '5 4' : undefined" vector-effect="non-scaling-stroke" />
              <circle v-for="(v, i) in s.values" :key="i" :cx="x(i, s.values.length)" :cy="y(v)" r="2.5"
                :fill="color(si)">
                <title>{{ labels[i] ?? '' }} · {{ s.label }}: {{ num(v).toLocaleString('en-US') }}</title>
              </circle>
            </template>
          </template>
        </svg>
        <!-- Axis labels as HTML: SVG text shrinks with the viewBox and became
             unreadable on phones. The 6% bottom offset matches the baseline. -->
        <span class="absolute top-0.5 left-1 text-[11px] leading-none font-medium text-muted pointer-events-none">{{ max.toLocaleString('en-US') }}</span>
        <span class="absolute bottom-[6%] left-1 text-[11px] leading-none font-medium text-muted pointer-events-none">0</span>
      </div>
      <div class="flex flex-wrap gap-3 mt-2 text-xs">
        <span v-for="(s, si) in series" :key="`${s.label}-${si}`" class="inline-flex items-center gap-1.5">
          <i class="w-2.5 h-2.5 rounded-none inline-block" :style="{ background: color(si) }" />{{ s.label }}
          <b class="font-bold">{{ totals[si].toLocaleString('en-US') }}</b>
        </span>
      </div>
      <div class="flex justify-between gap-1 mt-1 text-[11.5px] text-muted">
        <span v-for="o in shownLabels" :key="o.i">{{ o.l }}</span>
      </div>
    </template>
  </div>
</template>
