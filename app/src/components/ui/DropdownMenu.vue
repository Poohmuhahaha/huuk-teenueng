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
  <div ref="root" class="relative inline-flex">
    <button type="button" class="w-auto inline-flex items-center cursor-pointer bg-transparent border-0 p-0" :aria-haspopup="true" :aria-expanded="open" @click="open = !open"><slot name="trigger" /></button>
    <div v-if="open && items.length" class="absolute z-[60] top-[calc(100%+6px)] min-w-[190px] bg-surface border border-line rounded-none p-1.5 shadow-none flex flex-col" :class="align === 'end' ? 'right-0' : 'left-0'" role="menu">
      <template v-for="item in items" :key="item.id">
        <div v-if="item.separatorBefore" class="h-px bg-line my-1" role="separator" />
        <button
          type="button"
          class="block w-full bg-transparent border-0 rounded-none text-ink text-sm px-2.5 py-2 text-left cursor-pointer hover:bg-wash disabled:opacity-50 disabled:cursor-not-allowed"
          :class="item.danger ? 'font-bold text-danger' : ''"
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
