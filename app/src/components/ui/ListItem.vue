<script setup lang="ts">
withDefaults(defineProps<{ title?: string; description?: string; active?: boolean; disabled?: boolean }>(), {})
</script>

<template>
  <div
    class="flex items-center gap-2.5 w-full p-2.5 px-3 border bg-transparent text-ink rounded-none transition-colors"
    :class="[
      active ? 'border-ink bg-wash' : 'border-transparent',
      disabled ? 'opacity-45 cursor-not-allowed' : ''
    ]"
  >
    <span v-if="$slots.leading" class="inline-flex items-center shrink-0"><slot name="leading" /></span>
    <span class="flex flex-col gap-0.5 flex-1 min-w-0 text-left">
      <strong v-if="title" class="text-sm font-semibold">{{ title }}</strong>
      <span v-if="description" class="text-xs text-muted">{{ description }}</span>
      <slot />
    </span>
    <span v-if="$slots.trailing" class="inline-flex items-center shrink-0"><slot name="trailing" /></span>
  </div>
</template>
