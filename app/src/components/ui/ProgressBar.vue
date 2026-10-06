<script setup lang="ts">
// ProgressBar — thin track with an ink fill (ported from Pugu ui/ProgressBar).
import { computed } from 'vue'

const props = withDefaults(defineProps<{ value?: number }>(), { value: 0 })
const pct = computed(() => Math.max(0, Math.min(100, Number.isFinite(props.value) ? props.value : 0)))
</script>

<template>
  <div class="p-progress" role="progressbar" :aria-valuenow="pct" aria-valuemin="0" aria-valuemax="100">
    <div class="p-progress-fill" :style="{ width: `${pct}%` }" />
  </div>
</template>

<style scoped>
.p-progress { width: 100%; height: 8px; background: var(--wash, #f6f6f6); overflow: hidden; }
.p-progress-fill { height: 100%; background: var(--ink, #000); transition: width 0.3s ease; }
</style>
