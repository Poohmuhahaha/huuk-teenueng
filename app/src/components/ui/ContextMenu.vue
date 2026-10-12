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
  <div ref="root" class="w-full" @contextmenu="onContext">
    <slot />
    <div
      v-if="open"
      class="fixed z-[1000] min-w-[180px] bg-surface border border-ink p-1 shadow-[4px_4px_0_var(--color-ink)] rounded-none flex flex-col"
      :style="{ left: `${x}px`, top: `${y}px` }"
      role="menu"
    >
      <template v-for="item in items" :key="item.id">
        <div v-if="item.separatorBefore" class="h-px bg-line my-1" role="separator" />
        <button
          type="button"
          class="block w-full bg-transparent border-0 text-ink font-sans text-[13px] p-2 px-2.5 text-left cursor-pointer rounded-none hover:bg-wash disabled:opacity-40 disabled:cursor-not-allowed"
          :class="{ 'font-bold text-danger': item.danger }"
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
