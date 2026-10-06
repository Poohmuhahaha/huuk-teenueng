<script setup lang="ts">
const model = defineModel<number>({ default: 0 })
withDefaults(defineProps<{ min?: number; max?: number; step?: number; showValue?: boolean }>(), { min: 0, max: 100, step: 1 })
</script>

<template>
  <div class="p-slider">
    <input v-model.number="model" type="range" :min="min" :max="max" :step="step" />
    <span v-if="showValue" class="p-slider-val">{{ model }}</span>
  </div>
</template>

<style scoped>
.p-slider { display: flex; align-items: center; gap: 12px; width: 100%; }
.p-slider input { accent-color: var(--ink, #000); width: 100%; height: 22px; }
.p-slider-val {
  min-width: 3ch;
  text-align: right;
  font-size: 13px;
  font-weight: 600;
  color: var(--ink, #000);
  font-variant-numeric: tabular-nums;
}
</style>
