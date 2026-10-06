<script setup lang="ts">
defineProps<{ htmlFor?: string; required?: boolean }>()
</script>

<template>
  <label class="p-lbl" :for="htmlFor">
    <slot />
    <span v-if="required" class="p-lbl-req" aria-hidden="true">*</span>
  </label>
</template>

<style scoped>
.p-lbl {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  font-size: 12px;
  font-weight: 600;
  letter-spacing: 0.04em;
  text-transform: uppercase;
  color: var(--muted, #6b6b6b);
  width: 100%;
}
.p-lbl-req { color: var(--ink, #000); }
</style>
