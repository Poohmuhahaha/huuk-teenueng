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
  <div class="socialbar" :title="t('social.title')">
    <span class="socialbar-title">{{ t('social.title') }}</span>
    <label v-for="p in PLATFORMS" :key="p.id" class="socialcheck"
      :class="{ on: activePlatforms.includes(p.id) }" :title="`Include ${p.name}`">
      <input type="checkbox" :checked="activePlatforms.includes(p.id)" :aria-label="`Include ${p.name}`"
        @change="onToggle(p.id, $event)" />
      <span class="status-dot" :class="statusOf(p.id)" />
      <template v-if="linkOf(p.id)">
        <a class="socialname" :href="linkOf(p.id) as string" target="_blank" rel="noopener noreferrer"
          :title="`${p.name} — ${t('social.openProfile')}`">{{ p.name }}</a>
        <button class="socialmini" :aria-label="`${t('social.manage')} ${p.name}`"
          :title="`${t('social.manage')} ${p.name}`" @click.prevent.stop="loginFor = p.id">
          {{ t('social.manageShort') }}
        </button>
      </template>
      <button v-else class="socialname" :title="`${p.name} — ${t('social.connect')}`"
        @click.prevent.stop="loginFor = p.id">
        {{ p.name }}
      </button>
    </label>
    <button class="socialbar-manage" :title="t('social.manageHint')"
      @click="loginFor = activePlatforms[0] ?? null">{{ t('social.manage') }}…</button>
    <PlatformLogin :platform="loginFor" @close="loginFor = null" />
  </div>
</template>
