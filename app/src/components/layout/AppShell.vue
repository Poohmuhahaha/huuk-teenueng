<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useQueryClient } from '@tanstack/vue-query'
import {
  useSetup, usePermission, useOAuthPending, useChooseOAuthPage, qk,
  useWorkspaces, useCreateWorkspace, useRenameWorkspace, useDeleteWorkspace,
  usePlatforms,
} from '@/core/queries'
import { activeWorkspaceId, setActiveWorkspace } from '@/core/workspace'
import { currentRole, currentUser, logout, logoutAll } from '@/core/auth'
import { t } from '@/core/i18n'
import { plannerPathForDate } from '@/core/deeplink'
import { navDate } from '@/core/navdate'
import { PLATFORM_META, PLATFORMS } from '@/core/platforms'
import type { PlatformId } from '@/core/platforms'
import ChangePasswordModal from '@/components/overlays/ChangePasswordModal.vue'
import DatePickerPopup from '@/components/ui/DatePickerPopup.vue'
import GuideHero from '@/components/ui/GuideHero.vue'
import LoginModal from '@/components/overlays/LoginModal.vue'
import PlatformLogin from '@/components/overlays/PlatformLogin.vue'
import ProfilePanel from '@/components/overlays/ProfilePanel.vue'
import SettingsPanel from '@/components/overlays/SettingsPanel.vue'

const route = useRoute()
const router = useRouter()
const qc = useQueryClient()
const { data: setup } = useSetup()
const { can } = usePermission()

// ---- workspace switcher ----
// Every workspace owns its own brand + connections, so switching invalidates
// all queries; the HTTP client sends the active id on every request.
const { data: workspaces } = useWorkspaces()
const createWorkspace = useCreateWorkspace()
const renameWorkspace = useRenameWorkspace()
const deleteWorkspace = useDeleteWorkspace()

const wsOpen = ref(false)
const wsError = ref('')
const wsNewName = ref('')
const wsRenameDraft = ref('')
const wsRenaming = ref(false)
const wsRenameInput = ref<HTMLInputElement | null>(null)
const canManageWs = computed(() => can('platforms.manage'))

const activeWs = computed(() => {
  const list = workspaces.value ?? []
  return list.find((w) => w.id === activeWorkspaceId.value) ?? list[0] ?? null
})
const activeWsName = computed(() => activeWs.value?.name ?? setup.value?.workspaceName ?? 'workspace')

// A stored id can point at a deleted workspace: fall back to the first one.
watch(
  () => workspaces.value,
  (list) => {
    if (!list?.length) return
    if (!list.some((w) => w.id === activeWorkspaceId.value)) {
      setActiveWorkspace(list[0].id)
    }
  },
  { immediate: true },
)

function toggleWsMenu(): void {
  wsOpen.value = !wsOpen.value
  wsError.value = ''
  wsRenaming.value = false
}

function switchWs(id: string): void {
  wsOpen.value = false
  if (id === activeWs.value?.id) return
  setActiveWorkspace(id)
  // Everything the app shows is workspace-scoped — refetch it all.
  qc.invalidateQueries()
}

async function submitWsCreate(): Promise<void> {
  const name = wsNewName.value.trim()
  if (!name) return
  wsError.value = ''
  try {
    const ws = await createWorkspace.mutateAsync(name)
    wsNewName.value = ''
    setActiveWorkspace(ws.id)
    qc.invalidateQueries()
    wsOpen.value = false
  } catch (e) {
    wsError.value = e instanceof Error ? e.message : String(e)
  }
}

function startRenameWs(): void {
  wsRenaming.value = true
  wsRenameDraft.value = activeWs.value?.name ?? ''
  void nextTick(() => {
    wsRenameInput.value?.focus()
    wsRenameInput.value?.select()
  })
}

async function submitRenameWs(): Promise<void> {
  const name = wsRenameDraft.value.trim()
  const current = activeWs.value
  if (!current || !name || name === current.name) {
    wsRenaming.value = false
    return
  }
  wsError.value = ''
  try {
    await renameWorkspace.mutateAsync({ id: current.id, name })
    wsRenaming.value = false
  } catch (e) {
    wsError.value = e instanceof Error ? e.message : String(e)
  }
}

async function removeWs(): Promise<void> {
  const current = activeWs.value
  if (!current) return
  wsError.value = ''
  try {
    const res = await deleteWorkspace.mutateAsync(current.id)
    setActiveWorkspace(res.fallback || null)
    wsOpen.value = false
    qc.invalidateQueries()
  } catch (e) {
    wsError.value = e instanceof Error ? e.message : String(e)
  }
}

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

const profileOpen = ref(false)
const settingsOpen = ref(false)
const guideOpen = ref(false)
const authOpen = ref(false)
const authMode = ref<'login' | 'register'>('login')
const changeOpen = ref(false)

// ---- navbar date chooser ----
// A global date pick jumps to the Monthly Planner at that month; the Smart
// Calendar follows the same date via the shared navDate ref.
function goToDate(value: string | null): void {
  navDate.value = value
  const path = plannerPathForDate(value)
  if (path) void router.push(path)
}

// ---- connect section (avatar toggle) ----
// The social connect/manage entry lives in the profile toggle, next to the
// workspace switcher — never loose in the navbar. The dialog itself enforces
// the platforms.manage permission.
const { data: connections } = usePlatforms()
const loginFor = ref<PlatformId | null>(null)
const canManagePlatforms = computed(() => can('platforms.manage'))

function connStatus(id: PlatformId): string {
  return connections.value?.find((c) => c.id === id)?.status ?? 'disconnected'
}

function connLabel(id: PlatformId): string {
  return connStatus(id) === 'connected' ? t('social.manageShort') : t('social.connect')
}

const avatarLetter = computed(() =>
  currentUser.value ? currentUser.value.name.slice(0, 1).toUpperCase() : 'O',
)

async function doLogout(): Promise<void> {
  profileOpen.value = false
  await logout()
}

async function doLogoutAll(): Promise<void> {
  profileOpen.value = false
  await logoutAll()
}

function openLogin(mode: 'login' | 'register'): void {
  profileOpen.value = false
  authMode.value = mode
  authOpen.value = true
}

function openGuide(): void {
  profileOpen.value = false
  guideOpen.value = true
}

function openSettings(): void {
  profileOpen.value = false
  settingsOpen.value = true
}

function openChange(): void {
  profileOpen.value = false
  changeOpen.value = true
}

function onAuthSuccess(mode: 'login' | 'register'): void {
  authOpen.value = false
  // Client accounts land in their content studio, not the internal planner.
  if (currentRole.value === 'Client') {
    void router.push('/studio')
    return
  }
  if (mode !== 'register') return
  profileOpen.value = true
  showNotice(t('profile.welcome'))
}

function onDocClick(e: MouseEvent): void {
  const target = e.target as HTMLElement | null
  if (!target?.closest?.('.avatar-wrap')) profileOpen.value = false
  if (!target?.closest?.('.ws-wrap')) wsOpen.value = false
  if (!target?.closest?.('.navgroup')) openGroup.value = null
}
// Client accounts belong in the studio: steer them away from planner pages.
watch(currentRole, (role) => {
  if (role !== 'Client') return
  if (route.meta.standalone || route.meta.public) return
  void router.replace('/studio')
})

function onKeyGlobal(e: KeyboardEvent): void {
  if (e.key !== 'Escape') return
  if (guideOpen.value) guideOpen.value = false
  else if (settingsOpen.value) settingsOpen.value = false
  else if (wsOpen.value) wsOpen.value = false
  else if (profileOpen.value) profileOpen.value = false
  else if (openGroup.value) openGroup.value = null
}

onMounted(() => {
  document.documentElement.lang = 'en'
  document.addEventListener('click', onDocClick)
  window.addEventListener('keydown', onKeyGlobal)
})
onBeforeUnmount(() => {
  document.removeEventListener('click', onDocClick)
  window.removeEventListener('keydown', onKeyGlobal)
})

interface NavLink {
  to: string
  label: string
  match?: string[]
}

interface NavGroup {
  label: string
  items: NavLink[]
}

const isClient = computed(() => currentRole.value === 'Client')

// Grouped navbar: four dropdowns instead of ten flat links. Client accounts
// only get their studio; staff keep everything, with Members owner-only.
const groups = computed<NavGroup[]>(() => [
  {
    label: t('nav.gPlan'),
    items: [
      { to: '/brand', label: t('nav.brand'), match: ['/brand'] },
      { to: '/planner', label: t('nav.monthly'), match: ['/planner'] },
      { to: '/calendar', label: t('nav.calendar'), match: ['/calendar'] },
      { to: '/ideas', label: t('nav.ideas'), match: ['/ideas'] },
      { to: '/hashtags', label: t('nav.hashtags'), match: ['/hashtags'] },
    ],
  },
  {
    label: t('nav.gCreate'),
    items: [
      { to: '/feed', label: t('nav.feed') },
      { to: '/studio', label: t('nav.content'), match: ['/studio'] },
    ],
  },
  {
    label: t('nav.gPromote'),
    items: [
      { to: '/live', label: t('nav.live'), match: ['/live'] },
      { to: '/campaigns', label: t('nav.campaigns'), match: ['/campaigns'] },
      { to: '/ads', label: t('nav.ads'), match: ['/ads'] },
    ],
  },
  {
    label: t('nav.gAnalyze'),
    items: [
      { to: '/dashboard', label: t('nav.stats'), match: ['/dashboard', '/performance'] },
      { to: '/finance', label: t('nav.finance') },
    ],
  },
])

// Members is owner-only: the link follows the active workspace summary.
const membersLink = computed<NavLink | null>(() =>
  activeWs.value?.isOwner === true && !isClient.value
    ? { to: '/members', label: t('nav.members'), match: ['/members'] }
    : null,
)

const openGroup = ref<string | null>(null)

function toggleGroup(label: string): void {
  openGroup.value = openGroup.value === label ? null : label
}

function isActive(l: NavLink): boolean {
  if (l.match) return l.match.some((m) => route.path.startsWith(m))
  return route.path === l.to
}

function groupActive(g: NavGroup): boolean {
  return g.items.some(isActive)
}

// A route change from anywhere (deck slide, deep link) closes the menu.
watch(() => route.path, () => {
  openGroup.value = null
})

</script>

<template>
  <div class="app">
    <header class="topbar">
      <div class="brand">
        <RouterLink to="/" class="brand-home">
          <img class="brand-logo" src="/huuk-logo.svg" alt="Huuk by teenueng" />
        </RouterLink>
      </div>
      <nav class="links">
        <RouterLink v-if="isClient" to="/studio" class="navlink"
          :class="{ active: route.path.startsWith('/studio') }"
          :aria-current="route.path.startsWith('/studio') ? 'page' : undefined">
          {{ t('nav.content') }}
        </RouterLink>
        <template v-else>
          <div v-for="g in groups" :key="g.label" class="navgroup">
            <button type="button" class="navlink navdrop" :class="{ active: groupActive(g) }"
              :aria-current="groupActive(g) ? 'page' : undefined"
              aria-haspopup="menu" :aria-expanded="openGroup === g.label" @click.stop="toggleGroup(g.label)">
              {{ g.label }}
              <span class="ws-caret" aria-hidden="true">▾</span>
            </button>
            <div v-if="openGroup === g.label" class="navmenu" role="menu" :aria-label="g.label" @click.stop>
              <RouterLink v-for="l in g.items" :key="l.to" :to="l.to" class="navmenu-link"
                :class="{ active: isActive(l) }" role="menuitem"
                :aria-current="isActive(l) ? 'page' : undefined">
                {{ l.label }}
              </RouterLink>
            </div>
          </div>
          <RouterLink v-if="membersLink" :to="membersLink.to" class="navlink"
            :class="{ active: isActive(membersLink) }"
            :aria-current="isActive(membersLink) ? 'page' : undefined">
            {{ membersLink.label }}
          </RouterLink>
        </template>
      </nav>
      <div class="side">
        <template v-if="!currentUser">
          <button class="btn btn-primary" @click="openLogin('register')">{{ t('auth.signup') }}</button>
          <button class="btn" @click="openLogin('login')">{{ t('auth.login') }}</button>
        </template>
        <span v-if="!isClient" class="navdate">
          <DatePickerPopup :model-value="navDate" :title="t('calendar.title')" @update:model-value="goToDate" />
        </span>
        <div class="avatar-wrap">
          <button class="avatar" title="Owner menu" aria-label="Owner menu" aria-haspopup="dialog"
            :aria-expanded="profileOpen" @click.stop="profileOpen = !profileOpen">
            {{ avatarLetter }}
          </button>
          <div v-if="profileOpen" class="avatar-menu" role="dialog" aria-label="Account menu">
            <div class="pop-block"><ProfilePanel /></div>
            <div v-if="canManagePlatforms" class="pop-connect">
              <div class="ws-head">{{ t('social.title') }}</div>
              <div v-for="p in PLATFORMS" :key="p.id" class="conn-row">
                <span class="status-dot" :class="connStatus(p.id)" aria-hidden="true" />
                <span class="conn-name">{{ p.name }}</span>
                <button type="button" class="conn-btn" @click="loginFor = p.id">{{ connLabel(p.id) }}</button>
              </div>
            </div>
            <div class="ws-wrap">
              <button type="button" class="wire ws-trigger" :title="t('ws.switch')"
                aria-haspopup="menu" :aria-expanded="wsOpen" @click.stop="toggleWsMenu">
                {{ activeWsName }}
                <span class="ws-caret" aria-hidden="true">▾</span>
              </button>
              <div v-if="wsOpen" class="ws-menu" role="menu" :aria-label="t('ws.title')" @click.stop>
                <div class="ws-head">{{ t('ws.title') }}</div>
                <button v-for="w in workspaces ?? []" :key="w.id" type="button" class="ws-item" role="menuitem"
                  :class="{ active: w.id === activeWs?.id }" @click="switchWs(w.id)">
                  <span class="ws-name">{{ w.name }}</span>
                  <span class="ws-count">{{ w.connected }}/{{ w.total }}</span>
                  <span v-if="w.id === activeWs?.id" class="ws-check" aria-hidden="true">✓</span>
                </button>
                <p v-if="!(workspaces ?? []).length" class="muted" style="padding: 6px 10px;">{{ t('common.loading') }}</p>

                <div v-if="canManageWs" class="ws-actions">
                  <template v-if="wsRenaming">
                    <input ref="wsRenameInput" v-model="wsRenameDraft" class="field" maxlength="80"
                      :aria-label="t('ws.rename')" @keydown.enter.prevent="submitRenameWs"
                      @keydown.esc.prevent="wsRenaming = false" @blur="submitRenameWs" />
                  </template>
                  <template v-else>
                    <button type="button" class="menubtn" @click="startRenameWs">{{ t('ws.rename') }}</button>
                    <button type="button" class="menubtn danger" :disabled="(workspaces ?? []).length <= 1"
                      @click="removeWs">{{ t('ws.delete') }}</button>
                  </template>
                </div>

                <form v-if="canManageWs" class="ws-form" @submit.prevent="submitWsCreate">
                  <input v-model="wsNewName" class="field" maxlength="80" :placeholder="t('ws.newPlaceholder')"
                    :aria-label="t('ws.new')" />
                  <button type="submit" class="btn btn-primary" :disabled="!wsNewName.trim()">{{ t('ws.new') }}</button>
                </form>
                <p v-if="wsError" class="autherr" role="alert" style="margin: 6px 8px 0;">{{ wsError }}</p>
              </div>
            </div>
            <div class="pop-actions">
              <button class="menubtn" @click="openSettings">{{ t('profile.settings') }}</button>
              <button class="menubtn" @click="openGuide">{{ t('nav.guide') }}</button>
              <template v-if="currentUser">
                <button class="menubtn" @click="openChange">{{ t('auth.change') }}</button>
                <button class="menubtn" @click="doLogoutAll">{{ t('auth.logoutAll') }}</button>
                <button class="menubtn" @click="doLogout">{{ t('auth.logout') }}</button>
              </template>
              <button v-else class="menubtn" @click="openLogin('login')">{{ t('auth.login') }}</button>
            </div>
          </div>
        </div>
      </div>
    </header>
    <div v-if="notice" class="oauth-banner">
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
      <div class="shellgrid" :class="{ wide: route.meta.wide }"><slot /></div>
    </main>
    <footer class="footer">
      Huuk by Teeneung v0.1(Beta)
    </footer>
    <div v-if="guideOpen" class="guide-overlay" @click.self="guideOpen = false">
      <div class="guide-panel" role="dialog" aria-modal="true" aria-label="Guide">
        <div class="guide-head">
          <strong style="font-size: 20px;">{{ t('nav.guide') }}</strong>
          <button class="btn" @click="guideOpen = false">{{ t('common.close') }}</button>
        </div>
        <GuideHero />
      </div>
    </div>
    <div v-if="settingsOpen" class="guide-overlay" @click.self="settingsOpen = false">
      <div class="guide-panel" role="dialog" aria-modal="true" aria-label="Workspace settings">
        <div class="guide-head">
          <strong style="font-size: 20px;">{{ t('profile.settings') }}</strong>
          <button class="btn" @click="settingsOpen = false">{{ t('common.close') }}</button>
        </div>
        <div class="settings-body"><SettingsPanel /></div>
      </div>
    </div>
    <LoginModal :open="authOpen" :initial-mode="authMode" @close="authOpen = false" @success="onAuthSuccess" />
    <ChangePasswordModal :open="changeOpen" @close="changeOpen = false" />
    <PlatformLogin :platform="loginFor" @close="loginFor = null" />
  </div>
</template>
