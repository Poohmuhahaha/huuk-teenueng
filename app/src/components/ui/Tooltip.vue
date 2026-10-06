<script setup lang="ts">
withDefaults(defineProps<{ text: string; side?: 'top' | 'bottom' }>(), { side: 'top' })
</script>

<template>
  <span class="p-tip" tabindex="0">
    <slot />
    <span class="p-tip-bubble" :class="side" role="tooltip">{{ text }}</span>
  </span>
</template>

<style scoped>
.p-tip { position: relative; display: inline-flex; }
.p-tip-bubble {
  position: absolute;
  left: 50%;
  transform: translateX(-50%);
  z-index: 60;
  background: var(--ink, #000);
  color: var(--paper, #fff);
  font-size: 11px;
  font-weight: 500;
  padding: 4px 8px;
  white-space: nowrap;
  opacity: 0;
  pointer-events: none;
  transition: opacity 0.12s;
}
.p-tip-bubble.top { bottom: calc(100% + 6px); }
.p-tip-bubble.bottom { top: calc(100% + 6px); }
.p-tip:hover .p-tip-bubble,
.p-tip:focus-visible .p-tip-bubble { opacity: 1; }
</style>
