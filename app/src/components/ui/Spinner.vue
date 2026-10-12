<script setup lang="ts">
withDefaults(defineProps<{ size?: number; label?: string }>(), { size: 16, label: 'Loading' })
</script>

<template>
  <span
    class="inline-block border-2 border-wash border-t-ink rounded-full animate-spin"
    role="status"
    :aria-label="label"
    :style="{ width: `${size}px`, height: `${size}px` }"
  />
</template>
