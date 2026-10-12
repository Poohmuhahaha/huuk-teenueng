<script setup lang="ts">
import { watch } from 'vue'

const open = defineModel<boolean>({ default: false })
const props = withDefaults(
  defineProps<{ title?: string; message?: string; tone?: 'info' | 'success' | 'warning' | 'danger'; duration?: number; fixed?: boolean }>(),
  { tone: 'info', duration: 4000 },
)
let timer: number | undefined

watch(open, (value) => {
  if (!value) return
  if (timer) window.clearTimeout(timer)
  if (props.duration > 0) timer = window.setTimeout(() => (open.value = false), props.duration)
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
    class="flex items-start gap-3 w-full bg-surface border border-ink border-l-4 p-3 px-3.5 rounded-none"
    :class="[
      toneClasses[tone] || toneClasses.info,
      fixed ? 'fixed right-5 bottom-5 z-[1000] w-auto min-w-[280px] max-w-[380px] shadow-[4px_4px_0_var(--color-ink)]' : ''
    ]"
    role="status"
  >
    <div class="flex flex-col gap-0.5 flex-1 text-left">
      <strong v-if="title" class="text-[13px] font-semibold text-ink">{{ title }}</strong>
      <span v-if="message" class="text-xs text-muted">{{ message }}</span>
      <slot />
    </div>
    <button type="button" class="w-auto bg-transparent border-0 text-lg leading-none text-muted cursor-pointer p-0 hover:text-ink" aria-label="Dismiss" @click="open = false">×</button>
  </div>
</template>
