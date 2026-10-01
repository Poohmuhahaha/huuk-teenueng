<script setup lang="ts">
import { nextTick, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { login, register } from '@/core/auth'
import { t } from '@/core/i18n'

const props = defineProps<{ open: boolean; initialMode?: 'login' | 'register' }>()
const emit = defineEmits<{ close: []; success: [mode: 'login' | 'register'] }>()

const mode = ref<'login' | 'register'>('login')
const name = ref('')
const email = ref('')
const password = ref('')
const busy = ref(false)
const error = ref('')
const firstField = ref<HTMLInputElement | null>(null)

watch(
  () => props.open,
  async (open) => {
    if (!open) return
    mode.value = props.initialMode ?? 'login'
    error.value = ''
    // Never keep another user's name/email in the form.
    name.value = ''
    email.value = ''
    password.value = ''
    await nextTick()
    firstField.value?.focus()
  },
  { immediate: true },
)

function onKey(e: KeyboardEvent): void {
  if (e.key === 'Escape' && props.open && !busy.value) emit('close')
}
onMounted(() => window.addEventListener('keydown', onKey))
onBeforeUnmount(() => window.removeEventListener('keydown', onKey))

function toggleMode(): void {
  mode.value = mode.value === 'login' ? 'register' : 'login'
  error.value = ''
}

async function submit(): Promise<void> {
  if (busy.value) return
  busy.value = true
  error.value = ''
  const m = mode.value
  try {
    if (m === 'login') await login(email.value, password.value)
    else await register({ name: name.value, email: email.value, password: password.value })
    emit('success', m)
    emit('close')
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
      <h3>{{ mode === 'login' ? t('auth.login') : t('auth.registerTitle') }}</h3>
      <form @submit.prevent="submit">
        <template v-if="mode === 'register'">
          <label class="lbl" for="auth-name">{{ t('auth.name') }}</label>
          <input id="auth-name" ref="firstField" v-model="name" class="field" type="text" autocomplete="name" required />
        </template>
        <label class="lbl" for="auth-email">{{ t('auth.email') }}</label>
        <input v-if="mode === 'login'" id="auth-email" ref="firstField" v-model="email" class="field" type="email"
          autocomplete="email" required />
        <input v-else id="auth-email" v-model="email" class="field" type="email" autocomplete="email" required />
        <label class="lbl" for="auth-password">{{ t('auth.password') }}</label>
        <input id="auth-password" v-model="password" class="field" type="password"
          :autocomplete="mode === 'login' ? 'current-password' : 'new-password'" />
        <p v-if="error" class="autherr">{{ error }}</p>
        <button type="button" class="switchlink" @click="toggleMode">
          {{ mode === 'login' ? t('auth.noAccount') : t('auth.haveAccount') }}
        </button>
        <div class="actions">
          <button type="button" class="btn" @click="emit('close')">{{ t('common.cancel') }}</button>
          <button type="submit" class="btn btn-primary" :disabled="busy">
            {{ mode === 'login' ? t('auth.login') : t('auth.register') }}
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
.switchlink {
  display: inline-block;
  margin-top: 10px;
  padding: 0;
  border: 0;
  background: none;
  font: inherit;
  font-size: 14px;
  color: inherit;
  text-decoration: underline;
  cursor: pointer;
}
</style>
