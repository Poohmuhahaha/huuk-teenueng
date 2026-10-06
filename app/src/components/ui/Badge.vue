<script setup lang="ts">
withDefaults(defineProps<{ variant?: 'neutral' | 'success' | 'warning' | 'danger' | 'solid'; dot?: boolean }>(), {
  variant: 'neutral',
})
</script>

<template>
  <span class="p-badge" :class="`v-${variant}`">
    <span v-if="dot" class="p-badge-dot" aria-hidden="true" />
    <slot />
  </span>
</template>

<style scoped>
.p-badge {
  display: inline-flex;
  align-items: center;
  gap: 5px;
  border: 1px solid var(--line, #d9d9d9);
  background: var(--paper, #fff);
  color: var(--ink, #000);
  font-size: 11px;
  font-weight: 600;
  letter-spacing: 0.04em;
  text-transform: uppercase;
  padding: 2px 8px;
  white-space: nowrap;
}
.p-badge-dot { width: 5px; height: 5px; border-radius: 999px; background: currentColor; }
.v-success { border-style: double; }
.v-warning { border-style: dashed; }
.v-danger { background: var(--ink, #000); color: var(--paper, #fff); border-color: var(--ink, #000); }
.v-solid { background: var(--ink, #000); color: var(--paper, #fff); border-color: var(--ink, #000); }
</style>
