<script setup lang="ts">
import { onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { t } from '@/core/i18n'
import { suppressGuard } from '@/core/protected'

const props = defineProps<{ open: boolean; field: string }>()
const emit = defineEmits<{ cancel: []; confirm: [] }>()
const hide = ref(false)

watch(
  () => props.open,
  (open) => {
    if (open) hide.value = false
  },
)

function confirm(): void {
  if (hide.value) suppressGuard(5)
  emit('confirm')
}

function onKey(e: KeyboardEvent): void {
  if (e.key === 'Escape' && props.open) emit('cancel')
}
onMounted(() => window.addEventListener('keydown', onKey))
onBeforeUnmount(() => window.removeEventListener('keydown', onKey))
</script>

<template>
  <div v-if="open" class="backdrop" @click.self="emit('cancel')">
    <div class="modal" role="dialog" aria-modal="true" aria-labelledby="guard-title">
      <h3 id="guard-title">{{ t('guard.title') }}</h3>
      <p class="muted">{{ t('guard.message').replace('{field}', field) }}</p>
      <label class="muted" style="display: block; margin-top: 8px;">
        <input v-model="hide" type="checkbox" /> {{ t('guard.hide') }}
      </label>
      <div class="actions">
        <button class="btn" @click="emit('cancel')">{{ t('common.cancel') }}</button>
        <button class="btn btn-primary" @click="confirm">{{ t('common.confirm') }}</button>
      </div>
    </div>
  </div>
</template>
