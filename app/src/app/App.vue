<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue'
import { useRoute } from 'vue-router'
import AppShell from '@/components/layout/AppShell.vue'
import ErrorBoundary from '@/components/ui/ErrorBoundary.vue'
import ScreensDeck from '@/components/layout/ScreensDeck.vue'
import LoginPage from '@/pages/LoginPage.vue'
import WorkspaceSetupPage from '@/pages/WorkspaceSetupPage.vue'
import { restore } from '@/core/auth'
import { useBrand, useBrandFonts, usePermission, useSetup, useWorkspaces } from '@/core/queries'
import { applyBrandTheme, persistBrandTheme } from '@/core/theme'
import { t } from '@/core/i18n'

const route = useRoute()
const { authRequired, isLoggedIn } = usePermission()
const { isPending: setupPending } = useSetup()
const restored = ref(false)

// A signed-in account always needs at least one workspace before the planner
// is usable; workspaces keep each account's brands and connections separate.
const { data: workspaces, isSuccess: workspacesLoaded } = useWorkspaces()

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
})

const ready = computed(() => restored.value && !setupPending.value)
const showLogin = computed(() => ready.value && authRequired.value && !isLoggedIn.value)
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
  <!-- Boot Loading Animation Screen: 1:1 Squares Wave Loader -->
  <div
    v-else-if="!ready"
    class="min-h-screen w-full flex flex-col items-center justify-center bg-surface select-none relative overflow-hidden"
  >
    <!-- Ambient subtle background glow -->
    <div
      class="absolute w-80 h-80 rounded-full bg-wash blur-3xl -z-10 pointer-events-none opacity-80"
    />

    <div class="flex flex-col items-center text-center gap-6 px-6 max-w-sm">
      <!-- Huuk Brand Logo -->
      <img
        src="/huuk-logo.svg"
        alt="Huuk"
        class="h-8 sm:h-9 w-auto object-contain opacity-90 transition-opacity"
      />

      <!-- 1:1 Squares Loading Wave -->
      <div class="flex items-center gap-2.5 sm:gap-3 py-2" role="status" aria-label="Loading">
        <div
          v-for="i in 5"
          :key="i"
          class="square-loader-item w-7 h-7 sm:w-8 sm:h-8 aspect-square flex-none"
          :style="{ animationDelay: `${(i - 1) * 0.14}s` }"
        />
      </div>

      <!-- Dynamic Loading Status with Animated Bouncing Dots -->
      <div class="flex items-center gap-1.5 text-xs font-semibold text-muted tracking-wider uppercase">
        <span>{{ t('common.loading').replace(/…|\.\.\./, '') }}</span>
        <span class="inline-flex gap-1 items-center">
          <span class="w-1 h-1 rounded-full bg-muted/70 animate-bounce [animation-delay:0ms]" />
          <span class="w-1 h-1 rounded-full bg-muted/70 animate-bounce [animation-delay:150ms]" />
          <span class="w-1 h-1 rounded-full bg-muted/70 animate-bounce [animation-delay:300ms]" />
        </span>
      </div>
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
