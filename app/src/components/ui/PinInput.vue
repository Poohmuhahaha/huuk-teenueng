<script setup lang="ts">
import { computed, ref } from 'vue'

const model = defineModel<string>({ default: '' })
const props = withDefaults(defineProps<{ length?: number; disabled?: boolean; type?: 'text' | 'password' }>(), {
  length: 4,
  type: 'text',
})
const inputs = ref<(HTMLInputElement | null)[]>([])
const chars = computed(() => Array.from({ length: props.length }, (_, i) => model.value[i] ?? ''))

function setChar(index: number, raw: string): void {
  const digit = raw.replace(/\D/g, '').slice(-1)
  model.value = chars.value.map((char, i) => (i === index ? digit : char)).join('')
  if (digit && index < props.length - 1) inputs.value[index + 1]?.focus()
}

function onKeydown(index: number, event: KeyboardEvent): void {
  if (event.key === 'Backspace' && !chars.value[index] && index > 0) inputs.value[index - 1]?.focus()
}

function onPaste(event: ClipboardEvent): void {
  const text = event.clipboardData?.getData('text')?.replace(/\D/g, '') ?? ''
  if (!text) return
  event.preventDefault()
  model.value = text.slice(0, props.length)
}
</script>

<template>
  <div class="p-pin" @paste="onPaste">
    <input
      v-for="(char, index) in chars"
      :key="index"
      :ref="(el) => (inputs[index] = el as HTMLInputElement | null)"
      class="p-pin-box"
      :type="type"
      :value="char"
      :disabled="disabled"
      inputmode="numeric"
      maxlength="1"
      @input="setChar(index, ($event.target as HTMLInputElement).value)"
      @keydown="onKeydown(index, $event)"
    />
  </div>
</template>

<style scoped>
.p-pin { display: flex; gap: 8px; }
.p-pin-box {
  width: 42px;
  height: 48px;
  text-align: center;
  font-size: 20px;
  font-weight: 700;
  border: 1px solid var(--line, #d9d9d9);
  background: var(--paper, #fff);
  color: var(--ink, #000);
  padding: 0;
}
.p-pin-box:focus { outline: none; border-color: var(--ink, #000); }
.p-pin-box:disabled { opacity: 0.5; }
</style>
