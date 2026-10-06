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
  <div class="card copybar" :class="{ compact }" style="background: var(--wash);">
    <div class="copybar-info">
      <strong>Copy-paste assembled</strong>
      <p class="copybar-text muted">{{ text || '(empty — fill caption + CTA + tags)' }}</p>
    </div>
    <button v-if="!compact" class="btn btn-primary" :disabled="!text" @click="copy">
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
    <p v-if="failed" class="autherr" role="alert">
      Clipboard unavailable — select the text above and copy it manually.
    </p>
  </div>
</template>

<style scoped>
.copybar-text { white-space: pre-wrap; }

/* Compact strip: label + one-line preview on the left, copy icon on the right. */
.copybar.compact {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 10px 12px;
}
.copybar.compact .copybar-info {
  flex: 1;
  min-width: 0;
  display: flex;
  align-items: baseline;
  gap: 10px;
}
.copybar.compact strong { flex: none; font-size: 13px; }
.copybar.compact .copybar-text {
  margin: 0;
  font-size: 12.5px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
</style>
