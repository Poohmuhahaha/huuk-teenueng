<script setup lang="ts">
import { ref } from 'vue'
import { useDismiss } from '@/composables/useDismiss'

const open = defineModel<boolean>({ default: false })
withDefaults(defineProps<{ align?: 'start' | 'end'; width?: string }>(), { align: 'start' })
const root = ref<HTMLElement | null>(null)
useDismiss(root, () => (open.value = false))
</script>

<template>
  <div ref="root" class="p-pop">
    <span class="p-pop-trigger" @click="open = !open"><slot name="trigger" /></span>
    <div v-if="open" class="p-pop-panel" :class="align" :style="width ? { width } : undefined">
      <slot />
    </div>
  </div>
</template>

<style scoped>
.p-pop { position: relative; display: inline-flex; }
.p-pop-trigger { display: inline-flex; cursor: pointer; }
.p-pop-panel {
  position: absolute;
  z-index: 60;
  top: calc(100% + 6px);
  min-width: 200px;
  background: var(--paper, #fff);
  border: 1px solid var(--ink, #000);
  padding: 12px;
  box-shadow: 4px 4px 0 var(--ink, #000);
}
.p-pop-panel.start { left: 0; }
.p-pop-panel.end { right: 0; }
</style>
