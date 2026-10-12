<script setup lang="ts">
// First-run onboarding: a signed-in account without any workspace lands here
// to create its first one. Workspaces keep brands and social accounts apart,
// so this happens before the planner is usable.
import { computed, ref } from 'vue'
import { useRouter } from 'vue-router'
import { useQueryClient } from '@tanstack/vue-query'
import { useCreateWorkspace } from '@/core/queries'
import { setActiveWorkspace } from '@/core/workspace'
import { currentName } from '@/core/auth'
import { t } from '@/core/i18n'

const router = useRouter()
const qc = useQueryClient()
const createWorkspace = useCreateWorkspace()

const name = ref('')
const error = ref('')
const busy = computed(() => createWorkspace.isPending.value)

async function submit(): Promise<void> {
  const value = name.value.trim()
  if (!value || busy.value) return
  error.value = ''
  try {
    const workspace = await createWorkspace.mutateAsync(value)
    setActiveWorkspace(workspace.id)
    qc.invalidateQueries()
    await router.replace('/')
  } catch (e) {
    error.value = e instanceof Error ? e.message : String(e)
  }
}
</script>

<template>
  <div class="min-h-screen grid place-items-center p-6 bg-wash">
    <form class="w-[min(460px,100%)] flex flex-col gap-2.5 p-8 bg-surface border border-faint rounded-[14px] shadow-lg" @submit.prevent="submit">
      <div class="font-extrabold tracking-wide text-accent">Huuk</div>
      <h1 class="m-0 text-2xl font-bold text-ink">{{ t('ws.onboardingTitle') }}</h1>
      <p class="text-sm text-muted">{{ t('ws.onboardingBody') }}</p>
      <p v-if="currentName" class="m-0 text-[13px] text-muted">{{ t('ws.onboardingHello') }} {{ currentName }}</p>

      <label class="lbl text-sm font-medium text-ink" for="ws-onboarding-name">{{ t('ws.nameLabel') }}</label>
      <input id="ws-onboarding-name" v-model="name" class="field" maxlength="80" autofocus
        :placeholder="t('ws.namePlaceholder')" />
      <p v-if="error" class="autherr text-xs text-danger" role="alert">{{ error }}</p>

      <button type="submit" class="btn btn-primary mt-1.5" :disabled="!name.trim() || busy">
        {{ busy ? t('common.loading') : t('ws.create') }}
      </button>
      <p class="m-0 mt-0.5 text-xs text-muted">{{ t('ws.onboardingHint') }}</p>
    </form>
  </div>
</template>
