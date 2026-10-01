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
import { applyBrandTheme } from '@/core/theme'
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
watch([brand, fontAssets], () => applyBrandTheme(brand.value, fontAssets.value ?? []), { immediate: true, deep: true })

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
  <div v-else-if="!ready" class="boot muted">{{ t('common.loading') }}</div>
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

<style scoped>
.boot {
  min-height: 100vh;
  display: grid;
  place-items: center;
  font-size: 14px;
}
</style>
