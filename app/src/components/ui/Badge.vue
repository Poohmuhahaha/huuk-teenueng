<script setup lang="ts">
withDefaults(defineProps<{ variant?: 'neutral' | 'success' | 'warning' | 'danger' | 'solid'; dot?: boolean }>(), {
  variant: 'neutral',
})

const variantClasses: Record<string, string> = {
  neutral: 'border-line bg-surface text-ink',
  success: 'border-line border-double bg-surface text-ink',
  warning: 'border-line border-dashed bg-surface text-ink',
  danger: 'border-ink bg-ink text-surface',
  solid: 'border-ink bg-ink text-surface',
}
</script>

<template>
  <span
    class="inline-flex items-center gap-[5px] border text-[11px] font-semibold tracking-[0.04em] uppercase px-2 py-0.5 whitespace-nowrap rounded-none"
    :class="variantClasses[variant] || variantClasses.neutral"
  >
    <span v-if="dot" class="w-[5px] h-[5px] rounded-full bg-current" aria-hidden="true" />
    <slot />
  </span>
</template>
