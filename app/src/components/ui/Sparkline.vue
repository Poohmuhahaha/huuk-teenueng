<script setup lang="ts">
import { computed } from 'vue'

const props = withDefaults(defineProps<{ values: number[]; width?: number; height?: number }>(), {
  width: 96,
  height: 26,
})

function num(v: unknown): number {
  return typeof v === 'number' && Number.isFinite(v) ? Math.max(0, v) : 0
}

const clean = computed(() => props.values.map(num))
const max = computed(() => Math.max(1, ...clean.value))
const points = computed(() =>
  clean.value
    .map((v, i) => {
      const x = (i / Math.max(1, clean.value.length - 1)) * (props.width - 2) + 1
      const y = props.height - 2 - (v / max.value) * (props.height - 4)
      return `${x.toFixed(1)},${y.toFixed(1)}`
    })
    .join(' '),
)
const single = computed(() => {
  if (clean.value.length !== 1) return null
  const v = clean.value[0]
  return { x: props.width / 2, y: props.height - 2 - (v / max.value) * (props.height - 4) }
})
</script>

<template>
  <svg v-if="values.length" class="block w-full" :viewBox="`0 0 ${width} ${height}`" preserveAspectRatio="none" aria-hidden="true"
    :style="{ height: `${height}px` }">
    <circle v-if="single" :cx="single.x" :cy="single.y" r="2" fill="currentColor" />
    <polyline v-else :points="points" fill="none" stroke="currentColor" stroke-width="1.5"
      vector-effect="non-scaling-stroke" />
  </svg>
</template>
