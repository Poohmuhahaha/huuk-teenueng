<script setup lang="ts">
defineProps<{ label: string; value: string | number }>()
</script>

<template>
  <div class="card" style="padding: 8px; text-align: center;">
    <div class="muted" style="font-size: 12px;">{{ label }}</div>
    <div style="font-size: 20px; font-weight: 800;">{{ value }}</div>
  </div>
</template>
