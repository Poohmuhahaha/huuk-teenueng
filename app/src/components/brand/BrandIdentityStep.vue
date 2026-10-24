<script setup lang="ts">
import { t } from '@/core/i18n'
import type { Brand } from '@/mock/db'

defineProps<{
  brand: Brand
  canWrite: boolean
  busy: boolean
}>()

const emit = defineEmits<{
  (e: 'save', key: 'channel', event: Event): void
}>()
</script>

<template>
  <section class="brand-body brand-hero">
    <p class="brand-prompt">{{ t('brand.stepIdentityHint') }}</p>
    <input
      id="brand-channel"
      class="field"
      :value="brand.channel"
      :disabled="!canWrite || busy"
      :title="canWrite ? '' : t('auth.noPerm')"
      :aria-label="t('brand.stepIdentityHint')"
      @change="emit('save', 'channel', $event)"
    />
  </section>
</template>
