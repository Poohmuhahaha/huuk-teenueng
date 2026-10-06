<script setup lang="ts">
const model = defineModel<string>({ default: '' })
withDefaults(
  defineProps<{ type?: string; placeholder?: string; disabled?: boolean; invalid?: boolean; id?: string; size?: 'sm' | 'md' | 'lg' }>(),
  { type: 'text', size: 'md' },
)
</script>

<template>
  <input
    :id="id"
    v-model="model"
    class="p-input"
    :class="[`s-${size}`, { invalid }]"
    :type="type"
    :placeholder="placeholder"
    :disabled="disabled"
    :aria-invalid="invalid || undefined"
  />
</template>

<style scoped>
.p-input {
  background: var(--paper, #fff);
  border: 1px solid var(--line, #d9d9d9);
  color: var(--ink, #000);
  font: inherit;
  font-size: 14px;
  padding: 9px 12px;
  transition: border-color 0.15s;
}
.p-input:focus { outline: none; border-color: var(--ink, #000); }
.p-input::placeholder { color: var(--faint, #a3a3a3); }
.p-input.invalid { border-color: var(--ink, #000); background: var(--wash, #f6f6f6); }
.p-input:disabled { opacity: 0.5; cursor: not-allowed; }
.s-sm { padding: 6px 10px; font-size: 13px; }
.s-lg { padding: 12px 14px; font-size: 15px; }
</style>
