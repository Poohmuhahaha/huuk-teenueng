<script setup lang="ts">
const model = defineModel<string>()
defineProps<{ tabs: { id: string; label: string; badge?: string | number; disabled?: boolean }[] }>()
</script>

<template>
  <div class="flex gap-1 border-b border-line w-full overflow-x-auto" role="tablist">
    <button
      v-for="t in tabs"
      :key="t.id"
      type="button"
      class="w-auto inline-flex items-center gap-1.5 bg-transparent border-0 border-b-2 font-sans text-sm font-semibold px-3 py-2 cursor-pointer whitespace-nowrap transition-colors rounded-none disabled:opacity-40 disabled:cursor-not-allowed hover:text-ink"
      :class="model === t.id ? 'border-b-ink text-ink' : 'border-b-transparent text-muted'"
      role="tab"
      :aria-selected="model === t.id"
      :disabled="t.disabled"
      @click="model = t.id"
    >
      <span>{{ t.label }}</span>
      <span
        v-if="t.badge !== undefined"
        class="text-[11px] font-semibold tracking-[0.04em] px-1.5 py-0.5 rounded-full"
        :class="model === t.id ? 'bg-ink text-surface' : 'bg-wash text-muted'"
      >{{ t.badge }}</span>
    </button>
  </div>
</template>
