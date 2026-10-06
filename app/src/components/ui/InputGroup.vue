<script setup lang="ts">
defineProps<{ prefix?: string; suffix?: string; disabled?: boolean }>()
</script>

<template>
  <div class="p-igroup" :class="{ disabled }">
    <span v-if="prefix || $slots.prefix" class="p-igroup-affix"><slot name="prefix">{{ prefix }}</slot></span>
    <div class="p-igroup-main"><slot /></div>
    <span v-if="suffix || $slots.suffix" class="p-igroup-affix"><slot name="suffix">{{ suffix }}</slot></span>
  </div>
</template>

<style scoped>
.p-igroup {
  display: flex;
  align-items: stretch;
  border: 1px solid var(--line, #d9d9d9);
  background: var(--paper, #fff);
  width: 100%;
}
.p-igroup:focus-within { border-color: var(--ink, #000); }
.p-igroup.disabled { opacity: 0.5; }
.p-igroup-affix {
  display: inline-flex;
  align-items: center;
  padding: 0 12px;
  background: var(--wash, #f6f6f6);
  color: var(--muted, #6b6b6b);
  font-size: 13px;
  white-space: nowrap;
}
.p-igroup-affix:first-child { border-right: 1px solid var(--line, #d9d9d9); }
.p-igroup-affix:last-child { border-left: 1px solid var(--line, #d9d9d9); }
.p-igroup-main { flex: 1; min-width: 0; display: flex; }
.p-igroup-main :deep(input),
.p-igroup-main :deep(textarea),
.p-igroup-main :deep(select) { border: 0 !important; background: transparent; }
.p-igroup-main :deep(input:focus) { outline: none; }
</style>
