<script setup lang="ts">
import { ref } from 'vue'

const model = defineModel<string>()
defineProps<{
  items: { id: string; label: string; icon?: string; to?: string }[]
  title?: string
}>()
const collapsed = ref(false)

function onNav(item: { id: string }, event: MouseEvent): void {
  if (event.metaKey || event.ctrlKey || event.shiftKey || event.altKey) return
  event.preventDefault()
  model.value = item.id
}
</script>

<template>
  <aside
    class="flex flex-col bg-surface border border-line rounded-none transition-[width] duration-150 shrink-0"
    :class="collapsed ? 'w-[52px]' : 'w-[220px]'"
  >
    <div class="flex items-center justify-between gap-2 p-2.5 px-3 border-b border-line">
      <strong v-if="!collapsed" class="text-[13px] tracking-[0.06em] uppercase text-ink font-bold">{{ title }}</strong>
      <button
        type="button"
        class="w-auto bg-transparent border-0 text-muted text-sm cursor-pointer p-0 hover:text-ink"
        aria-label="Toggle sidebar"
        :aria-expanded="!collapsed"
        @click="collapsed = !collapsed"
      >
        ☰
      </button>
    </div>
    <nav class="flex flex-col p-1.5 gap-0.5">
      <template v-for="it in items" :key="it.id">
        <a
          v-if="it.to"
          class="flex items-center gap-2.5 w-full bg-transparent border-0 font-sans text-[13px] font-semibold p-2 px-2.5 cursor-pointer text-left whitespace-nowrap no-underline rounded-none transition-colors"
          :class="model === it.id ? 'bg-ink text-surface' : 'text-muted hover:bg-wash hover:text-ink'"
          :href="it.to"
          :title="it.label"
          :aria-current="model === it.id ? 'page' : undefined"
          @click="onNav(it, $event)"
        >
          <span class="w-4 text-center shrink-0" aria-hidden="true">{{ it.icon ?? '·' }}</span>
          <span v-if="!collapsed" class="truncate">{{ it.label }}</span>
        </a>
        <button
          v-else
          type="button"
          class="flex items-center gap-2.5 w-full bg-transparent border-0 font-sans text-[13px] font-semibold p-2 px-2.5 cursor-pointer text-left whitespace-nowrap rounded-none transition-colors"
          :class="model === it.id ? 'bg-ink text-surface' : 'text-muted hover:bg-wash hover:text-ink'"
          :title="it.label"
          @click="model = it.id"
        >
          <span class="w-4 text-center shrink-0" aria-hidden="true">{{ it.icon ?? '·' }}</span>
          <span v-if="!collapsed" class="truncate">{{ it.label }}</span>
        </button>
      </template>
    </nav>
  </aside>
</template>
