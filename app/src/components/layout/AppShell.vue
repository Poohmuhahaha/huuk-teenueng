<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useQueryClient } from '@tanstack/vue-query'
import {
  useOAuthPending, useChooseOAuthPage, qk,
} from '@/core/queries'
import { currentRole, currentUser } from '@/core/auth'
import { t } from '@/core/i18n'
import { PLATFORM_META } from '@/core/platforms'
import type { PlatformId } from '@/core/platforms'
import LoginModal from '@/components/overlays/LoginModal.vue'
import MainNavPicker from './MainNavPicker.vue'
import Subnavbar from './Subnavbar.vue'

const route = useRoute()
const router = useRouter()
// Brand tab owns the topbar (long main-nav picker, no logo/burger); its
// settings tabs live in the shared below-bar like every other hub.
// Missing/invalid ?tab= falls back to 'brand', mirroring useHubTab.
const onBrand = computed(
  () => (route.path === '/settings' || route.path.startsWith('/settings/'))
    && !['workspace', 'members', 'connections'].includes(route.query.tab as string),
)
const qc = useQueryClient()

// OAuth callback landing: /#/setup?oauth=meta&status=ok (redirects to /profile)
const notice = ref('')
let noticeTimer: number | undefined

function showNotice(text: string): void {
  notice.value = text
  window.clearTimeout(noticeTimer)
  noticeTimer = window.setTimeout(() => (notice.value = ''), 6000)
}

// Multi-page OAuth: Meta returned several Pages and the user picks one.
const pickReq = ref<{ platform: PlatformId; pick: string } | null>(null)
const pickError = ref('')
const choosingId = ref('')
const pickQ = useOAuthPending(
  computed(() => pickReq.value?.platform ?? ''),
  computed(() => pickReq.value?.pick ?? ''),
)
const choosePage = useChooseOAuthPage()

function closePick(): void {
  pickReq.value = null
  pickError.value = ''
  choosingId.value = ''
}

async function chooseCandidate(externalId: string): Promise<void> {
  if (!pickReq.value || choosingId.value) return
  choosingId.value = externalId
  pickError.value = ''
  try {
    await choosePage.mutateAsync({ id: pickReq.value.platform, pick: pickReq.value.pick, external_id: externalId })
    qc.invalidateQueries({ queryKey: qk.metrics })
    const name = PLATFORM_META[pickReq.value.platform]?.name ?? pickReq.value.platform
    showNotice(`${name} ${t('slogin.connected')}`)
    pickReq.value = null
  } catch (e) {
    pickError.value = e instanceof Error ? e.message : String(e)
  } finally {
    choosingId.value = ''
  }
}

watch(
  () => route.query,
  (q) => {
    const id = typeof q.oauth === 'string' ? q.oauth : ''
    if (!id) return
    const meta = PLATFORM_META[id as PlatformId]
    const pick = typeof q.pick === 'string' ? q.pick : ''
    if (q.status === 'pick' && pick && meta) {
      pickReq.value = { platform: id as PlatformId, pick }
      pickError.value = ''
      const { oauth: _oauth, status: _status, reason: _reason, pick: _pick, ...rest } = q
      void router.replace({ query: rest })
      return
    }
    const name = meta?.name ?? id
    showNotice(
      q.status === 'ok'
        ? `${name} ${t('slogin.connected')}`
        : `${name} — ${t('slogin.error')}${typeof q.reason === 'string' && q.reason ? `: ${q.reason}` : ''}`,
    )
    qc.invalidateQueries({ queryKey: qk.platforms })
    qc.invalidateQueries({ queryKey: qk.live })
    qc.invalidateQueries({ queryKey: qk.metrics })
    // Only drop the OAuth keys — other query params stay meaningful.
    const { oauth: _oauth, status: _status, reason: _reason, pick: _pick, ...rest } = q
    void router.replace({ query: rest })
  },
  { immediate: true, deep: true },
)
onBeforeUnmount(() => window.clearTimeout(noticeTimer))

const authOpen = ref(false)
const authMode = ref<'login' | 'register'>('login')

const avatarLetter = computed(() =>
  currentUser.value ? currentUser.value.name.slice(0, 1).toUpperCase() : 'O',
)

function openLogin(mode: 'login' | 'register'): void {
  authMode.value = mode
  authOpen.value = true
}

// The avatar is a shortcut to the account sections on the Settings
// workspace tab — never a popup.
function goAccount(): void {
  navOpen.value = false
  if (!currentUser.value) {
    openLogin('login')
    return
  }
  void router.push({ path: '/settings', query: { tab: 'workspace' } })
}

function onAuthSuccess(mode: 'login' | 'register'): void {
  authOpen.value = false
  // Client accounts land in their content studio, not the internal planner.
  if (currentRole.value === 'Client') {
    void router.push('/content')
    return
  }
  if (mode !== 'register') return
  showNotice(t('profile.welcome'))
  void router.push({ path: '/settings', query: { tab: 'workspace' } })
}

// Client accounts belong in the studio: steer them away from planner pages.
watch(currentRole, (role) => {
  if (role !== 'Client') return
  if (route.meta.standalone || route.meta.public) return
  void router.replace('/content')
})

onMounted(() => {
  document.documentElement.lang = 'en'
})

interface NavLink {
  to: string
  label: string
}

const isClient = computed(() => currentRole.value === 'Client')

// Compact navbar: one link per hub (the compact IA). Clients only get Content.
// Branding leads: it is the workspace foundation and the deck's front card.
const navLinks = computed<NavLink[]>(() =>
  isClient.value
    ? [{ to: '/content', label: t('nav.content') }]
    : [
        { to: '/settings', label: t('nav.branding') },
        { to: '/dashboard', label: t('nav.home') },
        { to: '/plan', label: t('nav.plan') },
        { to: '/content', label: t('nav.content') },
        { to: '/promote', label: t('nav.promote') },
        { to: '/analyze', label: t('nav.analyze') },
      ],
)

function isActive(l: NavLink): boolean {
  // Segment match: '/plan' must not match '/plans'. '/' matches only itself.
  if (l.to === '/') return route.path === '/'
  return route.path === l.to || route.path.startsWith(l.to + '/')
}

// Liquid-glass navbar: phone sizes collapse the links behind a hamburger
// (Framer "Phone Open" variant); navigating always closes the menu.
const navOpen = ref(false)
watch(
  () => route.path,
  () => { navOpen.value = false },
)

</script>

<template>
  <div class="app" :class="{ 'on-brand': onBrand }">
    <header class="topbar" :class="{ open: navOpen }">
      <div class="brand">
        <RouterLink to="/" class="brand-home">
          <img class="brand-logo" src="/huuk-logo.svg" alt="Huuk by teenueng" />
        </RouterLink>
      </div>
      <MainNavPicker :links="navLinks" :expanded="navOpen" @toggle="navOpen = !navOpen" />
      <nav class="links">
        <div class="nav-toggle-group">
          <RouterLink v-for="l in navLinks" :key="l.to" :to="l.to" class="navlink"
            :class="{ active: isActive(l) }"
            :aria-current="isActive(l) ? 'page' : undefined">
            {{ l.label }}
          </RouterLink>
        </div>
      </nav>
      <button type="button" class="nav-burger" :class="{ open: navOpen }"
        :aria-expanded="navOpen ? 'true' : 'false'" aria-label="Menu"
        @click="navOpen = !navOpen">
        <span aria-hidden="true" />
        <span aria-hidden="true" />
      </button>
      <div class="side">
        <template v-if="!currentUser">
          <button class="btn btn-primary" @click="openLogin('register')">{{ t('auth.signup') }}</button>
          <button class="btn" @click="openLogin('login')">{{ t('auth.login') }}</button>
        </template>
        <div class="avatar-wrap">
          <button class="avatar" :title="t('profile.settings')" :aria-label="t('profile.settings')"
            @click="goAccount">
            {{ avatarLetter }}
          </button>
        </div>
      </div>
    </header>
    <Subnavbar class="below-topbar" />
    <div v-if="notice" class="oauth-banner" role="status" aria-live="polite">
      <span>{{ notice }}</span>
      <button class="btn" @click="notice = ''">{{ t('common.close') }}</button>
    </div>
    <div v-if="pickReq" class="guide-overlay" @click.self="closePick">
      <div class="guide-panel" role="dialog" aria-modal="true" :aria-label="t('slogin.pickTitle')">
        <div class="guide-head">
          <strong style="font-size: 20px;">{{ t('slogin.pickTitle') }}</strong>
          <button class="btn" @click="closePick">{{ t('common.close') }}</button>
        </div>
        <p class="muted">{{ t('slogin.pickHint') }}</p>
        <div v-if="pickQ.isPending.value" class="muted">{{ t('common.loading') }}</div>
        <div v-else-if="pickQ.isError.value" class="autherr" role="alert">{{ t('slogin.pickFailed') }}</div>
        <div v-else class="pick-list">
          <button
            v-for="a in pickQ.data.value?.accounts ?? []"
            :key="a.external_id"
            class="btn pick-row"
            :disabled="!!choosingId"
            @click="chooseCandidate(a.external_id)"
          >
            <span>{{ a.handle }}</span>
            <span class="muted">{{ choosingId === a.external_id ? t('slogin.connecting') : t('slogin.pickChoose') }}</span>
          </button>
        </div>
        <p v-if="pickError" class="autherr" role="alert">{{ pickError }}</p>
      </div>
    </div>
    <main class="main">
      <div class="shellgrid"><slot /></div>
    </main>
    <LoginModal :open="authOpen" :initial-mode="authMode" @close="authOpen = false" @success="onAuthSuccess" />
  </div>
</template>
