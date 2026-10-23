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
import Drawer from '@/components/overlays/Drawer.vue'
import WorkspaceSwitcher from '@/components/settings/WorkspaceSwitcher.vue'
import SettingsPanel from '@/components/overlays/SettingsPanel.vue'
import AccountPanel from '@/components/settings/AccountPanel.vue'
import MainNavPicker from './MainNavPicker.vue'
import Subnavbar from './Subnavbar.vue'
import MonthToggle from '@/components/ui/MonthToggle.vue'
import { activeMonth, activeYear } from '@/core/navdate'
import { deckPeek } from '@/core/deck'
import { getRememberedTab } from '@/core/subnav'

const route = useRoute()
const router = useRouter()
// Brand tab owns the topbar (long main-nav picker, no logo/burger); its
// settings tabs live in the shared below-bar like every other hub.
// Missing/invalid ?tab= falls back to 'brand', mirroring useHubTab.
const onBrand = computed(
  () => (route.path === '/branding' || route.path.startsWith('/branding/') || route.path === '/settings' || route.path.startsWith('/settings/'))
    && !['members', 'connections'].includes((route.query.tab as string) || getRememberedTab('/branding') || ''),
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

const workspaceOpen = ref(false)

function toggleWorkspace(): void {
  navOpen.value = false
  if (!currentUser.value) {
    openLogin('login')
    return
  }
  workspaceOpen.value = !workspaceOpen.value
}

// Watch for ?workspace=1 or legacy ?tab=workspace to pop open the modal
watch(
  () => route.query.tab,
  (tab) => {
    if (tab === 'workspace') {
      workspaceOpen.value = true
      const { tab: _t, ...rest } = route.query
      void router.replace({ query: rest })
    }
  },
  { immediate: true },
)

function onKeydown(e: KeyboardEvent): void {
  if (e.key === 'Escape' && workspaceOpen.value) {
    workspaceOpen.value = false
  }
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
  workspaceOpen.value = true
}

// Client accounts belong in the studio: steer them away from planner pages.
watch(currentRole, (role) => {
  if (role !== 'Client') return
  if (route.meta.standalone || route.meta.public) return
  void router.replace('/content')
})

onMounted(() => {
  document.documentElement.lang = 'en'
  window.addEventListener('keydown', onKeydown)
})

onBeforeUnmount(() => {
  window.removeEventListener('keydown', onKeydown)
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
        { to: '/branding', label: t('nav.branding') },
        { to: '/dashboard', label: t('nav.home') },
        { to: '/plan', label: t('nav.plan') },
        { to: '/content', label: t('nav.content') },
        { to: '/promote', label: t('nav.promote') },
        { to: '/analyze', label: t('nav.analyze') },
      ],
)

function targetFor(l: NavLink): { path: string; query?: Record<string, string> } {
  const remembered = getRememberedTab(l.to)
  if (remembered) {
    const { tab: _t, ...rest } = (route.query ?? {}) as Record<string, string>
    return { path: l.to, query: { ...rest, tab: remembered } }
  }
  return { path: l.to }
}

function isActive(l: NavLink): boolean {
  // Segment match: '/plan' must not match '/plans'. '/' matches only itself.
  const path = deckPeek.value ?? route.path
  if (l.to === '/') return path === '/'
  return path === l.to || path.startsWith(l.to + '/')
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
          <RouterLink v-for="l in navLinks" :key="l.to" :to="targetFor(l)" class="navlink"
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
        <div class="topbar-month">
          <MonthToggle v-model="activeMonth" v-model:year="activeYear" />
        </div>
        <template v-if="!currentUser">
          <button class="btn btn-primary" @click="openLogin('register')">{{ t('auth.signup') }}</button>
          <button class="btn" @click="openLogin('login')">{{ t('auth.login') }}</button>
        </template>
        <div class="avatar-wrap">
          <button class="avatar" :title="t('profile.settings')" :aria-label="t('profile.settings')"
            :aria-expanded="workspaceOpen ? 'true' : 'false'"
            @click="toggleWorkspace">
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
    <!-- OAuth Account Pick Sheet -->
    <Drawer
      :model-value="!!pickReq"
      :title="t('slogin.pickTitle')"
      side="bottom"
      @update:model-value="(val) => !val && closePick()"
    >
      <p class="muted" style="margin-top: 0;">{{ t('slogin.pickHint') }}</p>
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
    </Drawer>

    <!-- Workspace Settings Sheet (Profile Toggle) -->
    <Drawer
      v-model="workspaceOpen"
      title="Workspace"
      side="bottom"
    >
      <div class="workspace-panel settings-stack">
        <WorkspaceSwitcher />
        <SettingsPanel />
        <AccountPanel />
      </div>
    </Drawer>
    <main class="main">
      <div class="shellgrid"><slot /></div>
    </main>
    <LoginModal :open="authOpen" :initial-mode="authMode" @close="authOpen = false" @success="onAuthSuccess" />
  </div>
</template>
