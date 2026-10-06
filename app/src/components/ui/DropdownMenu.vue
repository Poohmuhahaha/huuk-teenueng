<script setup lang="ts">
import { ref } from 'vue'
import { useDismiss } from '@/composables/useDismiss'
import type { MenuItem } from '@/core/menu'

const open = defineModel<boolean>({ default: false })
withDefaults(defineProps<{ items?: MenuItem[]; align?: 'start' | 'end' }>(), { items: () => [], align: 'start' })
const emit = defineEmits<{ select: [string] }>()
const root = ref<HTMLElement | null>(null)
useDismiss(root, () => (open.value = false))

function pick(item: MenuItem): void {
  if (item.disabled) return
  open.value = false
  emit('select', item.id)
}
</script>

<template>
  <div ref="root" class="p-menu">
    <span class="p-menu-trigger" @click="open = !open"><slot name="trigger" /></span>
    <div v-if="open" class="p-menu-panel" :class="align" role="menu">
      <template v-for="item in items" :key="item.id">
        <div v-if="item.separatorBefore" class="p-menu-sep" role="separator" />
        <button
          type="button"
          class="p-menu-item"
          :class="{ danger: item.danger }"
          role="menuitem"
          :disabled="item.disabled"
          @click="pick(item)"
        >
          {{ item.label }}
        </button>
      </template>
    </div>
  </div>
</template>

<style scoped>
.p-menu { position: relative; display: inline-flex; }
.p-menu-trigger { display: inline-flex; cursor: pointer; }
.p-menu-panel {
  position: absolute;
  z-index: 60;
  top: calc(100% + 6px);
  min-width: 190px;
  background: var(--paper, #fff);
  border: 1px solid var(--line, #e4e4e4);
  border-radius: 10px;
  padding: 6px;
  box-shadow: 0 12px 40px rgba(0, 0, 0, 0.12);
}
.p-menu-panel.start { left: 0; }
.p-menu-panel.end { right: 0; }
.p-menu-sep { height: 1px; background: var(--line, #d9d9d9); margin: 4px 0; }
.p-menu-item {
  display: block;
  width: 100%;
  background: transparent;
  border: 0;
  border-radius: 8px;
  color: var(--ink, #000);
  font: inherit;
  font-size: 13px;
  padding: 8px 10px;
  text-align: left;
  cursor: pointer;
}
.p-menu-item:hover { background: var(--wash, #f6f6f6); }
.p-menu-item.danger { color: var(--ink, #000); font-weight: 700; }
.p-menu-item:disabled { opacity: 0.4; cursor: not-allowed; }
</style>
