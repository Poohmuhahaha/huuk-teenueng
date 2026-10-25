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
  return (i / (n - 1)) * (W - 32) + 16
}
function y(v: number): number {
  const topPad = 22
  const bottomPad = 16
  const availHeight = props.height - topPad - bottomPad
  return props.height - bottomPad - (num(v) / (max.value || 1)) * availHeight
}
function linePoints(values: number[]): string {
  return values.map((v, i) => `${x(i, values.length).toFixed(1)},${y(v).toFixed(1)}`).join(' ')
}
function areaPoints(values: number[]): string {
  if (!values.length) return ''
  const first = x(0, values.length).toFixed(1)
  const last = x(values.length - 1, values.length).toFixed(1)
  const baseline = (props.height - 16).toFixed(1)
  return `${first},${baseline} ${linePoints(values)} ${last},${baseline}`
}
// Strictly monochrome palette — solid black and charcoal shades
const palette = ['#0a0a0a', '#666666', '#a3a3a3', '#404040']
function color(i: number): string {
  return palette[i % palette.length]
}

const gridLines = computed(() => {
  const topPad = 22
  const bottomPad = 16
  const availHeight = props.height - topPad - bottomPad
  return [0.25, 0.5, 0.75].map((f) => props.height - bottomPad - f * availHeight)
})
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
          <line v-for="(gy, i) in gridLines" :key="i" :x1="0" :x2="W" :y1="gy" :y2="gy" class="stroke-faint" stroke-width="1" stroke-dasharray="3 3" />
          <template v-for="(s, si) in series" :key="`${s.label}-${si}`">
            <template v-if="hasData(s)">
              <polygon v-if="s.area" :points="areaPoints(s.values)" :fill="color(si)" opacity="0.07" />
              <polyline :points="linePoints(s.values)" fill="none" :stroke="color(si)"
                :stroke-width="si % 2 ? 1.5 : 2" :stroke-opacity="si % 2 ? 0.75 : 1"
                :stroke-dasharray="si % 2 ? '4 3' : undefined" vector-effect="non-scaling-stroke" />
              <template v-for="(v, i) in s.values" :key="i">
                <circle v-if="num(v) > 0" :cx="x(i, s.values.length)" :cy="y(v)" r="3"
                  :fill="color(si)" stroke="var(--surface, #fff)" stroke-width="1.5">
                  <title>{{ labels[i] ?? '' }} · {{ s.label }}: {{ num(v).toLocaleString('en-US') }}</title>
                </circle>
              </template>
            </template>
          </template>
        </svg>
        <!-- Clean Axis labels -->
        <span class="absolute top-1 left-1.5 text-[10.5px] leading-none font-mono font-medium text-muted/80 pointer-events-none select-none">
          Peak: {{ max.toLocaleString('en-US') }}
        </span>
        <span class="absolute bottom-2 left-1.5 text-[10.5px] leading-none font-mono font-medium text-muted/60 pointer-events-none select-none">
          0
        </span>
      </div>
      <div class="flex flex-wrap items-center justify-between gap-3 mt-3 pt-2 border-t border-faint text-xs">
        <div class="flex flex-wrap gap-4">
          <span v-for="(s, si) in series" :key="`${s.label}-${si}`" class="inline-flex items-center gap-1.5 text-muted">
            <span class="w-2 h-2 rounded-full inline-block" :style="{ background: color(si) }" />
            <span>{{ s.label }}:</span>
            <strong class="font-bold font-mono text-ink">{{ (totals[si] ?? 0).toLocaleString('en-US') }}</strong>
          </span>
        </div>
        <div class="flex gap-3 text-[11px] font-mono text-muted/70">
          <span v-for="o in shownLabels" :key="o.i">{{ o.l }}</span>
        </div>
      </div>
    </template>
  </div>
</template>
