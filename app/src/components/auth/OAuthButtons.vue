<script setup lang="ts">
import { computed, onMounted, reactive } from 'vue'
import { BASE } from '@/api/client'
import { t } from '@/core/i18n'

const providers = reactive({
  google: false,
  github: false,
})
const anyReady = computed(() => providers.google || providers.github)

onMounted(async () => {
  const check = async (id: 'google' | 'github'): Promise<void> => {
    try {
      const res = await fetch(`${BASE}/api/auth/oauth/${id}/status`)
      const data = res.ok ? await res.json() : null
      providers[id] = Boolean(data?.configured)
    } catch {
      providers[id] = false
    }
  }
  await Promise.all([check('google'), check('github')])
})
</script>

<template>
  <div v-if="anyReady" class="mt-4">
    <div class="flex items-center gap-3 text-muted text-2xs uppercase tracking-wider my-4">
      <span class="h-px flex-1 bg-line" />
      <span>{{ t('auth.or') }}</span>
      <span class="h-px flex-1 bg-line" />
    </div>
    <div class="flex flex-col gap-2">
      <a
        v-if="providers.google"
        class="btn w-full justify-center text-sm py-2.5"
        :href="`${BASE}/api/auth/oauth/google/start`"
      >{{ t('auth.continueWithGoogle') }}</a>
      <a
        v-if="providers.github"
        class="btn w-full justify-center text-sm py-2.5"
        :href="`${BASE}/api/auth/oauth/github/start`"
      >{{ t('auth.continueWithGithub') }}</a>
    </div>
  </div>
</template>
