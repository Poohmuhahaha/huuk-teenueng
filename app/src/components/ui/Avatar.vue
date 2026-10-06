<script setup lang="ts">
// Avatar — square initials chip (ported from Pugu ui/Avatar).
defineProps<{ label?: string; more?: boolean }>()
</script>

<template>
  <span class="p-avatar" :class="{ more }"><slot>{{ label }}</slot></span>
</template>

<style scoped>
.p-avatar {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 30px;
  height: 30px;
  border: 1px solid var(--line, #e4e4e4);
  background: var(--paper, #fff);
  font-size: 12px;
  font-weight: 600;
  color: var(--ink, #000);
  flex: none;
}
.p-avatar.more { background: var(--ink, #000); color: var(--paper, #fff); border-color: var(--ink, #000); }
</style>
