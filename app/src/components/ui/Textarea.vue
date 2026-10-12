<script setup lang="ts">
const model = defineModel<string>({ default: '' })
withDefaults(defineProps<{ placeholder?: string; rows?: number; disabled?: boolean; invalid?: boolean }>(), { rows: 3 })
</script>

<template>
  <textarea
    v-model="model"
    class="w-full bg-surface border border-line text-ink font-sans text-sm p-3 py-[9px] resize-y min-h-[72px] transition-colors rounded-none placeholder:text-faint focus:outline-none focus:border-ink disabled:opacity-50 disabled:cursor-not-allowed"
    :class="{ 'border-ink bg-wash': invalid }"
    :rows="rows"
    :placeholder="placeholder"
    :disabled="disabled"
  />
</template>
