<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import AppShell from '@/components/layout/AppShell.vue'
import ErrorBoundary from '@/components/ui/ErrorBoundary.vue'
import ScreensDeck from '@/components/layout/ScreensDeck.vue'
import LoginPage from '@/pages/LoginPage.vue'
import WorkspaceSetupPage from '@/pages/WorkspaceSetupPage.vue'
import { restore } from '@/core/auth'
import { rememberLoginRedirect } from '@/core/login-redirect'
import { useBrand, useBrandFonts, usePermission, useSetup, useWorkspaces } from '@/core/queries'
import { applyBrandTheme, persistBrandTheme } from '@/core/theme'
import { t } from '@/core/i18n'

const route = useRoute()
const router = useRouter()
const { authRequired, isLoggedIn } = usePermission()
const { data: setup, isPending: setupPending, refetch: refetchSetup } = useSetup()
const restored = ref(false)

// A signed-in account always needs at least one workspace before the planner
// is usable; workspaces keep each account's brands and connections separate.
const {
  data: workspaces,
  isSuccess: workspacesLoaded,
  isPending: workspacesPending,
  isFetching: workspacesFetching,
  isError: workspacesError,
  refetch: refetchWorkspaces,
} = useWorkspaces()

// A query that raced the anonymous boot must be replayed as soon as a
// session exists; this is what makes post-signup workspace onboarding land
// on the create-workspace screen instead of an empty app shell.
watch(
  isLoggedIn,
  (loggedIn) => {
    if (loggedIn) void refetchWorkspaces()
  },
  { immediate: true },
)

// The brand's palette/fonts (CI) theme the whole site — public data.
const { data: brand } = useBrand()
const { data: fontAssets } = useBrandFonts()
watch([brand, fontAssets], () => {
  applyBrandTheme(brand.value, fontAssets.value ?? [])
  // Remember it for the next boot — but never persist the pending state
  // (undefined), or a reload would wipe a good remembered theme.
  if (brand.value !== undefined) persistBrandTheme(brand.value, fontAssets.value ?? [])
}, { immediate: true, deep: true })

// Restore the session once, before deciding between the login gate and the app.
onMounted(async () => {
  await restore().catch(() => undefined)
  restored.value = true
  // Establishing the session clears the query cache, which can strand the
  // boot-critical setup query; replay it so boot never waits forever.
  if (setup.value === undefined) void refetchSetup()
})

const slowBoot = ref(false)
const slowWorkspace = ref(false)
let bootTimer: number | undefined
let workspaceTimer: number | undefined

onMounted(() => {
  bootTimer = window.setTimeout(() => { slowBoot.value = true }, 6_000)
})
onBeforeUnmount(() => {
  window.clearTimeout(bootTimer)
  window.clearTimeout(workspaceTimer)
})

function reloadPage(): void {
  window.location.reload()
}

const ready = computed(() => restored.value && !setupPending.value)
const showLogin = computed(() => ready.value && authRequired.value && !isLoggedIn.value)
watch(
  () => showLogin.value && !route.meta.public && route.path !== '/login',
  (shouldRedirect) => {
    if (!shouldRedirect) return
    rememberLoginRedirect(route.path)
    void router.replace('/login')
  },
  { immediate: true },
)
const workspaceGatePending = computed(
  () =>
    ready.value &&
    !route.meta.public &&
    isLoggedIn.value &&
    !workspacesLoaded.value &&
    (workspacesPending.value || workspacesFetching.value),
)
const workspaceGateError = computed(
  () =>
    ready.value &&
    !route.meta.public &&
    isLoggedIn.value &&
    workspacesError.value &&
    !workspacesFetching.value,
)
watch(workspaceGatePending, (pending) => {
  window.clearTimeout(workspaceTimer)
  slowWorkspace.value = false
  if (pending) {
    workspaceTimer = window.setTimeout(() => { slowWorkspace.value = true }, 6_000)
  }
})
const showWorkspaceSetup = computed(
  () =>
    ready.value &&
    !route.meta.public &&
    isLoggedIn.value &&
    workspacesLoaded.value &&
    (workspaces.value?.length ?? 0) === 0,
)
</script>

<template>
  <!-- Public content delivery renders without the planner shell. -->
  <RouterView v-if="route.meta.public" />
  <!-- Production gate: nothing to do before signing in. -->
  <LoginPage v-else-if="showLogin" />
  <!-- Boot: a branded loading stage instead of a blank white page. -->
  <div v-else-if="!ready || workspaceGatePending" class="huuk-boot">
    <span class="huuk-boot-ambient" aria-hidden="true"></span>
    <div class="huuk-boot-stage">
      <div class="huuk-boot-mark" aria-hidden="true">
        <span class="huuk-boot-orbit"></span>
        <span class="huuk-boot-orbit is-two"></span>
        <span class="huuk-boot-monogram">H</span>
      </div>
      <p class="huuk-boot-brand">Huuk<span>by teenueng</span></p>
      <p class="huuk-boot-title">Getting your studio ready</p>
      <p class="huuk-boot-sub">
        {{ workspaceGatePending ? 'Loading your workspace…' : 'Restoring your session…' }}
      </p>
      <div class="huuk-boot-track" aria-hidden="true"><span class="huuk-boot-sweep"></span></div>
      <p class="huuk-boot-status"><span class="huuk-boot-pulse" aria-hidden="true"></span>Loading</p>
      <div v-if="slowBoot || slowWorkspace" class="huuk-boot-slow">
        <p>This is taking longer than usual. Check your connection and try again.</p>
        <button type="button" class="btn w-auto" @click="reloadPage">{{ t('common.reload') }}</button>
      </div>
    </div>
  </div>
  <!-- Workspace lookup failed: give the user a retry instead of an empty shell. -->
  <div v-else-if="workspaceGateError" class="min-h-screen grid place-items-center p-6 bg-wash">
    <div class="card p-6 max-w-sm w-full text-center grid gap-3">
      <p class="font-semibold text-ink m-0">{{ t('ws.loadError') }}</p>
      <button type="button" class="btn w-auto justify-self-center" @click="refetchWorkspaces()">
        {{ t('common.tryAgain') }}
      </button>
    </div>
  </div>
  <!-- First run: a signed-in account without a workspace creates one first. -->
  <WorkspaceSetupPage v-else-if="showWorkspaceSetup" />
  <AppShell v-else>
    <ErrorBoundary>
      <!-- Standalone pages (Content Studio) skip the card deck. -->
      <RouterView v-if="route.meta.standalone" />
      <ScreensDeck v-else />
    </ErrorBoundary>
  </AppShell>
</template>
