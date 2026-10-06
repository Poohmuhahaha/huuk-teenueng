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
  }
}
</script>

<template>
  <div v-if="open" class="p-cmd-backdrop" @click.self="open = false">
    <div class="p-cmd" role="dialog" aria-modal="true" aria-label="Command palette">
      <input
        ref="input"
        v-model="query"
        class="p-cmd-input"
        :placeholder="placeholder"
        @keydown="onKeydown"
      />
      <div class="p-cmd-list">
        <button
          v-for="(item, i) in filtered"
          :key="item.id"
          type="button"
          class="p-cmd-item"
          :class="{ active: i === index }"
          @mouseenter="index = i"
          @click="pick(item)"
        >
          <span>
            <span v-if="item.group" class="p-cmd-group">{{ item.group }}</span>
            {{ item.label }}
          </span>
          <kbd v-if="item.shortcut" class="p-cmd-kbd">{{ item.shortcut }}</kbd>
        </button>
        <p v-if="!filtered.length" class="p-cmd-empty">No results</p>
      </div>
    </div>
  </div>
</template>

<style scoped>
.p-cmd-backdrop {
  position: fixed;
  inset: 0;
  z-index: 1000;
  background: rgba(0, 0, 0, 0.4);
  display: flex;
  align-items: flex-start;
  justify-content: center;
  padding-top: 12vh;
}
.p-cmd {
  width: 90%;
  max-width: 560px;
  background: var(--paper, #fff);
  border: 1px solid var(--line, #e4e4e4);
  border-radius: 12px;
  box-shadow: 0 12px 40px rgba(0, 0, 0, 0.08);
  overflow: hidden;
}
.p-cmd-input {
  width: 100%;
  border: 0;
  border-bottom: 1px solid var(--line, #e4e4e4);
  background: transparent;
  color: var(--ink, #000);
  font: inherit;
  font-size: 15px;
  padding: 14px 16px;
}
.p-cmd-input:focus { outline: none; }
.p-cmd-list { max-height: 320px; overflow-y: auto; padding: 6px; }
.p-cmd-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  width: 100%;
  background: transparent;
  border: 0;
  color: var(--ink, #000);
  font: inherit;
  font-size: 13px;
  padding: 9px 10px;
  border-radius: 8px;
  cursor: pointer;
  text-align: left;
}
.p-cmd-item.active { background: var(--ink, #000); color: var(--paper, #fff); }
.p-cmd-group { font-size: 11px; text-transform: uppercase; letter-spacing: 0.06em; opacity: 0.6; margin-right: 8px; }
.p-cmd-kbd { font-family: var(--mono, ui-monospace, Menlo, monospace); font-size: 10px; border: 1px solid currentColor; border-radius: 4px; padding: 1px 5px; }
.p-cmd-empty { margin: 10px; font-size: 12px; color: var(--muted, #6b6b6b); }
</style>
