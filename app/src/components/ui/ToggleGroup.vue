<script setup lang="ts">
const model = defineModel<string>()
defineProps<{ options: { value: string; label: string; disabled?: boolean }[] }>()
</script>

<template>
  <div class="inline-flex w-auto [&>*+*]:-ml-px" role="group">
    <button
      v-for="o in options"
      :key="o.value"
      type="button"
      class="w-auto inline-flex items-center border font-sans text-[13px] font-semibold px-3 py-1.5 cursor-pointer rounded-none transition-colors disabled:opacity-40 disabled:cursor-not-allowed hover:text-ink"
      :class="model === o.value ? 'bg-ink text-surface border-ink z-10' : 'bg-transparent border-line text-muted'"
      :aria-pressed="model === o.value"
      :disabled="o.disabled"
      @click="model = o.value"
    >
      {{ o.label }}
    </button>
  </div>
</template>
