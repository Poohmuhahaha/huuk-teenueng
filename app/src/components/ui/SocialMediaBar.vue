<script setup lang="ts">
// Social Media switcher on the Feed page (workspace body, never the navbar):
// check platforms to include them in the workspace; connected platforms link
// to their real profile and open the connection dialog from "Manage".
import { ref } from 'vue'
import { usePlatforms } from '@/core/queries'
import { activePlatforms, PLATFORMS, profileUrl, togglePlatform } from '@/core/platforms'
import type { PlatformId } from '@/core/platforms'
import { t } from '@/core/i18n'
import PlatformLogin from '@/components/overlays/PlatformLogin.vue'
import PlatformLogo from '@/components/ui/PlatformLogo.vue'

const { data: connections } = usePlatforms()
const loginFor = ref<PlatformId | null>(null)

function connectionOf(id: PlatformId) {
  return connections.value?.find((c) => c.id === id) ?? null
}

function statusOf(id: PlatformId): string {
  return connectionOf(id)?.status ?? 'disconnected'
}

function onToggle(id: PlatformId, event: Event): void {
  togglePlatform(id)
  // A refused toggle (e.g. unchecking the last platform) must snap the DOM back.
  const input = event.target as HTMLInputElement
  input.checked = activePlatforms.value.includes(id)
}

function linkOf(id: PlatformId): string | null {
  const conn = connectionOf(id)
  if (!conn || conn.status !== 'connected') return null
  return profileUrl(id, conn.handle)
}
</script>

<template>
  <div class="flex flex-wrap items-center gap-2 min-w-0 mb-3" role="group" :aria-label="t('social.title')" :title="t('social.title')">
    <span class="tracking-[0.08em] uppercase text-muted flex-none text-2xs font-extrabold max-sm:hidden">{{ t('social.title') }}</span>
    <div v-for="p in PLATFORMS" :key="p.id" class="inline-flex items-center gap-[7px] px-[13px] py-[5px] rounded-full border text-[13.5px] font-[550] flex-none transition-colors hover:bg-wash hover:text-ink"
      :class="activePlatforms.includes(p.id) ? 'on bg-ink text-accent-ink border-ink font-[650]' : 'bg-surface text-muted border-faint'" :title="`Include ${p.name}`">
      <input type="checkbox" class="m-0 accent-accent" :checked="activePlatforms.includes(p.id)" :aria-label="`Include ${p.name}`"
        @change="onToggle(p.id, $event)" />
      <span class="flex-none inline-block w-2 h-2 rounded-full border-[1.5px]" :class="statusOf(p.id) === 'connected' ? (activePlatforms.includes(p.id) ? 'bg-accent-ink border-accent-ink' : 'bg-success border-success') : statusOf(p.id) === 'connecting' ? 'bg-warning border-warning' : statusOf(p.id) === 'error' ? 'border-dashed border-danger bg-surface' : 'bg-surface border-muted'" />
      <PlatformLogo :id="p.id" :size="15" />
      <template v-if="linkOf(p.id)">
        <a class="socialname text-inherit cursor-pointer bg-transparent border-0 p-0 hover:underline w-auto" :href="linkOf(p.id) as string" target="_blank" rel="noopener noreferrer"
          :title="`${p.name} — ${t('social.openProfile')}`">{{ p.name }}</a>
        <button class="cursor-pointer text-inherit opacity-70 hover:opacity-100 bg-transparent border-0 pl-1.5 p-0 text-2xs font-bold w-auto flex-none inline-flex items-center" :aria-label="`${t('social.manage')} ${p.name}`"
          :title="`${t('social.manage')} ${p.name}`" @click="loginFor = p.id">
          <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
            <circle cx="12" cy="12" r="3" />
            <path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z" />
          </svg>
        </button>
      </template>
      <button v-else class="text-inherit cursor-pointer bg-transparent border-0 p-0 hover:underline w-auto flex-none" :title="`${p.name} — ${t('social.connect')}`"
        @click="loginFor = p.id">
        {{ p.name }}
      </button>
    </div>
    <button v-if="activePlatforms.length" class="bg-surface border border-faint rounded-full cursor-pointer text-muted flex-none ml-auto px-[13px] py-[5px] text-sm font-semibold transition-colors hover:bg-accent hover:text-accent-ink hover:border-accent w-auto inline-flex items-center gap-1.5" :title="t('social.manageHint')"
      @click="loginFor = activePlatforms[0] ?? null">
      <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
        <circle cx="12" cy="12" r="3" />
        <path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z" />
      </svg>
      <span>{{ t('social.manage') }}…</span>
    </button>
    <PlatformLogin :platform="loginFor" @close="loginFor = null" />
  </div>
</template>
