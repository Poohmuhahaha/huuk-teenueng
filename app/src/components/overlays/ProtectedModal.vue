<script setup lang="ts">
import { nextTick, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { t } from '@/core/i18n'
import { suppressGuard } from '@/core/protected'

const props = defineProps<{ open: boolean; field: string }>()
const emit = defineEmits<{ cancel: []; confirm: [] }>()
const hide = ref(false)
const confirming = ref(false)
const cancelBtn = ref<HTMLButtonElement | null>(null)

watch(
  () => props.open,
  async (open) => {
    if (!open) return
    hide.value = false
    confirming.value = false
    await nextTick()
    cancelBtn.value?.focus()
  },
)

function confirm(): void {
  if (confirming.value) return
  confirming.value = true
  if (hide.value) suppressGuard(5)
  emit('confirm')
}

function onKey(e: KeyboardEvent): void {
  if (!props.open) return
  if (e.key === 'Escape') emit('cancel')
}
onMounted(() => window.addEventListener('keydown', onKey))
onBeforeUnmount(() => window.removeEventListener('keydown', onKey))
</script>

<template>
  <div v-if="open" class="backdrop" @click.self="emit('cancel')">
    <div class="modal" role="dialog" aria-modal="true" aria-labelledby="guard-title" aria-describedby="guard-message">
      <h3 id="guard-title">{{ t('guard.title') }}</h3>
      <p id="guard-message" class="muted">{{ t('guard.message').replace('{field}', field) }}</p>
      <label class="muted" style="display: block; margin-top: 8px;">
        <input v-model="hide" type="checkbox" /> {{ t('guard.hide') }}
      </label>
      <div class="actions">
        <button ref="cancelBtn" class="btn" @click="emit('cancel')">{{ t('common.cancel') }}</button>
        <button class="btn btn-primary" :disabled="confirming" @click="confirm">{{ t('common.confirm') }}</button>
      </div>
    </div>
  </div>
</template>
