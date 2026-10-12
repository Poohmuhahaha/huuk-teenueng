<script setup lang="ts">
import { ref } from 'vue'
import { useDismiss } from '@/composables/useDismiss'

const open = defineModel<boolean>({ default: false })
withDefaults(defineProps<{ align?: 'start' | 'end'; width?: string }>(), { align: 'start' })
const root = ref<HTMLElement | null>(null)
useDismiss(root, () => (open.value = false))
</script>

<template>
  <div ref="root" class="relative inline-flex">
    <span class="inline-flex cursor-pointer" @click="open = !open"><slot name="trigger" /></span>
    <div
      v-if="open"
      class="absolute z-[60] top-[calc(100%+6px)] min-w-[200px] bg-surface border border-ink p-3 shadow-[4px_4px_0_var(--color-ink)] rounded-none"
      :class="align === 'end' ? 'right-0' : 'left-0'"
      :style="width ? { width } : undefined"
    >
      <slot />
    </div>
  </div>
</template>
