<script setup lang="ts">
const model = defineModel<string>({ default: '' })
withDefaults(
  defineProps<{ type?: string; placeholder?: string; disabled?: boolean; invalid?: boolean; id?: string; size?: 'sm' | 'md' | 'lg' }>(),
  { type: 'text', size: 'md' },
)

const sizeClasses: Record<string, string> = {
  sm: 'px-2.5 py-1.5 text-[13px]',
  md: 'px-3 py-[9px] text-sm',
  lg: 'px-3.5 py-3 text-[15px]',
}
</script>

<template>
  <input
    :id="id"
    v-model="model"
    class="w-full bg-surface border border-line text-ink font-sans transition-colors rounded-none placeholder:text-faint focus:outline-none focus:border-ink disabled:opacity-50 disabled:cursor-not-allowed"
    :class="[sizeClasses[size] || sizeClasses.md, invalid ? 'border-ink bg-wash' : '']"
    :type="type"
    :placeholder="placeholder"
    :disabled="disabled"
    :aria-invalid="invalid || undefined"
  />
</template>
