<script setup lang="ts">
const open = defineModel<boolean>({ default: false })
defineProps<{ title?: string }>()
</script>

<template>
  <div class="p-collapse">
    <button type="button" class="p-collapse-trigger" :aria-expanded="open" @click="open = !open">
      <slot name="trigger">{{ title }}</slot>
      <span class="p-collapse-caret" :class="{ open }" aria-hidden="true">▸</span>
    </button>
    <div v-show="open" class="p-collapse-body">
      <slot />
    </div>
  </div>
</template>

<style scoped>
.p-collapse { width: 100%; }
.p-collapse-trigger {
  display: flex;
  align-items: center;
  gap: 8px;
  width: auto;
  background: transparent;
  border: 0;
  color: var(--ink, #000);
  font: inherit;
  font-size: 14px;
  font-weight: 600;
  padding: 4px 0;
  cursor: pointer;
  text-align: left;
}
.p-collapse-caret { color: var(--muted, #6b6b6b); transition: transform 0.15s; }
.p-collapse-caret.open { transform: rotate(90deg); }
.p-collapse-body { padding: 8px 0 0 20px; font-size: 13px; color: var(--muted, #6b6b6b); }
</style>
