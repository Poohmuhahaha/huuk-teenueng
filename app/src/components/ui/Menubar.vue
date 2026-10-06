<script setup lang="ts">
import { ref } from 'vue'
import { useDismiss } from '@/composables/useDismiss'
import type { MenuItem } from '@/core/menu'

withDefaults(defineProps<{ menus?: { id: string; label: string; items: MenuItem[] }[] }>(), { menus: () => [] })
const emit = defineEmits<{ select: [string, string] }>()
const openId = ref<string | null>(null)
const root = ref<HTMLElement | null>(null)
useDismiss(root, () => (openId.value = null))

function toggle(id: string): void {
  openId.value = openId.value === id ? null : id
}

function hover(id: string): void {
  if (openId.value) openId.value = id
}

function pick(menuId: string, item: MenuItem): void {
  if (item.disabled) return
  openId.value = null
  emit('select', menuId, item.id)
}
</script>

<template>
  <div ref="root" class="p-menubar" role="menubar">
    <div v-for="m in menus" :key="m.id" class="p-menubar-item">
      <button
        type="button"
        class="p-menubar-trigger"
        :class="{ open: openId === m.id }"
        :aria-expanded="openId === m.id"
        @click="toggle(m.id)"
        @mouseenter="hover(m.id)"
      >
        {{ m.label }}
      </button>
      <div v-if="openId === m.id" class="p-menubar-menu" role="menu">
        <button
          v-for="item in m.items"
          :key="item.id"
          type="button"
          class="p-menubar-item-btn"
          role="menuitem"
          :disabled="item.disabled"
          @click="pick(m.id, item)"
        >
          {{ item.label }}
        </button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.p-menubar { display: inline-flex; border: 1px solid var(--line, #d9d9d9); background: var(--paper, #fff); }
.p-menubar-item { position: relative; }
.p-menubar-item + .p-menubar-item { border-left: 1px solid var(--line, #d9d9d9); }
.p-menubar-trigger {
  width: auto;
  background: transparent;
  border: 0;
  color: var(--ink, #000);
  font: inherit;
  font-size: 13px;
  font-weight: 600;
  padding: 7px 12px;
  cursor: pointer;
}
.p-menubar-trigger:hover,
.p-menubar-trigger.open { background: var(--wash, #f6f6f6); }
.p-menubar-menu {
  position: absolute;
  z-index: 60;
  top: 100%;
  left: 0;
  min-width: 180px;
  background: var(--paper, #fff);
  border: 1px solid var(--ink, #000);
  padding: 4px;
  box-shadow: 4px 4px 0 var(--ink, #000);
}
.p-menubar-item-btn {
  display: block;
  width: 100%;
  background: transparent;
  border: 0;
  color: var(--ink, #000);
  font: inherit;
  font-size: 13px;
  padding: 8px 10px;
  text-align: left;
  cursor: pointer;
}
.p-menubar-item-btn:hover { background: var(--wash, #f6f6f6); }
.p-menubar-item-btn:disabled { opacity: 0.4; cursor: not-allowed; }
</style>
