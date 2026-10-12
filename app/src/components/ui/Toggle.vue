<script setup lang="ts">
const model = defineModel<boolean>({ default: false })
defineProps<{ label?: string; disabled?: boolean; size?: 'sm' | 'md' }>()
</script>

<template>
  <button
    type="button"
    class="w-auto inline-flex items-center gap-1.5 border font-sans font-semibold cursor-pointer rounded-none transition-all duration-100 disabled:opacity-40 disabled:cursor-not-allowed hover:border-ink hover:text-ink"
    :class="[
      size === 'sm' ? 'px-2 py-1 text-xs' : 'px-3 py-1.5 text-[13px]',
      model ? 'bg-ink text-surface border-ink' : 'bg-transparent border-line text-muted'
    ]"
    :aria-pressed="model"
    :disabled="disabled"
    @click="model = !model"
  >
    <slot>{{ label }}</slot>
  </button>
</template>
