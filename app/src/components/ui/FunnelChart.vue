<script setup lang="ts">
import { computed } from 'vue'

const props = defineProps<{ steps: { label: string; value: number }[] }>()

function num(v: unknown): number {
  return typeof v === 'number' && Number.isFinite(v) ? Math.max(0, v) : 0
}

const max = computed(() => Math.max(1, ...props.steps.map((s) => num(s.value))))
const first = computed(() => num(props.steps[0]?.value ?? 0))

const summary = computed(() => props.steps.map((s) => `${s.label}:${num(s.value)}`).join(', '))

function width(v: number): string {
  const n = num(v)
  if (n <= 0) return '0%'
  return `${Math.max(3, Math.round((n / max.value) * 100))}%`
}
function share(v: number): number {
  return first.value ? Math.round((num(v) / first.value) * 100) : 0
}
</script>

<template>
  <div class="border border-line rounded-none bg-surface p-[18px] shadow-none" role="img" :aria-label="summary">
    <div v-if="!steps.length" class="text-muted text-sm">No data yet.</div>
    <div v-else class="grid gap-2">
      <div v-for="(s, i) in steps" :key="`${s.label}-${i}`" class="grid grid-cols-[minmax(56px,92px)_minmax(0,1fr)_auto_40px] items-center gap-2.5">
        <span class="text-[12.5px] truncate overflow-hidden">{{ s.label }}</span>
        <div class="h-[14px] bg-faint rounded-full overflow-hidden">
          <div class="h-full bg-accent rounded-full" :style="{ width: width(s.value), opacity: Math.max(0.3, 1 - i * 0.16) }" />
        </div>
        <b class="text-[12.5px] min-w-8 text-right">{{ Number(num(s.value)).toLocaleString('en-US') }}</b>
        <span class="text-muted text-[11.5px] text-right">{{ share(s.value) }}%</span>
      </div>
    </div>
  </div>
</template>
