<script setup lang="ts">
// SaaS self-serve signup. On success the new account continues to /plans.
import { computed, onMounted, ref } from 'vue'
import { useRouter } from 'vue-router'
import { register } from '@/core/auth'
import { useSetup } from '@/core/queries'
import { t } from '@/core/i18n'
import InfoTip from '@/components/ui/InfoTip.vue'

const router = useRouter()
const { data: setup } = useSetup()

const name = ref('')
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
    await register({ name: name.value.trim(), email: email.value.trim(), password: password.value })
    await router.push('/plans')
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
      <h1>{{ t('auth.registerTitle') }}</h1>
      <p v-if="!canRegister" class="muted" style="margin-top: 0;">{{ t('auth.registerClosed') }}</p>
      <form v-else ref="form" @submit.prevent="submit">
        <label class="lbl" for="reg-name">{{ t('auth.name') }}</label>
        <input id="reg-name" v-model="name" class="field" type="text" autocomplete="name" required />
        <label class="lbl" for="reg-email">{{ t('auth.email') }}</label>
        <input id="reg-email" v-model="email" class="field" type="email" autocomplete="email" required />
        <label class="lbl" for="reg-password">{{ t('auth.password') }}<InfoTip :text="t('auth.passwordHint')" /></label>
        <input id="reg-password" v-model="password" class="field" type="password"
          autocomplete="new-password" required minlength="12" />
        <p v-if="error" class="autherr" role="alert">{{ error }}</p>
        <button type="submit" class="btn btn-primary lp-submit" :disabled="busy">
          {{ busy ? t('common.loading') : t('auth.register') }}
        </button>
      </form>
      <button type="button" class="lp-switch" @click="router.push('/login')">
        {{ t('auth.haveAccount') }}
      </button>
    </div>
  </div>
</template>
