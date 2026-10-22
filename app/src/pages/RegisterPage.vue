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
const mobileSheetOpen = ref(true)
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
          Create an account
        </button>
        <button
          type="button"
          class="whoop-btn whoop-btn-secondary"
          @click="router.push('/login')"
        >
          I have an account
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
      @click="router.push('/login')"
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
        @click="router.push('/login')"
      >
        ✕
      </button>
      <div class="lp-form">
        <h1>{{ t('auth.registerTitle') }}</h1>
        <p class="lp-sub">{{ t('auth.registerSubtitle') }}</p>
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
        <p class="lp-foot">
          <button type="button" class="lp-link" @click="router.push('/login')">
            {{ t('auth.haveAccount') }}
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
