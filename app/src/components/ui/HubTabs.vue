<script setup lang="ts">
// Compact IA: a hub page groups related screens as tabs instead of separate
// deck cards. Purely presentational — the parent owns the active tab.
// Keyboard contract: roving tabindex (active 0, rest -1); Left/Right/Home/End
// move selection via update:modelValue and focus the newly active tab.
import { nextTick } from 'vue'

export interface HubTab {
  id: string
  label: string
}

const props = withDefaults(
  defineProps<{ tabs: HubTab[]; modelValue: string; controlsPrefix?: string; idPrefix?: string }>(),
  { controlsPrefix: '', idPrefix: 'hubtab' },
)
const emit = defineEmits<{ 'update:modelValue': [string] }>()

function controlsId(id: string): string | undefined {
  return props.controlsPrefix ? `${props.controlsPrefix}-${id}` : undefined
}

function onKey(e: KeyboardEvent, index: number): void {
  if (!props.tabs.length) return
  let next = -1
  if (e.key === 'ArrowRight') next = (index + 1) % props.tabs.length
  else if (e.key === 'ArrowLeft') next = (index - 1 + props.tabs.length) % props.tabs.length
  else if (e.key === 'Home') next = 0
  else if (e.key === 'End') next = props.tabs.length - 1
  else return
  e.preventDefault()
  const id = props.tabs[next]?.id
  if (id === undefined) return
  emit('update:modelValue', id)
  void nextTick(() => {
    document.getElementById(`${props.idPrefix}-${id}`)?.focus()
  })
}
</script>

<template>
  <div class="hubtabs flex justify-center m-0" role="tablist">
    <!-- Segmented Toggle capsule matching Navbar toggle design -->
    <div class="inline-flex items-center gap-1 p-1 bg-wash border border-line rounded-full shadow-2xs max-w-full overflow-x-auto">
      <button
        v-for="(t, i) in tabs"
        :key="t.id"
        :id="`${idPrefix}-${t.id}`"
        type="button"
        role="tab"
        class="w-auto flex-none min-h-11 inline-flex items-center justify-center text-sm font-medium tracking-[0.01em] px-4 py-2 rounded-full border border-transparent cursor-pointer transition-all select-none whitespace-nowrap"
        :class="modelValue === t.id
          ? 'active bg-ink text-surface font-semibold shadow-xs border-ink'
          : 'text-muted hover:text-ink hover:bg-surface/50'"
        :aria-selected="modelValue === t.id"
        :aria-controls="controlsId(t.id)"
        :tabindex="modelValue === t.id ? 0 : -1"
        @click="emit('update:modelValue', t.id)"
        @keydown="onKey($event, i)"
      >
        {{ t.label }}
      </button>
    </div>
  </div>
</template>
