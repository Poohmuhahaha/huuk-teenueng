<script setup lang="ts">
// Public marketing entry — chrome ported from the teenueng portfolio
// design system (strict monochrome, Tinos display + Inter UI, square
// corners, hard-step hover shadows; see ./landing.css). Stays English-only.
import { computed, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { currentUser } from '@/core/auth'
import { t } from '@/core/i18n'
import './landing.css'

const router = useRouter()
const loggedIn = computed(() => currentUser.value !== null)

interface Hub {
  to: string
  name: string
  tagline: string
  tabs: string
  access: string
}

const hubs: Hub[] = [
  { to: '/dashboard', name: 'landing.hubHome', tagline: 'landing.hubHomeTag', tabs: 'Overview · Ads · Top posts · Performance · Finance', access: 'computed' },
  { to: '/plan', name: 'landing.hubPlan', tagline: 'landing.hubPlanTag', tabs: 'Monthly · Calendar · Ideas', access: 'WRITE master' },
  { to: '/content', name: 'landing.hubContent', tagline: 'landing.hubContentTag', tabs: 'Studio · Feed preview · Hashtags', access: 'WRITE master' },
  { to: '/promote', name: 'landing.hubPromote', tagline: 'landing.hubPromoteTag', tabs: 'Campaigns · Meta Ads', access: 'WRITE master' },
  { to: '/branding', name: 'landing.hubSettings', tagline: 'landing.hubSettingsTag', tabs: 'Brand · Members · Connections', access: 'owner' },
]

interface Tier {
  id: string
  name: string
  tag: string
  price: string
  features: string[]
  featured?: boolean
}

const tiers: Tier[] = [
  { id: 'free', name: 'plans.free', tag: 'plans.freeTag', price: 'plans.freePrice',
    features: ['plans.freeF1', 'plans.freeF2', 'plans.freeF3'] },
  { id: 'pro', name: 'plans.pro', tag: 'plans.proTag', price: 'plans.contactPrice', featured: true,
    features: ['plans.proF1', 'plans.proF2', 'plans.proF3'] },
  { id: 'business', name: 'plans.business', tag: 'plans.businessTag', price: 'plans.contactPrice',
    features: ['plans.businessF1', 'plans.businessF2', 'plans.businessF3'] },
]

const demos = [
  { email: 'owner@studio.local', role: 'Owner' },
  { email: 'editor@studio.local', role: 'Editor' },
  { email: 'client@studio.local', role: 'Client' },
]

const steps = [
  { n: '01', title: 'landing.step1', body: 'landing.step1Body' },
  { n: '02', title: 'landing.step2', body: 'landing.step2Body' },
  { n: '03', title: 'landing.step3', body: 'landing.step3Body' },
]

function go(path: string): void {
  void router.push(path)
}

onMounted(() => {
  const els = Array.from(document.querySelectorAll('.pf-rv'))
  if (typeof IntersectionObserver === 'undefined') {
    els.forEach((el) => el.classList.add('in'))
    return
  }
  const io = new IntersectionObserver(
    (entries) => {
      for (const e of entries) {
        if (e.isIntersecting) {
          e.target.classList.add('in')
          io.unobserve(e.target)
        }
      }
    },
    { rootMargin: '0px 0px -12% 0px' },
  )
  els.forEach((el) => io.observe(el))
})
</script>

<template>
  <div class="pf">
    <header class="pf-header">
      <RouterLink class="pf-wordmark" to="/landing">huuk</RouterLink>
      <nav class="pf-nav" aria-label="Landing">
        <RouterLink class="pf-navlink" to="/read">{{ t('landing.navRead') }}</RouterLink>
        <RouterLink class="pf-navlink" to="/plans">{{ t('landing.navPlans') }}</RouterLink>
        <RouterLink v-if="!loggedIn" class="pf-navlink" to="/login">{{ t('auth.login') }}</RouterLink>
        <RouterLink v-else class="pf-navlink" to="/">{{ t('plans.backToApp') }}</RouterLink>
        <button type="button" class="pf-btn pf-btn-solid pf-btn-sm" @click="go('/register')">
          {{ t('landing.ctaStart') }}
        </button>
      </nav>
    </header>

    <main>
      <!-- hero -->
      <section class="pf-band">
        <div class="pf-hero-grid">
          <div class="pf-rv">
            <p class="pf-eyebrow">{{ t('landing.eyebrow') }}</p>
            <h1 class="pf-display">{{ t('landing.heroTitle') }}</h1>
            <p class="pf-lede">{{ t('landing.heroSub') }}</p>
            <div class="pf-cta-row" style="margin-top: 28px;">
              <button type="button" class="pf-btn pf-btn-solid" @click="go('/register')">
                {{ t('landing.ctaStart') }}
              </button>
              <button type="button" class="pf-btn pf-btn-ghost" @click="go('/')">
                {{ t('landing.ctaOpen') }}
              </button>
              <button type="button" class="pf-btn pf-btn-ghost" @click="go('/plans')">
                {{ t('landing.ctaPlans') }}
              </button>
            </div>
            <ul class="pf-tags" aria-label="Stack">
              <li class="pf-tag">{{ t('landing.stackVue') }}</li>
              <li class="pf-tag">{{ t('landing.stackRust') }}</li>
              <li class="pf-tag">{{ t('landing.stackSwap') }}</li>
            </ul>
          </div>
          <div class="pf-scene pf-rv" aria-hidden="true">
            <p class="pf-scene-word">huuk</p>
            <p class="pf-scene-cap">{{ t('landing.sceneCap') }}</p>
          </div>
        </div>
      </section>

      <!-- hubs -->
      <section class="pf-band" aria-labelledby="landing-features">
        <div class="pf-rv">
          <p class="pf-eyebrow">{{ t('landing.featuresEyebrow') }}</p>
          <h2 id="landing-features" class="pf-h2">{{ t('landing.featuresTitle') }}</h2>
          <p class="pf-lede">{{ t('landing.featuresSub') }}</p>
        </div>
        <div class="pf-grid-3">
          <article v-for="(hub, i) in hubs" :key="hub.to" class="pf-rv" :style="`transition-delay: ${(i % 3) * 90}ms`">
            <p class="pf-work-meta">{{ hub.tabs }}</p>
            <h3 class="pf-work-name">{{ t(hub.name) }}</h3>
            <p class="pf-work-desc">{{ t(hub.tagline) }}</p>
            <p class="pf-work-access">{{ hub.access }}</p>
            <RouterLink class="pf-work-link" :to="hub.to">{{ t('landing.openHub') }} →</RouterLink>
          </article>
        </div>
      </section>

      <!-- how it works -->
      <section class="pf-band" aria-labelledby="landing-how">
        <div class="pf-rv">
          <p class="pf-eyebrow">{{ t('landing.howEyebrow') }}</p>
          <h2 id="landing-how" class="pf-h2">{{ t('landing.howTitle') }}</h2>
        </div>
        <div class="pf-rows">
          <div v-for="s in steps" :key="s.n" class="pf-row pf-rv">
            <span class="pf-row-num">{{ s.n }}</span>
            <strong class="pf-row-title">{{ t(s.title) }}</strong>
            <p class="pf-row-body">{{ t(s.body) }}</p>
          </div>
        </div>
      </section>

      <!-- pricing -->
      <section class="pf-band" aria-labelledby="landing-pricing">
        <div class="pf-rv">
          <p class="pf-eyebrow">{{ t('landing.pricingEyebrow') }}</p>
          <h2 id="landing-pricing" class="pf-h2">{{ t('plans.title') }}</h2>
          <p class="pf-lede">{{ t('plans.subtitle') }}</p>
        </div>
        <div class="pf-price-grid">
          <div v-for="(tier, i) in tiers" :key="tier.id" class="pf-card pf-rv"
            :class="tier.featured ? 'pf-featured' : ''" :style="`transition-delay: ${i * 90}ms`">
            <strong class="pf-card-name">{{ t(tier.name) }}</strong>
            <p class="pf-card-tag">{{ t(tier.tag) }}</p>
            <p class="pf-card-price">{{ t(tier.price) }}</p>
            <ul>
              <li v-for="f in tier.features" :key="f">{{ t(f) }}</li>
            </ul>
            <button v-if="tier.id === 'free'" type="button"
              class="pf-btn" :class="tier.featured ? 'pf-btn-solid' : 'pf-btn-ghost'" @click="go('/register')">
              {{ t('plans.continueFree') }}
            </button>
            <a v-else class="pf-btn" :class="tier.featured ? 'pf-btn-solid' : 'pf-btn-ghost'"
              href="https://teenueng.com" target="_blank" rel="noopener">{{ t('plans.contact') }}</a>
          </div>
        </div>
      </section>

      <!-- demo -->
      <section class="pf-band" aria-labelledby="landing-demo">
        <div class="pf-rv">
          <p class="pf-eyebrow">{{ t('landing.demoEyebrow') }}</p>
          <h2 id="landing-demo" class="pf-h2">{{ t('landing.demoTitle') }}</h2>
          <p class="pf-lede">{{ t('landing.demoSub') }}</p>
        </div>
        <div class="pf-rv">
          <div v-for="d in demos" :key="d.email" class="pf-demo-row">
            <code>{{ d.email }}</code>
            <span class="pf-demo-role">{{ d.role }} · demo1234</span>
            <button type="button" class="pf-btn pf-btn-ghost pf-btn-sm" @click="go('/login')">
              {{ t('auth.login') }}
            </button>
          </div>
          <p class="pf-lede" style="margin-top: 20px; font-size: 16px;">{{ t('landing.demoNote') }}</p>
        </div>
      </section>

      <!-- final CTA -->
      <section class="pf-band pf-cta-final">
        <div class="pf-rv">
          <p class="pf-eyebrow">{{ t('landing.finalEyebrow') }}</p>
          <h2 class="pf-display-2">{{ t('landing.finalTitle') }}</h2>
          <p class="pf-lede">{{ t('landing.finalLede') }}</p>
          <div class="pf-cta-row" style="margin-top: 28px;">
            <button type="button" class="pf-btn pf-btn-solid" @click="go('/register')">
              {{ t('landing.ctaStart') }}
            </button>
          </div>
        </div>
      </section>
    </main>

    <footer class="pf-footer">
      <RouterLink class="pf-wordmark" to="/landing">huuk</RouterLink>
      <div class="pf-footer-links">
        <RouterLink to="/read">{{ t('landing.navRead') }}</RouterLink>
        <RouterLink to="/plans">{{ t('landing.navPlans') }}</RouterLink>
        <RouterLink to="/login">{{ t('auth.login') }}</RouterLink>
      </div>
      <p class="pf-copy">{{ t('landing.footerCopy') }}</p>
    </footer>
  </div>
</template>
