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
  <aside class="p-sidebar" :class="{ collapsed }">
    <div class="p-sidebar-head">
      <strong v-if="!collapsed" class="p-sidebar-title">{{ title }}</strong>
      <button
        type="button"
        class="p-sidebar-toggle"
        aria-label="Toggle sidebar"
        :aria-expanded="!collapsed"
        @click="collapsed = !collapsed"
      >
        ☰
      </button>
    </div>
    <nav class="p-sidebar-nav">
      <template v-for="it in items" :key="it.id">
        <a
          v-if="it.to"
          class="p-sidebar-item"
          :class="{ active: model === it.id }"
          :href="it.to"
          :title="it.label"
          :aria-current="model === it.id ? 'page' : undefined"
          @click="onNav(it, $event)"
        >
          <span class="p-sidebar-icon" aria-hidden="true">{{ it.icon ?? '·' }}</span>
          <span v-if="!collapsed" class="p-sidebar-label">{{ it.label }}</span>
        </a>
        <button
          v-else
          type="button"
          class="p-sidebar-item"
          :class="{ active: model === it.id }"
          :title="it.label"
          @click="model = it.id"
        >
          <span class="p-sidebar-icon" aria-hidden="true">{{ it.icon ?? '·' }}</span>
          <span v-if="!collapsed" class="p-sidebar-label">{{ it.label }}</span>
        </button>
      </template>
    </nav>
  </aside>
</template>

<style scoped>
.p-sidebar {
  display: flex;
  flex-direction: column;
  width: 220px;
  background: var(--paper, #fff);
  border: 1px solid var(--line, #d9d9d9);
  transition: width 0.15s;
}
.p-sidebar.collapsed { width: 52px; }
.p-sidebar-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
  padding: 10px 12px;
  border-bottom: 1px solid var(--line, #d9d9d9);
}
.p-sidebar-title { font-size: 13px; letter-spacing: 0.06em; text-transform: uppercase; }
.p-sidebar-toggle {
  width: auto;
  background: transparent;
  border: 0;
  color: var(--muted, #6b6b6b);
  font-size: 14px;
  cursor: pointer;
  padding: 0;
}
.p-sidebar-toggle:hover { color: var(--ink, #000); }
.p-sidebar-nav { display: flex; flex-direction: column; padding: 6px; gap: 2px; }
.p-sidebar-item {
  display: flex;
  align-items: center;
  gap: 10px;
  width: 100%;
  background: transparent;
  border: 0;
  color: var(--muted, #6b6b6b);
  font: inherit;
  font-size: 13px;
  font-weight: 600;
  padding: 8px 10px;
  cursor: pointer;
  text-align: left;
  white-space: nowrap;
  text-decoration: none;
}
.p-sidebar-item:hover { background: var(--wash, #f6f6f6); color: var(--ink, #000); text-decoration: none; }
.p-sidebar-item.active { background: var(--ink, #000); color: var(--paper, #fff); }
.p-sidebar-icon { width: 16px; text-align: center; flex: none; }
</style>
