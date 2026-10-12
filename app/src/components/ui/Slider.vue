<script setup lang="ts">
const model = defineModel<number>({ default: 0 })
withDefaults(defineProps<{ min?: number; max?: number; step?: number; showValue?: boolean }>(), { min: 0, max: 100, step: 1 })
</script>

<template>
  <div class="flex items-center gap-3 w-full">
    <input
      v-model.number="model"
      type="range"
      class="w-full h-[22px] accent-ink cursor-pointer"
      :min="min"
      :max="max"
      :step="step"
    />
    <span v-if="showValue" class="min-w-[3ch] text-right text-[13px] font-semibold text-ink tabular-nums">{{ model }}</span>
  </div>
</template>
