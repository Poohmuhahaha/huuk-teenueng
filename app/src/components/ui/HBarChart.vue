<script setup lang="ts">
import { computed } from 'vue'

const props = defineProps<{ items: { label: string; value: number }[] }>()

function num(v: unknown): number {
  return typeof v === 'number' && Number.isFinite(v) ? Math.max(0, v) : 0
}

const max = computed(() => Math.max(1, ...props.items.map((x) => num(x.value))))

const summary = computed(() => props.items.map((i) => `${i.label}:${num(i.value)}`).join(', '))

function width(v: number): string {
  const n = num(v)
  if (n <= 0) return '0%'
  return `${Math.max(2, Math.round((n / max.value) * 100))}%`
}
const colors = ['#0a0a0a', '#404040', '#666666', '#8c8c8c']
function color(i: number): string {
  return colors[i % colors.length]
}
function opacity(i: number): number {
  return Math.max(0.3, 1 - i * 0.16)
}
</script>

<template>
  <div class="border border-line rounded-xl bg-surface p-4.5 shadow-xs transition-all" role="img" :aria-label="summary">
    <div v-if="!items.length" class="text-muted text-sm">No data yet.</div>
    <div v-else class="grid gap-2">
      <div v-for="(it, i) in items" :key="`${it.label}-${i}`" class="grid grid-cols-[minmax(64px,140px)_minmax(0,1fr)_auto] items-center gap-2.5"
        :title="`${it.label}: ${num(it.value).toLocaleString('en-US')}`">
        <span class="text-[12.5px] truncate overflow-hidden">{{ it.label }}</span>
        <div class="h-3 bg-faint rounded-full overflow-hidden">
          <div class="h-full rounded-full" :style="{ width: width(it.value), background: color(i), opacity: opacity(i) }" />
        </div>
        <b class="text-[12.5px] min-w-12 text-right">{{ num(it.value).toLocaleString('en-US') }}</b>
      </div>
    </div>
  </div>
</template>
