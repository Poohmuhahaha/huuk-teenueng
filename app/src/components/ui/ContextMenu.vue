<script setup lang="ts">
import { ref } from 'vue'
import { useDismiss } from '@/composables/useDismiss'
import type { MenuItem } from '@/core/menu'

withDefaults(defineProps<{ items?: MenuItem[] }>(), { items: () => [] })
const emit = defineEmits<{ select: [string] }>()
const open = ref(false)
const x = ref(0)
const y = ref(0)
const root = ref<HTMLElement | null>(null)
useDismiss(root, () => (open.value = false))

function onContext(event: MouseEvent): void {
  event.preventDefault()
  x.value = event.clientX
  y.value = event.clientY
  open.value = true
}

function pick(item: MenuItem): void {
  if (item.disabled) return
  open.value = false
  emit('select', item.id)
}
</script>

<template>
  <div ref="root" class="p-ctx" @contextmenu="onContext">
    <slot />
    <div v-if="open" class="p-ctx-menu" :style="{ left: `${x}px`, top: `${y}px` }" role="menu">
      <template v-for="item in items" :key="item.id">
        <div v-if="item.separatorBefore" class="p-ctx-sep" role="separator" />
        <button
          type="button"
          class="p-ctx-item"
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
.p-ctx { width: 100%; }
.p-ctx-menu {
  position: fixed;
  z-index: 1000;
  min-width: 180px;
  background: var(--paper, #fff);
  border: 1px solid var(--ink, #000);
  padding: 4px;
  box-shadow: 4px 4px 0 var(--ink, #000);
}
.p-ctx-sep { height: 1px; background: var(--line, #d9d9d9); margin: 4px 0; }
.p-ctx-item {
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
.p-ctx-item:hover { background: var(--wash, #f6f6f6); }
.p-ctx-item.danger { font-weight: 700; }
.p-ctx-item:disabled { opacity: 0.4; cursor: not-allowed; }
</style>
