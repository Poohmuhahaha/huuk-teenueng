<script setup lang="ts">
import { ref, watchEffect } from 'vue'

const model = defineModel<boolean>({ default: false })
const props = defineProps<{ label?: string; disabled?: boolean; indeterminate?: boolean }>()
const box = ref<HTMLInputElement | null>(null)

watchEffect(() => {
  if (box.value) box.value.indeterminate = !!props.indeterminate && !model.value
})
</script>

<template>
  <label class="p-check" :class="{ disabled }">
    <input
      ref="box"
      type="checkbox"
      :checked="model"
      :disabled="disabled"
      @change="model = ($event.target as HTMLInputElement).checked"
    />
    <span class="p-check-mark" aria-hidden="true">✓</span>
    <span v-if="label || $slots.default" class="p-check-label"><slot>{{ label }}</slot></span>
  </label>
</template>

<style scoped>
.p-check { display: inline-flex; align-items: center; gap: 8px; cursor: pointer; font-size: 14px; color: var(--ink, #000); }
.p-check.disabled { opacity: 0.5; cursor: not-allowed; }
.p-check input { position: absolute; opacity: 0; width: 1px; height: 1px; }
.p-check-mark {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 18px;
  height: 18px;
  border: 1px solid var(--line, #d9d9d9);
  background: var(--paper, #fff);
  font-size: 12px;
  color: transparent;
  flex: none;
  transition: background 0.1s, color 0.1s;
}
.p-check input:checked + .p-check-mark { background: var(--ink, #000); color: var(--paper, #fff); border-color: var(--ink, #000); }
.p-check input:indeterminate + .p-check-mark { background: var(--ink, #000); color: var(--paper, #fff); border-color: var(--ink, #000); }
.p-check input:indeterminate + .p-check-mark::after { content: '–'; }
.p-check input:indeterminate + .p-check-mark { font-size: 0; }
.p-check input:focus-visible + .p-check-mark { outline: 2px solid var(--ink, #000); outline-offset: 2px; }
</style>
