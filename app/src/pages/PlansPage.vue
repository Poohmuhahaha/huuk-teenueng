<script setup lang="ts">
// SaaS packages. Free continues straight into the workspace; paid tiers are
// handled by the main site until billing is wired up.
import { computed, ref } from 'vue'
import { useRouter } from 'vue-router'
import { choosePlan, currentUser } from '@/core/auth'
import { t } from '@/core/i18n'

const router = useRouter()
const busy = ref('')
const error = ref('')

const loggedIn = computed(() => currentUser.value !== null)
const currentPlan = computed(() => currentUser.value?.plan ?? null)

interface Plan {
  id: 'free' | 'pro' | 'business'
  name: string
  tag: string
  price: string
  features: string[]
  popular?: boolean
}

const plans: Plan[] = [
  { id: 'free', name: 'plans.free', tag: 'plans.freeTag', price: 'plans.freePrice',
    features: ['plans.freeF1', 'plans.freeF2', 'plans.freeF3'] },
  { id: 'pro', name: 'plans.pro', tag: 'plans.proTag', price: 'plans.contactPrice', popular: true,
    features: ['plans.proF1', 'plans.proF2', 'plans.proF3'] },
  { id: 'business', name: 'plans.business', tag: 'plans.businessTag', price: 'plans.contactPrice',
    features: ['plans.businessF1', 'plans.businessF2', 'plans.businessF3'] },
]

async function continueFree(): Promise<void> {
  if (!loggedIn.value) {
    await router.push('/register')
    return
  }
  error.value = ''
  busy.value = 'free'
  try {
    await choosePlan('free')
    await router.push('/brand?onboarding=1')
  } catch (e) {
    error.value = e instanceof Error ? e.message : String(e)
  } finally {
    busy.value = ''
  }
}
</script>

<template>
  <div class="planspage">
    <div class="plans-card">
      <div class="plans-head">
        <strong>Huuk</strong><span class="lp-by">by teenueng</span>
        <span class="spacer" />
        <RouterLink v-if="loggedIn" class="btn" to="/">{{ t('plans.backToApp') }}</RouterLink>
        <RouterLink v-else class="btn" to="/login">{{ t('auth.login') }}</RouterLink>
      </div>
      <h1>{{ t('plans.title') }}</h1>
      <p class="plans-sub">{{ t('plans.subtitle') }}</p>
      <div class="plans-grid">
        <div v-for="plan in plans" :key="plan.id" class="plan-card" :class="{ popular: plan.popular }">
          <span v-if="plan.popular" class="plan-badge">{{ t('plans.popular') }}</span>
          <strong>{{ t(plan.name) }}</strong>
          <p class="plan-tag">{{ t(plan.tag) }}</p>
          <p class="plan-price">{{ t(plan.price) }}</p>
          <ul class="plan-list">
            <li v-for="f in plan.features" :key="f">{{ t(f) }}</li>
          </ul>
          <div class="plan-cta">
            <button v-if="plan.id === 'free'" class="btn btn-primary" style="width: 100%;"
              :disabled="busy === 'free'" @click="continueFree">
              {{ busy === 'free' ? t('common.loading') : t('plans.continueFree') }}
            </button>
            <a v-else class="btn" style="width: 100%; text-align: center;"
              href="https://teenueng.com" target="_blank" rel="noopener">{{ t('plans.contact') }}</a>
          </div>
          <p v-if="currentPlan === plan.id" class="plan-current">{{ t('plans.current') }}</p>
        </div>
      </div>
      <p v-if="error" class="autherr" role="alert">{{ error }}</p>
      <div class="plans-foot">
        <span class="muted">teenueng.com · huuk.teenueng.com</span>
      </div>
    </div>
  </div>
</template>
