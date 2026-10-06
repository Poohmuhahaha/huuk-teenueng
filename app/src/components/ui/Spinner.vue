<script setup lang="ts">
withDefaults(defineProps<{ size?: number; label?: string }>(), { size: 16, label: 'Loading' })
</script>

<template>
  <span
    class="p-spinner"
    role="status"
    :aria-label="label"
    :style="{ width: `${size}px`, height: `${size}px` }"
  />
</template>

<style scoped>
.p-spinner {
  display: inline-block;
  border: 2px solid var(--wash, #f6f6f6);
  border-top-color: var(--ink, #000);
  border-radius: 999px;
  animation: p-spin 0.7s linear infinite;
}
@keyframes p-spin { to { transform: rotate(360deg); } }
</style>
