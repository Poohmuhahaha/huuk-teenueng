<script setup lang="ts">
import { computed } from 'vue'

const props = withDefaults(
  defineProps<{
    labels: string[]
    series: { label: string; values: number[]; hollow?: boolean }[]
    height?: number
  }>(),
  { height: 110 },
)

function num(v: unknown): number {
  return typeof v === 'number' && Number.isFinite(v) ? Math.max(0, v) : 0
}

const columns = computed(() =>
  props.labels.map((label, i) => {
    const segs = props.series.map((s) => ({ label: s.label, value: num(s.values[i] ?? 0), hollow: s.hollow ?? false }))
    const total = segs.reduce((a, b) => a + b.value, 0)
    return { label, segs, total }
  }),
)
const maxTotal = computed(() => Math.max(1, ...columns.value.map((c) => c.total)))

const summary = computed(() =>
  columns.value.map((c) => `${c.label}:${c.total}`).join(', '),
)

const colors = ['#0a0a0a', '#404040', '#666666', '#8c8c8c']
function color(i: number): string {
  return colors[i % colors.length]
}
function opacity(i: number): number {
  return Math.max(0.3, 1 - i * 0.16)
}
</script>

<template>
  <div class="border border-line rounded-none bg-surface p-[18px] shadow-none" role="img" :aria-label="summary">
    <div v-if="!labels.length" class="text-muted text-sm">No data yet.</div>
    <template v-else>
      <div class="flex items-end gap-2.5 px-1 pt-2" :style="{ height: `${height}px` }">
        <div v-for="(c, ci) in columns" :key="`${c.label}-${ci}`" class="flex-1 h-full flex items-end" :title="`${c.label}: ${c.total}`">
          <div class="w-full flex flex-col-reverse rounded-none overflow-hidden" :style="{ height: c.total ? `${Math.max(3, Math.round((c.total / maxTotal) * 100))}%` : '0' }">
            <div v-for="(s, si) in c.segs" :key="`${s.label}-${si}`" class="w-full" :style="{
              height: c.total ? `${(s.value / c.total) * 100}%` : '0%',
              background: color(si),
              opacity: s.hollow ? 0.18 : opacity(si),
            }" />
          </div>
        </div>
      </div>
      <div class="flex gap-2.5 mt-1.5 text-2xs text-muted">
        <span v-for="(c, ci) in columns" :key="`${c.label}-${ci}`" class="flex-1 text-center">{{ c.label }}</span>
      </div>
      <div class="flex flex-wrap gap-3 mt-2 text-xs">
        <span v-for="(s, si) in series" :key="`${s.label}-${si}`" class="inline-flex items-center gap-1.5">
          <i class="w-2.5 h-2.5 rounded-none inline-block" :style="{ background: color(si), opacity: s.hollow ? 0.18 : opacity(si) }" />{{ s.label }}
        </span>
      </div>
    </template>
  </div>
</template>
