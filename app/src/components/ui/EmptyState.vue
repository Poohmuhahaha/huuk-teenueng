<script setup lang="ts">
// EmptyState — dashed placeholder box with title / message / action slot.
defineProps<{ title?: string; message?: string }>()
</script>

<template>
  <div class="p-empty">
    <h3 v-if="title">{{ title }}</h3>
    <p v-if="message">{{ message }}</p>
    <slot />
  </div>
</template>

<style scoped>
.p-empty {
  border: 1px dashed var(--line, #e4e4e4);
  padding: 24px 20px;
  width: 100%;
  text-align: center;
  color: var(--muted, #6b6b6b);
  font-size: 13px;
}
.p-empty h3 { color: var(--ink, #000); margin: 0 0 4px; font-size: 16px; }
.p-empty p { margin: 0; }
</style>
