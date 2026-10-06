<script setup lang="ts">
withDefaults(defineProps<{ width?: string }>(), { width: '280px' })
</script>

<template>
  <span class="p-hover">
    <span class="p-hover-trigger"><slot name="trigger" /></span>
    <span class="p-hover-card" :style="{ width }">
      <slot />
    </span>
  </span>
</template>

<style scoped>
.p-hover { position: relative; display: inline-flex; }
.p-hover-trigger { display: inline-flex; }
.p-hover-card {
  position: absolute;
  left: 0;
  top: calc(100% + 8px);
  z-index: 60;
  background: var(--paper, #fff);
  border: 1px solid var(--ink, #000);
  padding: 14px;
  opacity: 0;
  visibility: hidden;
  transition: opacity 0.15s, visibility 0.15s;
}
.p-hover:hover .p-hover-card,
.p-hover:focus-within .p-hover-card { opacity: 1; visibility: visible; }
</style>
