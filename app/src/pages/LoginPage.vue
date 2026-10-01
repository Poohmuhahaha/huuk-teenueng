<script setup lang="ts">
// Full-screen gate shown before the app when AUTH_REQUIRED is on.
// New users go to the dedicated /register page (SaaS self-serve signup).
import { computed, onMounted, ref } from 'vue'
import { useRouter } from 'vue-router'
import { login } from '@/core/auth'
import { useSetup } from '@/core/queries'
import { t } from '@/core/i18n'

const router = useRouter()
const { data: setup } = useSetup()

const email = ref('')
const password = ref('')
const busy = ref(false)
const error = ref('')
const form = ref<HTMLFormElement | null>(null)

const canRegister = computed(() => setup.value?.allowRegistration ?? false)
const workspace = computed(() => setup.value?.workspaceName || 'workspace')

onMounted(() => {
  form.value?.querySelector('input')?.focus()
})

async function submit(): Promise<void> {
  if (busy.value) return
  error.value = ''
  busy.value = true
  try {
    await login(email.value.trim(), password.value)
  } catch (e) {
    error.value = e instanceof Error ? e.message : String(e)
  } finally {
    busy.value = false
  }
}
</script>

<template>
  <div class="loginpage">
    <div class="lp-card">
      <div class="lp-brand">
        <img class="lp-logo" src="/huuk-logo.svg" alt="Huuk by teenueng" />
        <span class="lp-ws">{{ workspace }}</span>
      </div>
      <h1>{{ t('auth.login') }}</h1>
      <form ref="form" @submit.prevent="submit">
        <label class="lbl" for="lp-email">{{ t('auth.email') }}</label>
        <input id="lp-email" v-model="email" class="field" type="email" autocomplete="email" required />
        <label class="lbl" for="lp-password">{{ t('auth.password') }}</label>
        <input id="lp-password" v-model="password" class="field" type="password"
          autocomplete="current-password" required />
        <p v-if="error" class="autherr" role="alert">{{ error }}</p>
        <button type="submit" class="btn btn-primary lp-submit" :disabled="busy">
          {{ busy ? t('common.loading') : t('auth.login') }}
        </button>
      </form>
      <button v-if="canRegister" type="button" class="lp-switch" @click="router.push('/register')">
        {{ t('auth.noAccount') }}
      </button>
    </div>
  </div>
</template>
