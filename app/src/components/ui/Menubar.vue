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
  <div ref="root" class="inline-flex border border-line bg-surface rounded-none" role="menubar">
    <div v-for="m in menus" :key="m.id" class="relative border-l border-line first:border-l-0">
      <button
        type="button"
        class="w-auto bg-transparent border-0 text-ink font-sans text-[13px] font-semibold py-1.5 px-3 cursor-pointer rounded-none transition-colors hover:bg-wash"
        :class="{ 'bg-wash': openId === m.id }"
        :aria-expanded="openId === m.id"
        @click="toggle(m.id)"
        @mouseenter="hover(m.id)"
      >
        {{ m.label }}
      </button>
      <div
        v-if="openId === m.id"
        class="absolute z-[60] top-full left-0 min-w-[180px] bg-surface border border-ink p-1 shadow-[4px_4px_0_var(--color-ink)] rounded-none flex flex-col"
        role="menu"
      >
        <button
          v-for="item in m.items"
          :key="item.id"
          type="button"
          class="block w-full bg-transparent border-0 text-ink font-sans text-[13px] p-2 px-2.5 text-left cursor-pointer rounded-none hover:bg-wash disabled:opacity-40 disabled:cursor-not-allowed"
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
