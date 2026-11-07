<script setup lang="ts">
// Public pricing page. Shares the login/register art direction: warm paper,
// deep forest green, full-bleed editorial imagery and quiet, tactile cards.
import { computed, ref } from 'vue'
import { useRouter } from 'vue-router'
import { choosePlan, currentUser } from '@/core/auth'
import { usePermission } from '@/core/queries'
import { t } from '@/core/i18n'
import './plans.css'

const router = useRouter()
const { can } = usePermission()
const busy = ref('')
const error = ref('')

const loggedIn = computed(() => currentUser.value !== null)
const currentPlan = computed(() => currentUser.value?.plan ?? null)
const canChoosePaid = computed(() => can('users.manage'))

interface Plan {
  id: string
  name: string
  tag: string
  price: string
  period: string
  features: string[]
  popular?: boolean
}

const freePlan: Plan = {
  id: 'free',
  name: 'Free',
  tag: 'Community Edition',
  price: '฿0',
  period: 'forever',
  features: [
    '1 user · 1 workspace',
    '12-month content calendar',
    'Content Studio & Markdown editor',
    'Brand identity kit',
  ],
}

const paidPlans: Plan[] = [
  {
    id: 'starter',
    name: 'Starter',
    tag: 'For freelancers & solo creators',
    price: '฿390',
    period: '/ month',
    features: [
      'Unlimited content calendar',
      'Content Studio & export',
      'Idea backlog & tags',
      'Cloud auto-sync',
    ],
  },
  {
    id: 'growth',
    name: 'Growth',
    tag: 'For brands & teams of 2–5',
    price: '฿1,290',
    period: '/ month',
    features: [
      'Up to 5 seats with RBAC',
      'Campaigns & expense tracking',
      'Views & follower dashboard',
      'Social OAuth connections',
    ],
  },
  {
    id: 'agency',
    name: 'Agency Pro',
    tag: 'For agencies & studios',
    price: '฿3,900',
    period: '/ month',
    popular: true,
    features: [
      'Guest magic-link review',
      'Frame-accurate video proofing',
      'Visual diff slider',
      'White-label PDF reports',
    ],
  },
  {
    id: 'enterprise',
    name: 'Enterprise',
    tag: 'For organizations & media houses',
    price: '฿8,900',
    period: '/ month',
    features: [
      'Unlimited workspaces & brands',
      'Custom review domain',
      'Dedicated S3 storage & priority transcode',
      '24/7 support team',
    ],
  },
]

function freeCta(): string {
  if (busy.value === 'free') return t('common.loading')
  return loggedIn.value ? t('plans.continueFree') : t('plans.startFree')
}

function paidCta(plan: Plan): string {
  if (busy.value === plan.id) return t('common.loading')
  if (currentPlan.value === plan.id) return t('plans.current')
  if (!canChoosePaid.value) return t('plans.contactSales')
  return t('plans.choose', { plan: plan.name })
}

async function continueFree(): Promise<void> {
  if (!loggedIn.value) {
    await router.push('/register')
    return
  }
  error.value = ''
  busy.value = 'free'
  try {
    // New self-serve accounts are created on Free already; skip the redundant
    // write before entering first-run workspace setup.
    if (currentPlan.value !== 'free') await choosePlan('free')
    await router.push('/brand?onboarding=1')
  } catch (e) {
    error.value = e instanceof Error ? e.message : String(e)
  } finally {
    busy.value = ''
  }
}

async function selectTier(planId: string): Promise<void> {
  if (!loggedIn.value) {
    await router.push('/register')
    return
  }
  if (!canChoosePaid.value) {
    error.value = t('plans.paidContact')
    return
  }
  error.value = ''
  busy.value = planId
  try {
    await choosePlan(planId)
    await router.push('/brand?onboarding=1')
  } catch (e) {
    error.value = e instanceof Error ? e.message : String(e)
  } finally {
    busy.value = ''
  }
}
</script>

<template>
  <div class="pricing-page">
    <header class="pricing-top">
      <RouterLink class="pricing-wordmark" to="/" aria-label="Huuk by teenueng">
        <span class="pricing-wordmark-name">Huuk</span>
        <span class="pricing-wordmark-by">by teenueng</span>
      </RouterLink>
      <nav class="pricing-nav" aria-label="Plans">
        <RouterLink class="pricing-navlink pricing-navlink-read" to="/read">{{ t('landing.navRead') }}</RouterLink>
        <RouterLink v-if="!loggedIn" class="pricing-navlink" to="/login">{{ t('auth.login') }}</RouterLink>
        <RouterLink v-else class="pricing-navlink" to="/">{{ t('plans.backToApp') }}</RouterLink>
        <RouterLink v-if="!loggedIn" class="pricing-navcta" to="/register">{{ t('plans.startFree') }}</RouterLink>
      </nav>
    </header>

    <main class="pricing-layout">
      <section class="pricing-content">
        <header class="pricing-hero">
          <p class="pricing-eyebrow">{{ t('plans.eyebrow') }}</p>
          <h1 class="pricing-title">{{ t('plans.title') }}</h1>
          <p class="pricing-lede">{{ t('plans.subtitle') }}</p>
          <div class="pricing-meta" aria-label="Plan notes">
            <span>{{ t('plans.noCard') }}</span>
            <span>{{ t('plans.builtFor') }}</span>
            <span>{{ t('plans.pricesInThb') }}</span>
          </div>
        </header>

        <section class="pricing-free" :class="{ current: currentPlan === 'free' }" aria-labelledby="plan-free">
          <div class="pricing-free-main">
            <div class="pricing-planhead">
              <span class="pricing-index">01</span>
              <div>
                <h2 id="plan-free" class="pricing-plan-name">{{ freePlan.name }}</h2>
                <p class="pricing-plan-tag">{{ freePlan.tag }}</p>
              </div>
            </div>
            <ul class="pricing-features pricing-free-features">
              <li v-for="f in freePlan.features" :key="f">
                <span class="pricing-check" aria-hidden="true">✓</span>{{ f }}
              </li>
            </ul>
          </div>
          <div class="pricing-free-buy">
            <p class="pricing-price">{{ freePlan.price }}<span>{{ freePlan.period }}</span></p>
            <button type="button" class="pricing-btn pricing-btn-dark" data-testid="plan-free-cta"
              :disabled="busy === 'free'" @click="continueFree">
              {{ freeCta() }}
            </button>
          </div>
        </section>

        <div class="pricing-grid">
          <article v-for="(plan, i) in paidPlans" :key="plan.id" class="pricing-card"
            :class="{ popular: plan.popular, current: currentPlan === plan.id }">
            <div class="pricing-cardhead">
              <div>
                <p class="pricing-index">0{{ i + 2 }}</p>
                <h2 class="pricing-plan-name">{{ plan.name }}</h2>
                <p class="pricing-plan-tag">{{ plan.tag }}</p>
              </div>
              <span v-if="plan.popular" class="pricing-badge">{{ t('plans.popular') }}</span>
            </div>
            <p class="pricing-price">{{ plan.price }}<span>{{ plan.period }}</span></p>
            <ul class="pricing-features">
              <li v-for="f in plan.features" :key="f">
                <span class="pricing-check" aria-hidden="true">✓</span>{{ f }}
              </li>
            </ul>
            <button type="button" class="pricing-btn"
              :class="plan.popular ? 'pricing-btn-light' : 'pricing-btn-ghost'"
              :data-testid="`plan-${plan.id}-cta`"
              :disabled="busy === plan.id || currentPlan === plan.id" @click="selectTier(plan.id)">
              {{ paidCta(plan) }}
            </button>
          </article>
        </div>

        <p v-if="error" class="pricing-error" role="alert">{{ error }}</p>

        <p class="pricing-custom">
          {{ t('plans.custom') }}
          <a href="https://teenueng.com" target="_blank" rel="noopener">{{ t('plans.talkToUs') }} →</a>
        </p>

        <footer class="pricing-footer">
          <span>© 2026 Huuk by teenueng</span>
          <span>Beta · teenueng.com</span>
        </footer>
      </section>

      <aside class="pricing-art" aria-label="Huuk by teenueng">
        <div class="pricing-art-top">
          <span class="pricing-mark" aria-hidden="true">H</span>
          <span class="pricing-beta">Beta</span>
        </div>
        <div class="pricing-art-copy">
          <p class="pricing-art-kicker">Huuk by teenueng</p>
          <p class="pricing-art-title">Be a part of<br /><strong>something beautiful.</strong></p>
        </div>
      </aside>
    </main>
  </div>
</template>
