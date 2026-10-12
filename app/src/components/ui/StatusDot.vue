<script setup lang="ts">
// StatusDot — small on/off indicator.
defineProps<{ on?: boolean; title?: string }>()
</script>

<template>
  <span
    class="inline-block w-2 h-2 rounded-full"
    :class="on ? 'bg-ink' : 'bg-faint'"
    :title="title"
    aria-hidden="true"
  />
</template>
