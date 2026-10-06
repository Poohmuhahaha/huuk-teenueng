<script setup lang="ts">
withDefaults(defineProps<{ title?: string; description?: string; active?: boolean; disabled?: boolean }>(), {})
</script>

<template>
  <div class="p-item" :class="{ active, disabled }">
    <span v-if="$slots.leading" class="p-item-lead"><slot name="leading" /></span>
    <span class="p-item-main">
      <strong v-if="title" class="p-item-title">{{ title }}</strong>
      <span v-if="description" class="p-item-desc">{{ description }}</span>
      <slot />
    </span>
    <span v-if="$slots.trailing" class="p-item-trail"><slot name="trailing" /></span>
  </div>
</template>

<style scoped>
.p-item {
  display: flex;
  align-items: center;
  gap: 10px;
  width: 100%;
  padding: 10px 12px;
  border: 1px solid transparent;
  background: transparent;
  color: var(--ink, #000);
}
.p-item.active { border-color: var(--ink, #000); background: var(--wash, #f6f6f6); }
.p-item.disabled { opacity: 0.45; }
.p-item-lead, .p-item-trail { display: inline-flex; align-items: center; flex: none; }
.p-item-main { display: flex; flex-direction: column; gap: 2px; flex: 1; min-width: 0; text-align: left; }
.p-item-title { font-size: 14px; font-weight: 600; }
.p-item-desc { font-size: 12px; color: var(--muted, #6b6b6b); }
</style>
