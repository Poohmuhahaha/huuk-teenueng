<script setup lang="ts">
import { computed, nextTick, ref, watch } from 'vue'

interface CommandItem {
  id: string
  label: string
  group?: string
  shortcut?: string
}

const open = defineModel<boolean>({ default: false })
const props = withDefaults(defineProps<{ items?: CommandItem[]; placeholder?: string }>(), {
  items: () => [],
  placeholder: 'Type a command or search…',
})
const emit = defineEmits<{ select: [string] }>()
const query = ref('')
const index = ref(0)
const input = ref<HTMLInputElement | null>(null)

const filtered = computed(() => {
  const q = query.value.trim().toLowerCase()
  if (!q) return props.items
  return props.items.filter((item) => item.label.toLowerCase().includes(q))
})

const activeId = computed(() => {
  const item = filtered.value[index.value]
  return item ? `cmd-${item.id}` : undefined
})

watch(open, async (value) => {
  if (!value) return
  query.value = ''
  index.value = 0
  await nextTick()
  input.value?.focus()
})

watch(filtered, () => (index.value = 0))

function move(delta: number): void {
  if (!filtered.value.length) return
  index.value = (index.value + delta + filtered.value.length) % filtered.value.length
}

function pick(item: CommandItem): void {
  open.value = false
  emit('select', item.id)
}

function onKeydown(event: KeyboardEvent): void {
  if (event.key === 'ArrowDown') {
    event.preventDefault()
    move(1)
  } else if (event.key === 'ArrowUp') {
    event.preventDefault()
    move(-1)
  } else if (event.key === 'Enter') {
    const item = filtered.value[index.value]
    if (item) pick(item)
  } else if (event.key === 'Escape') {
    open.value = false
  }
}
</script>

<template>
  <div v-if="open" class="fixed inset-0 z-[1000] bg-black/40 flex items-start justify-center pt-[12vh] p-4" @click.self="open = false">
    <div class="w-[90%] max-w-[560px] bg-surface border border-line shadow-[0_12px_40px_rgba(0,0,0,0.08)] overflow-hidden rounded-none" role="dialog" aria-modal="true" aria-label="Command palette">
      <input
        ref="input"
        v-model="query"
        class="w-full border-0 border-b border-line bg-transparent text-ink font-sans text-[15px] p-3.5 px-4 outline-none rounded-none focus:outline-none focus-visible:outline-2 focus-visible:outline-ink focus-visible:-outline-offset-2"
        :placeholder="placeholder"
        role="combobox"
        aria-label="Command palette"
        aria-expanded="true"
        aria-controls="cmd-listbox"
        :aria-activedescendant="activeId"
        @keydown="onKeydown"
      />
      <div id="cmd-listbox" role="listbox" class="max-h-[320px] overflow-y-auto p-1.5">
        <button
          v-for="(item, i) in filtered"
          :key="item.id"
          :id="'cmd-' + item.id"
          type="button"
          role="option"
          :aria-selected="i === index"
          class="flex items-center justify-between gap-3 w-full bg-transparent border-0 font-sans text-[13px] p-2.5 px-2.5 rounded-none cursor-pointer text-left transition-colors"
          :class="i === index ? 'bg-ink text-surface' : 'text-ink hover:bg-wash'"
          @mouseenter="index = i"
          @click="pick(item)"
        >
          <span>
            <span v-if="item.group" class="text-[11px] uppercase tracking-[0.06em] opacity-60 mr-2">{{ item.group }}</span>
            {{ item.label }}
          </span>
          <kbd v-if="item.shortcut" class="font-mono text-[10px] border border-current rounded-none px-1 py-0.5">{{ item.shortcut }}</kbd>
        </button>
        <p v-if="!filtered.length" role="status" class="m-2.5 text-xs text-muted">No results</p>
      </div>
    </div>
  </div>
</template>
