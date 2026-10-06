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
  <div v-if="open" class="p-dialog-backdrop" :class="{ expanded }" @click.self="open = false">
    <div
      class="p-dialog"
      :class="{ expanded }"
      role="dialog"
      aria-modal="true"
      :style="{ maxWidth: expanded ? undefined : width }"
    >
      <div class="p-dialog-head">
        <div class="p-dialog-titles">
          <input
            v-if="editableTitle"
            v-model="title"
            class="p-dialog-title-input"
            aria-label="Title"
            placeholder="Untitled"
          />
          <h3 v-else-if="title" class="p-dialog-title">{{ title }}</h3>
          <p v-if="description" class="p-dialog-desc">{{ description }}</p>
        </div>
        <div class="p-dialog-tools">
          <button
            v-if="expandable"
            type="button"
            class="p-dialog-tool"
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
          <button type="button" class="p-dialog-x" aria-label="Close" @click="open = false">×</button>
        </div>
      </div>
      <div class="p-dialog-body"><slot /></div>
      <div v-if="$slots.footer" class="p-dialog-foot"><slot name="footer" /></div>
    </div>
  </div>
</template>

<style scoped>
.p-dialog-backdrop {
  position: fixed;
  inset: 0;
  z-index: 1000;
  background: rgba(0, 0, 0, 0.4);
  display: flex;
  align-items: flex-start;
  justify-content: center;
  padding: 10vh 16px 16px;
  overflow-y: auto;
}
.p-dialog-backdrop.expanded {
  padding: 24px;
  align-items: center;
  overflow: hidden;
}
.p-dialog {
  width: 100%;
  background: var(--paper, #fff);
  border: 1px solid var(--line, #e4e4e4);
  border-radius: 12px;
  box-shadow: 0 12px 40px rgba(0, 0, 0, 0.08);
  padding: 28px 32px;
}
.p-dialog.expanded {
  display: flex;
  flex-direction: column;
  width: calc(100vw - 48px);
  max-width: none;
  height: calc(100vh - 48px);
}
.p-dialog.expanded .p-dialog-body {
  display: flex;
  flex-direction: column;
  flex: 1;
  min-height: 0;
  overflow: auto;
}
.p-dialog.expanded .p-dialog-foot { flex: none; }
.p-dialog-head {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 12px;
  margin-bottom: 20px;
}
.p-dialog-titles { display: flex; flex-direction: column; }
.p-dialog-title {
  margin: 0;
  font-size: 24px;
  line-height: 1.1;
  letter-spacing: -0.02em;
  text-transform: none;
  font-family: var(--font-display, Georgia, serif);
}
.p-dialog-title-input {
  width: 100%;
  background: transparent;
  border: 0;
  border-radius: 0;
  padding: 0;
  margin: 0;
  color: var(--ink, #000);
  font-size: 24px;
  line-height: 1.1;
  letter-spacing: -0.02em;
  font-family: var(--font-display, Georgia, serif);
  outline: none;
}
.p-dialog-title-input::placeholder { color: var(--faint, #a3a3a3); }
.p-dialog-title-input:focus { border-bottom: 1px solid var(--ink, #000); }
.p-dialog-desc { margin: 6px 0 0; font-size: 14px; color: var(--muted, #6b6b6b); }
.p-dialog-tools { display: flex; align-items: center; gap: 10px; }
.p-dialog-tool {
  width: auto;
  display: inline-flex;
  align-items: center;
  background: transparent;
  border: 0;
  color: var(--muted, #6b6b6b);
  cursor: pointer;
  padding: 0;
}
.p-dialog-tool:hover { color: var(--ink, #000); }
.p-dialog-x {
  width: auto;
  background: transparent;
  border: 0;
  font-size: 24px;
  line-height: 1;
  color: var(--muted, #6b6b6b);
  cursor: pointer;
  padding: 0;
}
.p-dialog-x:hover { color: var(--ink, #000); }
.p-dialog-foot {
  display: flex;
  justify-content: flex-end;
  gap: 10px;
  margin-top: 24px;
  padding-top: 18px;
  border-top: 1px solid var(--line, #e4e4e4);
}
</style>
