<script setup lang="ts">
// Alert — inline message box. Ported from Pugu (ui/Alert) to Huuk design
// (monochrome, hairline, square).
withDefaults(defineProps<{ title?: string; tone?: 'info' | 'success' | 'warning' | 'danger' }>(), {
  tone: 'info',
})

const toneClasses: Record<string, string> = {
  info: 'border-l-ink',
  success: 'border-l-ink border-l-double',
  warning: 'border-l-ink border-l-dashed',
  danger: 'border-l-ink bg-wash',
}
</script>

<template>
  <div
    class="border border-line border-l-4 p-3 text-sm text-ink w-full rounded-none"
    :class="toneClasses[tone] || toneClasses.info"
    role="alert"
  >
    <strong v-if="title" class="block mb-0.5">{{ title }}</strong>
    <slot />
  </div>
</template>
