<script setup lang="ts">
import { onBeforeUnmount, ref } from 'vue'

const props = defineProps<{ text: string }>()
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
  <div class="card" style="background: var(--wash);">
    <strong>Copy-paste assembled</strong>
    <p class="muted" style="white-space: pre-wrap;">{{ text || '(empty — fill caption + CTA + tags)' }}</p>
    <button class="btn btn-primary" :disabled="!text" @click="copy">
      {{ copied ? 'Copied' : 'Copy for publishing' }}
    </button>
    <p v-if="failed" class="autherr" role="alert">
      Clipboard unavailable — select the text above and copy it manually.
    </p>
  </div>
</template>
