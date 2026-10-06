<script setup lang="ts">
const model = defineModel<string>({ default: '' })
withDefaults(defineProps<{ placeholder?: string; rows?: number; disabled?: boolean; invalid?: boolean }>(), { rows: 3 })
</script>

<template>
  <textarea
    v-model="model"
    class="p-textarea"
    :class="{ invalid }"
    :rows="rows"
    :placeholder="placeholder"
    :disabled="disabled"
  />
</template>

<style scoped>
.p-textarea {
  background: var(--paper, #fff);
  border: 1px solid var(--line, #d9d9d9);
  color: var(--ink, #000);
  font: inherit;
  font-size: 14px;
  padding: 9px 12px;
  resize: vertical;
  min-height: 72px;
}
.p-textarea:focus { outline: none; border-color: var(--ink, #000); }
.p-textarea::placeholder { color: var(--faint, #a3a3a3); }
.p-textarea.invalid { border-color: var(--ink, #000); background: var(--wash, #f6f6f6); }
.p-textarea:disabled { opacity: 0.5; cursor: not-allowed; }
</style>
