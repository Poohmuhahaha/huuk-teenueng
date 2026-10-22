<script setup lang="ts">
import { nextTick, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { changePassword } from '@/core/auth'
import { t } from '@/core/i18n'
import Drawer from '@/components/overlays/Drawer.vue'

const props = defineProps<{ open: boolean }>()
const emit = defineEmits<{ close: [] }>()

function onKey(e: KeyboardEvent): void {
  if (e.key === 'Escape' && props.open && !busy.value) emit('close')
}
onMounted(() => window.addEventListener('keydown', onKey))
onBeforeUnmount(() => window.removeEventListener('keydown', onKey))

const oldPassword = ref('')
const newPassword = ref('')
const confirmPassword = ref('')
const busy = ref(false)
const error = ref('')
const done = ref(false)
const oldField = ref<HTMLInputElement | null>(null)

watch(
  () => props.open,
  async (open) => {
    if (!open) return
    oldPassword.value = ''
    newPassword.value = ''
    confirmPassword.value = ''
    error.value = ''
    done.value = false
    await nextTick()
    oldField.value?.focus()
  },
)

async function submit(): Promise<void> {
  if (busy.value) return
  error.value = ''
  if (!newPassword.value) {
    error.value = t('auth.passwordsNoMatch')
    return
  }
  if (newPassword.value !== confirmPassword.value) {
    error.value = t('auth.passwordsNoMatch')
    return
  }
  busy.value = true
  try {
    await changePassword(oldPassword.value, newPassword.value)
    done.value = true
    oldPassword.value = ''
    newPassword.value = ''
    confirmPassword.value = ''
  } catch (e) {
    error.value = e instanceof Error ? e.message : String(e)
  } finally {
    busy.value = false
  }
}
</script>

<template>
  <Drawer
    :model-value="open"
    :title="t('auth.change')"
    side="bottom"
    @update:model-value="(val) => !val && (!busy && emit('close'))"
  >
    <div class="backdrop" style="position: static; inset: auto; background: none; backdrop-filter: none; padding: 0; display: block;" @click.self="!busy && emit('close')">
      <div class="modal" style="position: static; border: 0; padding: 0; box-shadow: none; background: transparent; max-width: none;" role="region" aria-labelledby="cp-title">
        <h3 id="cp-title" style="display: none;">{{ t('auth.change') }}</h3>
        <form @submit.prevent="submit" class="flex flex-col gap-3">
          <label class="lbl" for="cp-old">{{ t('auth.oldPassword') }}</label>
          <input id="cp-old" ref="oldField" v-model="oldPassword" class="field" type="password" autocomplete="current-password" required minlength="8" />
          <label class="lbl" for="cp-new">{{ t('auth.newPassword') }}</label>
          <input id="cp-new" v-model="newPassword" class="field" type="password" autocomplete="new-password" required minlength="8" />
          <label class="lbl" for="cp-confirm">{{ t('auth.confirmNew') }}</label>
          <input id="cp-confirm" v-model="confirmPassword" class="field" type="password" autocomplete="new-password" required minlength="8" />
          <p v-if="error" class="mt-2 text-sm font-bold text-danger" role="alert">{{ error }}</p>
          <p v-if="done" class="mt-2 text-sm font-bold text-success" role="status">{{ t('auth.passwordChanged') }}</p>
          <div class="actions flex justify-end gap-2 mt-4 pt-4 border-t border-line">
            <button type="button" class="btn" :disabled="busy" @click="emit('close')">{{ t('common.cancel') }}</button>
            <button v-if="done" type="button" class="btn btn-primary" @click="emit('close')">
              {{ t('common.close') }}
            </button>
            <button v-else type="submit" class="btn btn-primary" :disabled="busy">
              {{ t('common.confirm') }}
            </button>
          </div>
        </form>
      </div>
    </div>
  </Drawer>
</template>
