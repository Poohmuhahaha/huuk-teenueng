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
  <div class="wspage">
    <form class="wscard" @submit.prevent="submit">
      <div class="wsbrand">Huuk</div>
      <h1>{{ t('ws.onboardingTitle') }}</h1>
      <p class="muted">{{ t('ws.onboardingBody') }}</p>
      <p v-if="currentName" class="muted wsuser">{{ t('ws.onboardingHello') }} {{ currentName }}</p>

      <label class="lbl" for="ws-onboarding-name">{{ t('ws.nameLabel') }}</label>
      <input id="ws-onboarding-name" v-model="name" class="field" maxlength="80" autofocus
        :placeholder="t('ws.namePlaceholder')" />
      <p v-if="error" class="autherr" role="alert">{{ error }}</p>

      <button type="submit" class="btn btn-primary wsgo" :disabled="!name.trim() || busy">
        {{ busy ? t('common.loading') : t('ws.create') }}
      </button>
      <p class="muted wshint">{{ t('ws.onboardingHint') }}</p>
    </form>
  </div>
</template>

<style scoped>
.wspage {
  min-height: 100vh;
  display: grid;
  place-items: center;
  padding: 24px;
  background: var(--wash, #f6f7f9);
}
.wscard {
  width: min(460px, 100%);
  display: flex;
  flex-direction: column;
  gap: 10px;
  padding: 32px;
  background: var(--surface, #fff);
  border: 1px solid var(--faint, #e6e6e6);
  border-radius: var(--radius, 14px);
  box-shadow: var(--shadow-lg, 0 18px 40px rgba(0, 0, 0, 0.08));
}
.wsbrand {
  font-weight: 800;
  letter-spacing: 0.02em;
  color: var(--accent, #0b7a75);
}
.wscard h1 {
  margin: 0;
  font-size: 24px;
}
.wsuser {
  margin: 0;
  font-size: 13px;
}
.wsgo {
  margin-top: 6px;
}
.wshint {
  margin: 2px 0 0;
  font-size: 12px;
}
</style>
