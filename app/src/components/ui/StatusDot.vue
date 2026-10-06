<script setup lang="ts">
// StatusDot — small on/off indicator.
defineProps<{ on?: boolean; title?: string }>()
</script>

<template>
  <span class="p-dot" :class="{ on }" :title="title" aria-hidden="true" />
</template>

<style scoped>
.p-dot { display: inline-block; width: 8px; height: 8px; border-radius: 999px; background: var(--faint, #a3a3a3); }
.p-dot.on { background: var(--ink, #000); }
</style>
