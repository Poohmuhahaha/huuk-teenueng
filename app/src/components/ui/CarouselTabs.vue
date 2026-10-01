<script setup lang="ts">
import { ref } from 'vue'

// M0 snap carousel: click, slide (touch + mouse drag), or arrow keys.
// One gesture advances exactly one screen (04 §M0).
const props = defineProps<{ items: (string | number)[]; modelValue: string | number }>()
const emit = defineEmits<{ 'update:modelValue': [v: string | number] }>()

const track = ref<HTMLElement | null>(null)
let startX = 0
let startScroll = 0
let dragging = false
let moved = false

function step(dir: 1 | -1): void {
  const items = props.items
  const i = items.indexOf(props.modelValue)
  const n = i < 0 ? (dir > 0 ? 0 : items.length - 1) : Math.min(items.length - 1, Math.max(0, i + dir))
  emit('update:modelValue', items[n] as string | number)
}

/** Centers a tab inside the tab strip only — never scrolls the page or deck. */
function centerTab(el: HTMLElement | null): void {
  const strip = track.value
  if (!el || !strip) return
  const left = Math.max(0, el.offsetLeft - (strip.clientWidth - el.offsetWidth) / 2)
  if (window.matchMedia?.('(prefers-reduced-motion: reduce)').matches) {
    strip.scrollLeft = left
    return
  }
  if (typeof strip.scrollTo === 'function') strip.scrollTo({ left, behavior: 'smooth' })
  else strip.scrollLeft = left
}

function pick(it: string | number, e: Event): void {
  emit('update:modelValue', it)
  centerTab(e.currentTarget as HTMLElement | null)
}

function onDown(e: PointerEvent): void {
  // Touch scrolls natively (momentum + snap); the custom step-drag is for
  // mouse/pen only.
  if (e.pointerType === 'touch') return
  dragging = true
  moved = false
  startX = e.clientX
  const el = track.value
  if (el) startScroll = el.scrollLeft
}

function onMove(e: PointerEvent): void {
  const el = track.value
  if (!dragging || !el) return
  const dx = e.clientX - startX
  if (!moved && Math.abs(dx) > 8) {
    moved = true
    // Capture only once a drag starts: capturing on pointerdown makes Chrome
    // retarget the following `click` to the track, so tab clicks never fire.
    try { el.setPointerCapture(e.pointerId) } catch { /* happy-dom / no-op */ }
  }
  if (moved) el.scrollLeft = startScroll - dx
}

function settle(): void {
  requestAnimationFrame(() => {
    centerTab(track.value?.querySelector<HTMLElement>('.tab.active') ?? null)
  })
}

function onUp(e: PointerEvent): void {
  const el = track.value
  if (!dragging || !el) return
  dragging = false
  const dx = e.clientX - startX
  if (!moved) return
  if (dx < -40) step(1)
  else if (dx > 40) step(-1)
  settle()
  window.setTimeout(() => { moved = false }, 0)
}

function onClickCapture(e: Event): void {
  // A drag ending on a button must not trigger it.
  if (moved) {
    e.preventDefault()
    e.stopPropagation()
    moved = false
  }
}

function onKey(e: KeyboardEvent): void {
  if (e.key === 'ArrowRight') step(1)
  else if (e.key === 'ArrowLeft') step(-1)
}
</script>

<template>
  <div ref="track" class="tabs" role="tablist" tabindex="0"
    @pointerdown="onDown" @pointermove="onMove" @pointerup="onUp" @pointercancel="onUp"
    @click.capture="onClickCapture" @keydown="onKey">
    <button
      v-for="it in items" :key="it" class="tab"
      :class="{ active: it === modelValue }"
      @click="pick(it, $event)"
    >
      {{ it }}
    </button>
  </div>
</template>

