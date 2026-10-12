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
  <div ref="root" class="flex items-stretch w-full border border-line rounded-none select-none">
    <div class="min-w-0 overflow-auto p-3" :style="{ width: `${pct}%` }"><slot name="a" /></div>
    <div
      class="w-1.5 shrink-0 cursor-col-resize bg-wash border-x border-line transition-colors hover:bg-ink"
      :class="{ 'bg-ink': dragging }"
      role="separator"
      aria-orientation="vertical"
      @pointerdown="dragging = true"
      @pointermove="onMove"
      @pointerup="dragging = false"
      @pointercancel="dragging = false"
    />
    <div class="flex-1 min-w-0 overflow-auto p-3"><slot name="b" /></div>
  </div>
</template>
