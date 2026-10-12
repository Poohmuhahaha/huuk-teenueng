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
  <div class="flex gap-2" @paste="onPaste">
    <input
      v-for="(char, index) in chars"
      :key="index"
      :ref="(el) => (inputs[index] = el as HTMLInputElement | null)"
      class="w-[42px] h-12 text-center text-xl font-bold border border-line bg-surface text-ink p-0 rounded-none transition-colors focus:outline-none focus:border-ink disabled:opacity-50 disabled:cursor-not-allowed"
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
