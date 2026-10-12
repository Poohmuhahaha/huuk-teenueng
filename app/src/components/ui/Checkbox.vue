<script setup lang="ts">
import { ref, watchEffect } from 'vue'

const model = defineModel<boolean>({ default: false })
const props = defineProps<{ label?: string; disabled?: boolean; indeterminate?: boolean }>()
const box = ref<HTMLInputElement | null>(null)

watchEffect(() => {
  if (box.value) box.value.indeterminate = !!props.indeterminate && !model.value
})
</script>

<template>
  <label class="group inline-flex items-center gap-2 cursor-pointer text-sm text-ink select-none" :class="{ 'opacity-50 cursor-not-allowed': disabled }">
    <input
      ref="box"
      type="checkbox"
      class="peer sr-only"
      :checked="model"
      :disabled="disabled"
      @change="model = ($event.target as HTMLInputElement).checked"
    />
    <span
      class="inline-flex items-center justify-center w-[18px] h-[18px] shrink-0 border border-line bg-surface text-transparent text-xs rounded-none transition-colors peer-checked:bg-ink peer-checked:text-surface peer-checked:border-ink peer-indeterminate:bg-ink peer-indeterminate:text-surface peer-indeterminate:border-ink peer-focus-visible:outline-2 peer-focus-visible:outline-ink peer-focus-visible:outline-offset-2"
      aria-hidden="true"
    >
      <span v-if="indeterminate" class="text-xs leading-none font-bold text-surface">–</span>
      <span v-else class="text-xs leading-none">✓</span>
    </span>
    <span v-if="label || $slots.default"><slot>{{ label }}</slot></span>
  </label>
</template>
