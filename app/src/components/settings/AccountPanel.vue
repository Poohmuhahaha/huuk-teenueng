<script setup lang="ts">
// Account section — moved out of the avatar popup onto the Settings
// workspace tab: profile name, password change, guide and sign-out.
import { ref } from 'vue'
import { currentUser, logout, logoutAll } from '@/core/auth'
import { t } from '@/core/i18n'
import ProfilePanel from '@/components/overlays/ProfilePanel.vue'
import ChangePasswordModal from '@/components/overlays/ChangePasswordModal.vue'
import GuideHero from '@/components/ui/GuideHero.vue'

const changeOpen = ref(false)
const guideOpen = ref(false)

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
      <h2 class="settings-title">My Profile & Account</h2>
      <p class="settings-desc">Manage your display name, active sign-in sessions, and account actions.</p>
    </div>
    <div class="settings-profile-card">
      <div class="pop-block"><ProfilePanel /></div>
      <div v-if="currentUser" class="settings-actions-bar pop-actions">
        <button class="btn menubtn" @click="changeOpen = true">{{ t('auth.change') }}</button>
        <button class="btn menubtn" @click="guideOpen = true">{{ t('nav.guide') }}</button>
        <button class="btn menubtn" @click="doLogoutAll">{{ t('auth.logoutAll') }}</button>
        <button class="btn btn-danger menubtn" @click="doLogout">{{ t('auth.logout') }}</button>
      </div>
    </div>
  </section>
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
