<script setup lang="ts">
// Full-screen gate shown before the app when AUTH_REQUIRED is on.
// New users go to the dedicated /register page (SaaS self-serve signup).
import { computed, onMounted, ref } from 'vue'
import { useRouter } from 'vue-router'
import { login } from '@/core/auth'
import { useSetup } from '@/core/queries'
import { t } from '@/core/i18n'

const REMEMBER_KEY = 'huuk.login.email'

const router = useRouter()
const { data: setup } = useSetup()

const email = ref('')
const password = ref('')
const remember = ref(false)
const busy = ref(false)
const error = ref('')
const form = ref<HTMLFormElement | null>(null)
const mobileSheetOpen = ref(false)
const dragY = ref(0)
const isDragging = ref(false)

let startClientY = 0
let lastClientY = 0
let lastTimestamp = 0
let velocityY = 0

function startDrag(e: PointerEvent): void {
  if (e.button !== 0) return
  isDragging.value = true
  startClientY = e.clientY
  lastClientY = e.clientY
  lastTimestamp = performance.now()
  dragY.value = 0
  velocityY = 0

  const onPointerMove = (ev: PointerEvent): void => {
    if (!isDragging.value) return
    const now = performance.now()
    const dt = now - lastTimestamp
    const dy = ev.clientY - lastClientY
    if (dt > 0) velocityY = (dy / dt) * 1000
    lastClientY = ev.clientY
    lastTimestamp = now

    const delta = ev.clientY - startClientY
    if (delta > 0) {
      dragY.value = delta
    } else {
      dragY.value = delta * 0.15
    }
  }

  const onPointerUp = (): void => {
    if (!isDragging.value) return
    isDragging.value = false
    window.removeEventListener('pointermove', onPointerMove)
    window.removeEventListener('pointerup', onPointerUp)
    window.removeEventListener('pointercancel', onPointerUp)

    if (dragY.value > 90 || velocityY > 450) {
      mobileSheetOpen.value = false
    }
    dragY.value = 0
  }

  window.addEventListener('pointermove', onPointerMove)
  window.addEventListener('pointerup', onPointerUp)
  window.addEventListener('pointercancel', onPointerUp)
}

const canRegister = computed(() => setup.value?.allowRegistration ?? false)

onMounted(() => {
  try {
    const saved = localStorage.getItem(REMEMBER_KEY)
    if (saved) {
      email.value = saved
      remember.value = true
    }
  } catch {
    // storage unavailable (private mode) — the checkbox simply stays off
  }
  form.value?.querySelector('input')?.focus()
})

async function submit(): Promise<void> {
  if (busy.value) return
  error.value = ''
  busy.value = true
  try {
    try {
      if (remember.value) localStorage.setItem(REMEMBER_KEY, email.value.trim())
      else localStorage.removeItem(REMEMBER_KEY)
    } catch {
      // storage unavailable — login still proceeds
    }
    await login(email.value.trim(), password.value)
    void router.push('/')
  } catch (e) {
    error.value = e instanceof Error ? e.message : String(e)
    mobileSheetOpen.value = true
  } finally {
    busy.value = false
  }
}
</script>

<template>
  <div class="loginpage">
    <!-- Mobile WHOOP-style Gateway view -->
    <div class="whoop-view">
      <div class="whoop-brand">
        <h1 class="whoop-logo">HUUK<span class="whoop-reg">®</span></h1>
      </div>
      <div class="whoop-actions">
        <button type="button" class="whoop-btn whoop-btn-primary" @click="mobileSheetOpen = true">
          I have an account
        </button>
        <button
          v-if="canRegister"
          type="button"
          class="whoop-btn whoop-btn-secondary"
          @click="router.push('/register')"
        >
          I do not have an account
        </button>
        <p class="whoop-tagline">
          {{ t('auth.heroLine1') }} {{ t('auth.heroLine2') }} <strong>{{ t('auth.heroStrong') }}</strong>
        </p>
      </div>
    </div>

    <!-- Mobile Backdrop for Bottom Sheet -->
    <div
      class="lp-backdrop"
      :class="{ open: mobileSheetOpen }"
      aria-hidden="true"
      @click="mobileSheetOpen = false"
    />

    <!-- Form container: Desktop left column / Mobile bottom sheet -->
    <div
      class="lp-card"
      :class="{ 'sheet-open': mobileSheetOpen, 'is-dragging': isDragging }"
      :style="mobileSheetOpen && (isDragging || dragY !== 0) ? { transform: `translateY(${Math.max(0, dragY)}px)`, transition: isDragging ? 'none' : undefined } : undefined"
    >
      <div
        class="lp-sheet-handle-bar"
        @pointerdown="startDrag"
      >
        <div class="lp-sheet-handle" aria-hidden="true" />
      </div>
      <button
        type="button"
        class="lp-sheet-close"
        aria-label="Close"
        @click="mobileSheetOpen = false"
      >
        ✕
      </button>
      <div class="lp-form">
        <h1>{{ t('auth.login') }}</h1>
        <p class="lp-sub">{{ t('auth.loginSubtitle') }}</p>
        <form ref="form" @submit.prevent="submit">
          <label class="lbl" for="lp-email">{{ t('auth.email') }}</label>
          <input id="lp-email" v-model="email" class="field" type="email" autocomplete="email" required />
          <label class="lbl" for="lp-password">{{ t('auth.password') }}</label>
          <input id="lp-password" v-model="password" class="field" type="password"
            autocomplete="current-password" required />
          <label class="lp-remember" for="lp-remember">
            <input id="lp-remember" v-model="remember" type="checkbox" />
            <span>{{ t('auth.remember') }}</span>
          </label>
          <p v-if="error" class="autherr" role="alert">{{ error }}</p>
          <button type="submit" class="btn btn-primary lp-submit" :disabled="busy">
            {{ busy ? t('common.loading') : t('auth.login') }}
          </button>
        </form>
        <p v-if="canRegister" class="lp-foot">
          <span>{{ t('auth.notMember') }}</span>
          <button type="button" class="lp-link" @click="router.push('/register')">
            {{ t('auth.register') }}
          </button>
        </p>
      </div>
    </div>
    <aside class="lp-hero">
      <img class="lp-mark" src="/favicon.svg" alt="" aria-hidden="true" />
      <p class="lp-tagline">
        {{ t('auth.heroLine1') }}<br />
        {{ t('auth.heroLine2') }} <strong>{{ t('auth.heroStrong') }}</strong>
      </p>
    </aside>
  </div>
</template>
