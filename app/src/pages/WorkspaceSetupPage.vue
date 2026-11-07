<script setup lang="ts">
// First-run onboarding: a signed-in account without any workspace lands here
// to create its first one. Workspaces keep brands and social accounts apart,
// so this happens before the planner is usable.
import { computed, ref } from 'vue'
import { useRouter } from 'vue-router'
import { useQueryClient } from '@tanstack/vue-query'
import { useCreateWorkspace } from '@/core/queries'
import { currentName, logout, restore } from '@/core/auth'
import { setActiveWorkspace } from '@/core/workspace'
import { t } from '@/core/i18n'

const router = useRouter()
const qc = useQueryClient()
const createWorkspace = useCreateWorkspace()

const name = ref('')
const error = ref('')
const signingOut = ref(false)
const busy = computed(() => createWorkspace.isPending.value)
const signedInName = computed(() => currentName.value ?? '')

function friendlyError(message: string): string {
  return /missing permission/i.test(message) ? t('ws.noPermission') : message
}

async function signOut(): Promise<void> {
  if (signingOut.value) return
  signingOut.value = true
  try {
    qc.clear()
    await logout()
    await router.replace('/login')
  } finally {
    signingOut.value = false
  }
}

async function submit(): Promise<void> {
  const value = name.value.trim()
  if (!value || busy.value) return
  error.value = ''
  try {
    const workspace = await createWorkspace.mutateAsync(value)
    setActiveWorkspace(workspace.id)
    // First-workspace bootstrap upgrades the starter Viewer role on the
    // server; refresh the session user before permission checks render.
    await restore()
    await qc.invalidateQueries()
    // Continue straight into first-run brand onboarding. Keeping the query
    // flag makes BrandPage ask for the name, background, logo, colours,
    // fonts and voice instead of dropping the user onto a prefilled demo.
    await router.replace({ path: '/branding', query: { tab: 'brand', onboarding: '1' } })
  } catch (e) {
    error.value = friendlyError(e instanceof Error ? e.message : String(e))
  }
}
</script>

<template>
  <div class="min-h-screen grid place-items-center p-6 bg-wash">
    <form class="form-onboarding flex flex-col gap-2.5 p-8 bg-surface border border-faint rounded-panel shadow-lg" @submit.prevent="submit">
      <img src="/huuk-logo.svg" alt="Huuk by teenueng" class="h-9 w-auto self-start" />
      <h1 class="m-0 text-2xl font-bold text-ink">{{ t('ws.onboardingTitle') }}</h1>

      <label class="lbl text-sm font-medium text-ink" for="ws-onboarding-name">{{ t('ws.nameLabel') }}</label>
      <input id="ws-onboarding-name" v-model="name" class="field" maxlength="80" autofocus
        :placeholder="t('ws.namePlaceholder')" />
      <p v-if="error" class="autherr text-xs text-danger" role="alert">{{ error }}</p>

      <button type="submit" class="btn btn-primary mt-1.5" :disabled="!name.trim() || busy">
        {{ busy ? t('common.loading') : t('ws.create') }}
      </button>

      <div class="flex items-center justify-between gap-3 pt-2 border-t border-faint">
        <span class="text-xs text-muted min-w-0 truncate">
          <template v-if="signedInName">{{ t('ws.signedInAs', { name: signedInName }) }}</template>
        </span>
        <button type="button" class="btn w-auto text-xs py-1.5 px-3 flex-none" :title="t('ws.signOutHint')" :disabled="signingOut" @click="signOut">
          {{ signingOut ? t('common.loading') : t('ws.signOut') }}
        </button>
      </div>
    </form>
  </div>
</template>
