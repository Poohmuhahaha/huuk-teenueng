<script setup lang="ts">
const model = defineModel<boolean>({ default: false })
defineProps<{ label?: string; disabled?: boolean }>()
</script>

<template>
  <label class="inline-flex items-center gap-2.5 cursor-pointer text-sm text-ink" :class="{ 'opacity-50 cursor-not-allowed': disabled }">
    <button
      type="button"
      class="w-[38px] h-[22px] shrink-0 border border-ink p-0.5 inline-flex items-center transition-colors rounded-none focus-visible:outline-2 focus-visible:outline-ink focus-visible:outline-offset-2 disabled:cursor-not-allowed"
      :class="model ? 'bg-ink' : 'bg-surface'"
      role="switch"
      :aria-checked="model"
      :disabled="disabled"
      @click="model = !model"
    >
      <span
        class="w-3.5 h-3.5 transition-all duration-150 rounded-none"
        :class="model ? 'bg-surface translate-x-4' : 'bg-ink translate-x-0'"
      />
    </button>
    <span v-if="label || $slots.default" class="text-sm"><slot>{{ label }}</slot></span>
  </label>
</template>
