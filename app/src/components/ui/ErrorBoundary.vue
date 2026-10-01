<script setup lang="ts">
// Catches render/handler errors from descendants so one broken screen cannot
// blank the whole deck.
import { onErrorCaptured, ref } from 'vue'
import { t } from '@/core/i18n'

const error = ref<Error | null>(null)

onErrorCaptured((err) => {
  error.value = err instanceof Error ? err : new Error(String(err))
  console.error('[content-planner] captured error:', err)
  return false
})

function retry(): void {
  error.value = null
}

function reload(): void {
  window.location.reload()
}
</script>

<template>
  <div v-if="error" class="card" role="alert">
    <strong>{{ t('common.somethingWrong') }}</strong>
    <p class="muted">{{ error.message }}</p>
    <div class="row mt">
      <button class="btn btn-primary" @click="retry">{{ t('common.tryAgain') }}</button>
      <button class="btn" @click="reload">{{ t('common.reload') }}</button>
    </div>
  </div>
  <slot v-else />
</template>
