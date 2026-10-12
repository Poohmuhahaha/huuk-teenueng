<script setup lang="ts">
import { ref } from 'vue'

const open = defineModel<boolean>({ default: false })
const title = defineModel<string>('title')
const expanded = ref(false)
withDefaults(
  defineProps<{ description?: string; width?: string; expandable?: boolean; editableTitle?: boolean }>(),
  { width: '480px' },
)
</script>

<template>
  <div
    v-if="open"
    class="fixed inset-0 z-[1000] bg-black/40 flex justify-center"
    :class="expanded ? 'p-6 items-center overflow-hidden' : 'p-4 pt-[10vh] items-start overflow-y-auto'"
    @click.self="open = false"
  >
    <div
      class="w-full bg-surface border border-line shadow-[0_12px_40px_rgba(0,0,0,0.08)] p-7 sm:p-8 rounded-none transition-all duration-200"
      :class="expanded ? 'flex flex-col w-[calc(100vw-48px)] max-w-none h-[calc(100vh-48px)]' : ''"
      role="dialog"
      aria-modal="true"
      :style="{ maxWidth: expanded ? undefined : width }"
    >
      <div class="flex items-start justify-between gap-3 mb-5">
        <div class="flex flex-col flex-1 min-w-0">
          <input
            v-if="editableTitle"
            v-model="title"
            class="w-full bg-transparent border-0 border-b border-transparent p-0 m-0 text-ink font-display text-2xl leading-tight tracking-tight rounded-none outline-none placeholder:text-faint focus:border-ink"
            aria-label="Title"
            placeholder="Untitled"
          />
          <h3 v-else-if="title" class="m-0 text-2xl leading-tight tracking-tight font-display text-ink">{{ title }}</h3>
          <p v-if="description" class="m-0 mt-1.5 text-sm text-muted">{{ description }}</p>
        </div>
        <div class="flex items-center gap-2.5 shrink-0">
          <button
            v-if="expandable"
            type="button"
            class="w-auto inline-flex items-center bg-transparent border-0 text-muted cursor-pointer p-0 hover:text-ink"
            :aria-label="expanded ? 'Collapse' : 'Expand'"
            :title="expanded ? 'Collapse' : 'Expand'"
            @click="expanded = !expanded"
          >
            <svg
              v-if="expanded"
              width="16"
              height="16"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="2"
              stroke-linecap="round"
              stroke-linejoin="round"
            >
              <polyline points="4 14 10 14 10 20" />
              <polyline points="20 10 14 10 14 4" />
              <line x1="14" y1="10" x2="21" y2="3" />
              <line x1="3" y1="21" x2="10" y2="14" />
            </svg>
            <svg
              v-else
              width="16"
              height="16"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="2"
              stroke-linecap="round"
              stroke-linejoin="round"
            >
              <polyline points="15 3 21 3 21 9" />
              <polyline points="9 21 3 21 3 15" />
              <line x1="21" y1="3" x2="14" y2="10" />
              <line x1="3" y1="21" x2="10" y2="14" />
            </svg>
          </button>
          <button type="button" class="w-auto bg-transparent border-0 text-2xl leading-none text-muted cursor-pointer p-0 hover:text-ink" aria-label="Close" @click="open = false">×</button>
        </div>
      </div>
      <div :class="expanded ? 'flex flex-col flex-1 min-h-0 overflow-auto' : ''"><slot /></div>
      <div v-if="$slots.footer" class="flex justify-end gap-2.5 mt-6 pt-4.5 border-t border-line shrink-0"><slot name="footer" /></div>
    </div>
  </div>
</template>
