<script setup lang="ts">
import { ref } from 'vue'

const props = withDefaults(defineProps<{ initial?: number; min?: number }>(), { initial: 50, min: 10 })
const pct = ref(props.initial)
const dragging = ref(false)
const root = ref<HTMLElement | null>(null)

function onMove(event: PointerEvent): void {
  if (!dragging.value || !root.value) return
  const rect = root.value.getBoundingClientRect()
  const next = ((event.clientX - rect.left) / rect.width) * 100
  pct.value = Math.min(100 - props.min, Math.max(props.min, next))
}
</script>

<template>
  <div ref="root" class="p-resizable" :class="{ dragging }">
    <div class="p-resize-pane" :style="{ width: `${pct}%` }"><slot name="a" /></div>
    <div
      class="p-resize-handle"
      role="separator"
      aria-orientation="vertical"
      @pointerdown="dragging = true"
      @pointermove="onMove"
      @pointerup="dragging = false"
      @pointercancel="dragging = false"
    />
    <div class="p-resize-pane p-resize-b"><slot name="b" /></div>
  </div>
</template>

<style scoped>
.p-resizable { display: flex; align-items: stretch; width: 100%; border: 1px solid var(--line, #d9d9d9); }
.p-resize-pane { min-width: 0; overflow: auto; padding: 12px; }
.p-resize-b { flex: 1; border-left: 0; }
.p-resize-handle {
  width: 6px;
  flex: none;
  cursor: col-resize;
  background: var(--wash, #f6f6f6);
  border-left: 1px solid var(--line, #d9d9d9);
  border-right: 1px solid var(--line, #d9d9d9);
}
.p-resize-handle:hover, .p-resizable.dragging .p-resize-handle { background: var(--ink, #000); }
</style>
