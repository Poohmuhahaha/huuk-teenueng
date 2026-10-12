<script setup lang="ts">
import { computed } from 'vue'

const props = defineProps<{ items: { label: string; value: number }[] }>()

const SIZE = 148
const CX = 74
const R = 54
const C = 2 * Math.PI * R

function num(v: unknown): number {
  return typeof v === 'number' && Number.isFinite(v) ? Math.max(0, v) : 0
}

const total = computed(() => props.items.reduce((s, x) => s + num(x.value), 0))
const summary = computed(() => props.items.map((i) => `${i.label}:${num(i.value)}`).join(', '))
const segs = computed(() => {
  let acc = 0
  return props.items.map((it, i) => {
    const frac = total.value ? num(it.value) / total.value : 0
    const seg = { ...it, value: num(it.value), i, len: frac * C, offset: -acc, pct: Math.round(frac * 100) }
    acc += frac * C
    return seg
  })
})

const colors = ['#0a0a0a', '#404040', '#666666', '#8c8c8c']
function color(i: number): string {
  return colors[i % colors.length]
}
function opacity(i: number): number {
  return Math.max(0.25, 1 - i * 0.16)
}
</script>

<template>
  <div class="border border-line rounded-none bg-surface p-[18px] shadow-none">
    <div v-if="!items.length || !total" class="text-muted text-sm">No data yet.</div>
    <div v-else class="flex items-center gap-[18px]">
      <svg class="w-[148px] h-[148px] flex-none" :viewBox="`0 0 ${SIZE} ${SIZE}`" role="img" :aria-label="summary">
        <g :transform="`rotate(-90 ${CX} ${CX})`">
          <circle :cx="CX" :cy="CX" :r="R" fill="none" stroke="var(--faint)" stroke-width="16" />
          <circle v-for="s in segs" :key="`${s.label}-${s.i}`" :cx="CX" :cy="CX" :r="R" fill="none" :stroke="color(s.i)"
            :stroke-opacity="opacity(s.i)" stroke-width="16" :stroke-dasharray="`${s.len} ${C - s.len}`"
            :stroke-dashoffset="s.offset">
            <title>{{ s.label }}: {{ s.value.toLocaleString('en-US') }} ({{ s.pct }}%)</title>
          </circle>
        </g>
        <text :x="CX" :y="70" text-anchor="middle" class="text-2xl font-extrabold fill-ink">{{ total.toLocaleString('en-US') }}</text>
        <text :x="CX" :y="86" text-anchor="middle" class="text-[9px] tracking-[0.14em] fill-muted">TOTAL</text>
      </svg>
      <ul class="list-none m-0 p-0 grid gap-1.5 flex-1 min-w-0">
        <li v-for="s in segs" :key="`${s.label}-${s.i}`" class="grid grid-cols-[10px_minmax(0,1fr)_auto_auto] items-center gap-2 text-[12.5px]">
          <i class="w-2.5 h-2.5 rounded-none block" :style="{ background: color(s.i), opacity: opacity(s.i) }" />
          <span class="truncate overflow-hidden">{{ s.label }}</span>
          <b>{{ s.value.toLocaleString('en-US') }}</b>
          <span class="text-muted text-[12.5px]">{{ s.pct }}%</span>
        </li>
      </ul>
    </div>
  </div>
</template>
