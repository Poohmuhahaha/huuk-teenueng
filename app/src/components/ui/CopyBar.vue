<script setup lang="ts">
import { onBeforeUnmount, ref } from 'vue'
import IconButton from './IconButton.vue'

const props = withDefaults(defineProps<{ text: string; compact?: boolean }>(), { compact: false })
const copied = ref(false)
const failed = ref(false)
let timer: number | undefined

async function copy(): Promise<void> {
  failed.value = false
  try {
    await navigator.clipboard.writeText(props.text)
    copied.value = true
    window.clearTimeout(timer)
    timer = window.setTimeout(() => { copied.value = false }, 1500)
  } catch {
    // Clipboard can be unavailable (permissions, insecure origin).
    failed.value = true
  }
}

onBeforeUnmount(() => window.clearTimeout(timer))
</script>

<template>
  <div
    class="border border-line rounded-none bg-wash shadow-sm"
    :class="compact ? 'flex items-center gap-3 p-2.5 px-3' : 'p-4 flex flex-col gap-3'"
  >
    <div :class="compact ? 'flex-1 min-w-0 flex items-baseline gap-2.5' : 'flex flex-col gap-1'">
      <strong class="text-ink" :class="compact ? 'shrink-0 text-[13px]' : 'text-sm font-semibold'">Copy-paste assembled</strong>
      <p
        class="text-muted"
        :class="compact ? 'm-0 text-xs overflow-hidden text-ellipsis whitespace-nowrap' : 'm-0 text-xs whitespace-pre-wrap'"
      >{{ text || '(empty — fill caption + CTA + tags)' }}</p>
    </div>
    <button
      v-if="!compact"
      class="w-full bg-ink text-surface border border-ink py-2 px-3 text-sm font-semibold cursor-pointer rounded-none transition-colors hover:bg-accent-hover hover:border-accent-hover disabled:opacity-50 disabled:cursor-not-allowed"
      :disabled="!text"
      @click="copy"
    >
      {{ copied ? 'Copied' : 'Copy for publishing' }}
    </button>
    <IconButton
      v-else
      tone="primary"
      :label="copied ? 'Copied' : 'Copy for publishing'"
      :disabled="!text"
      @click="copy"
    >
      <svg viewBox="0 0 16 16" width="16" height="16" fill="none" stroke="currentColor"
        stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
        <rect x="5.5" y="5.5" width="8" height="8" rx="1.5" />
        <path d="M10.5 5.5V4A1.5 1.5 0 0 0 9 2.5H4A1.5 1.5 0 0 0 2.5 4v5A1.5 1.5 0 0 0 4 10.5h1.5" />
      </svg>
    </IconButton>
    <p v-if="failed" class="text-xs text-danger font-semibold mt-1" role="alert">
      Clipboard unavailable — select the text above and copy it manually.
    </p>
  </div>
</template>
