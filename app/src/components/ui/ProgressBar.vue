<script setup lang="ts">
// ProgressBar — thin track with an ink fill (ported from Pugu ui/ProgressBar).
import { computed } from 'vue'

const props = withDefaults(defineProps<{ value?: number }>(), { value: 0 })
const pct = computed(() => Math.max(0, Math.min(100, Number.isFinite(props.value) ? props.value : 0)))
</script>

<template>
  <div class="w-full h-2 bg-wash overflow-hidden rounded-none" role="progressbar" :aria-valuenow="pct" aria-valuemin="0" aria-valuemax="100">
    <div class="h-full bg-ink transition-[width] duration-300 ease-out" :style="{ width: `${pct}%` }" />
  </div>
</template>
