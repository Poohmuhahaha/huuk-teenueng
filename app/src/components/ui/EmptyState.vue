<script setup lang="ts">
// EmptyState — dashed placeholder box with title / message / action slot.
defineProps<{ title?: string; message?: string }>()
</script>

<template>
  <div class="border border-dashed border-line p-5 py-6 w-full text-center text-muted text-sm rounded-none">
    <h3 v-if="title" class="text-ink m-0 mb-1 text-base font-semibold">{{ title }}</h3>
    <p v-if="message" class="m-0">{{ message }}</p>
    <slot />
  </div>
</template>
