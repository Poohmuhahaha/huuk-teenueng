<script setup lang="ts">
// Account section — moved out of the avatar popup onto the Settings
// workspace tab: profile name, password change, guide, packages, and sign-out.
import { computed, ref } from 'vue'
import { currentUser, logout, logoutAll } from '@/core/auth'
import { t } from '@/core/i18n'
import ProfilePanel from '@/components/overlays/ProfilePanel.vue'
import ChangePasswordModal from '@/components/overlays/ChangePasswordModal.vue'
import PackageModal from '@/components/overlays/PackageModal.vue'
import GuideHero from '@/components/ui/GuideHero.vue'
import InfoTip from '@/components/ui/InfoTip.vue'

const changeOpen = ref(false)
const guideOpen = ref(false)
const packageOpen = ref(false)

const planDisplay = computed(() => {
  const p = currentUser.value?.plan?.toLowerCase() ?? 'free'
  switch (p) {
    case 'starter':
      return { name: 'Starter', desc: 'Creator & Freelance Suite (฿390/ด.)', badge: 'bg-emerald-500/15 text-emerald-700' }
    case 'growth':
    case 'pro':
      return { name: 'Growth', desc: 'SME Marketing Team Suite (฿1,290/ด.)', badge: 'bg-sky-500/15 text-sky-700' }
    case 'agency':
    case 'business':
      return { name: 'Agency Pro', desc: 'The Wedge Proofing & Guest Portal (฿3,900/ด.)', badge: 'bg-ink text-surface' }
    case 'enterprise':
      return { name: 'Enterprise', desc: 'Unlimited Multi-Brand Operations (฿8,900/ด.)', badge: 'bg-purple-500/15 text-purple-700' }
    default:
      return { name: 'Free', desc: 'Community Edition (Local Planner)', badge: 'bg-wash text-muted border border-line' }
  }
})

async function doLogout(): Promise<void> {
  await logout()
}

async function doLogoutAll(): Promise<void> {
  await logoutAll()
}
</script>

<template>
  <section class="settings-section settings-profile-section">
    <div class="settings-section-head">
      <h2 class="settings-title flex items-center">
        <span>My Profile &amp; Account</span>
        <InfoTip text="Manage your display name, active sign-in sessions, package tier, and account actions." />
      </h2>
    </div>
    <div class="settings-profile-card">
      <div class="pop-block"><ProfilePanel /></div>

      <!-- Current Subscription Package Card -->
      <div v-if="currentUser" class="mt-4 p-4 rounded-xl border border-line bg-wash/40 flex flex-col sm:flex-row items-start sm:items-center justify-between gap-3">
        <div class="flex items-center gap-3 min-w-0">
          <div class="w-9 h-9 rounded-full bg-ink text-surface flex items-center justify-center font-bold text-xs shrink-0 shadow-xs">
            ★
          </div>
          <div class="min-w-0">
            <div class="flex items-center gap-2">
              <span class="text-[11px] uppercase tracking-wider text-muted font-bold">Package</span>
              <span class="px-2 py-0.5 rounded-full text-xs font-bold" :class="planDisplay.badge">
                {{ planDisplay.name }}
              </span>
            </div>
            <p class="text-xs text-ink/80 font-medium m-0 mt-0.5 truncate">{{ planDisplay.desc }}</p>
          </div>
        </div>
        <button
          type="button"
          class="btn btn-primary w-full sm:w-auto text-xs px-4 py-2 rounded-xl flex-none shadow-xs font-semibold"
          @click="packageOpen = true"
        >
          {{ planDisplay.name === 'Free' ? 'Upgrade Package' : 'Change Package' }}
        </button>
      </div>

      <div v-if="currentUser" class="settings-actions-bar pop-actions">
        <button class="btn menubtn" @click="changeOpen = true">{{ t('auth.change') }}</button>
        <button class="btn menubtn" @click="guideOpen = true">{{ t('nav.guide') }}</button>
        <button class="btn menubtn" @click="doLogoutAll">{{ t('auth.logoutAll') }}</button>
        <button class="btn btn-danger menubtn" @click="doLogout">{{ t('auth.logout') }}</button>
      </div>
    </div>
  </section>

  <!-- In-App Package Selection Modal -->
  <PackageModal :open="packageOpen" @close="packageOpen = false" />

  <ChangePasswordModal :open="changeOpen" @close="changeOpen = false" />
  <div v-if="guideOpen" class="guide-overlay" @click.self="guideOpen = false">
    <div class="guide-panel" role="dialog" aria-modal="true" aria-label="Guide">
      <div class="guide-head">
        <strong style="font-size: 20px;">{{ t('nav.guide') }}</strong>
        <button class="btn" @click="guideOpen = false">{{ t('common.close') }}</button>
      </div>
      <GuideHero />
    </div>
  </div>
</template>
