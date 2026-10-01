<script setup lang="ts">
import { onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { changePassword } from '@/core/auth'
import { t } from '@/core/i18n'

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

watch(
  () => props.open,
  (open) => {
    if (!open) return
    oldPassword.value = ''
    newPassword.value = ''
    confirmPassword.value = ''
    error.value = ''
    done.value = false
  },
)

async function submit(): Promise<void> {
  if (busy.value) return
  error.value = ''
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
  <div v-if="open" class="backdrop" @click.self="emit('close')">
    <div class="modal" role="dialog" aria-modal="true">
      <h3>{{ t('auth.change') }}</h3>
      <form @submit.prevent="submit">
        <label class="lbl" for="cp-old">{{ t('auth.oldPassword') }}</label>
        <input id="cp-old" v-model="oldPassword" class="field" type="password" autocomplete="current-password" />
        <label class="lbl" for="cp-new">{{ t('auth.newPassword') }}</label>
        <input id="cp-new" v-model="newPassword" class="field" type="password" autocomplete="new-password" />
        <label class="lbl" for="cp-confirm">{{ t('auth.confirmNew') }}</label>
        <input id="cp-confirm" v-model="confirmPassword" class="field" type="password" autocomplete="new-password" />
        <p v-if="error" class="autherr">{{ error }}</p>
        <p v-if="done" class="autherr ok">{{ t('auth.passwordChanged') }}</p>
        <div class="actions">
          <button type="button" class="btn" @click="emit('close')">{{ t('common.cancel') }}</button>
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
</template>

<style scoped>
.autherr {
  margin: 10px 0 0;
  font-size: 14px;
  font-weight: 700;
}
.autherr.ok { color: var(--success); }
</style>
