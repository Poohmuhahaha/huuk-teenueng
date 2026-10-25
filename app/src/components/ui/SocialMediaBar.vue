<script setup lang="ts">
// Social Media switcher on the Feed page:
// In default mode (popup: false): renders the inline bar (for backward-compatibility & tests).
// In popup mode (popup: true): renders a sleek trigger button that opens an interactive Popup Modal.
import { ref } from 'vue'
import { usePlatforms } from '@/core/queries'
import { activePlatforms, PLATFORMS, profileUrl, togglePlatform } from '@/core/platforms'
import type { PlatformId } from '@/core/platforms'
import { t } from '@/core/i18n'
import PlatformLogin from '@/components/overlays/PlatformLogin.vue'
import PlatformLogo from '@/components/ui/PlatformLogo.vue'

withDefaults(
  defineProps<{ popup?: boolean }>(),
  { popup: false },
)

const isOpen = ref(false)
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
  <!-- Inline Bar (default when popup=false) -->
  <div v-if="!popup" class="flex flex-wrap items-center gap-2 min-w-0 mb-3" role="group" :aria-label="t('social.title')" :title="t('social.title')">
    <span class="tracking-[0.08em] uppercase text-muted flex-none text-2xs font-extrabold max-sm:hidden">{{ t('social.title') }}</span>
    <div v-for="p in PLATFORMS" :key="p.id" class="inline-flex items-center gap-[7px] px-[13px] py-[5px] rounded-full border text-[13.5px] font-[550] flex-none transition-colors hover:bg-wash hover:text-ink"
      :class="activePlatforms.includes(p.id) ? 'on bg-ink text-accent-ink border-ink font-[650]' : 'bg-surface text-muted border-faint'" :title="`Include ${p.name}`">
      <input type="checkbox" class="m-0 accent-accent cursor-pointer" :checked="activePlatforms.includes(p.id)" :aria-label="`Include ${p.name}`"
        @change="onToggle(p.id, $event)" />
      <span v-if="statusOf(p.id) !== 'disconnected'" class="flex-none inline-block w-2 h-2 rounded-full"
        :class="statusOf(p.id) === 'connected' ? (activePlatforms.includes(p.id) ? 'bg-accent-ink' : 'bg-emerald-500') : statusOf(p.id) === 'connecting' ? 'bg-amber-500' : 'bg-rose-500'" />
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

  <!-- Popup Mode (when popup=true): Button + Modal Dialog -->
  <div v-else class="inline-flex items-center">
    <!-- Trigger Button -->
    <button
      type="button"
      class="inline-flex items-center gap-1.5 px-3 py-1.5 border border-line rounded-full text-xs font-semibold text-muted hover:text-ink hover:bg-wash bg-surface transition shadow-2xs cursor-pointer select-none"
      :title="t('social.title')"
      @click="isOpen = true"
    >
      <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
        <circle cx="12" cy="12" r="3" />
        <path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z" />
      </svg>
      <span>{{ t('social.title') }}</span>
      <span class="px-1.5 py-0.2 rounded-full text-[10px] font-bold bg-wash text-muted border border-line/60">
        {{ activePlatforms.length }}
      </span>
    </button>

    <!-- Social Media Popup Modal -->
    <div
      v-if="isOpen"
      class="fixed inset-0 z-[1000] bg-black/50 backdrop-blur-xs flex items-center justify-center p-4 overflow-y-auto"
      @click.self="isOpen = false"
    >
      <div
        class="w-full max-w-lg bg-surface border border-line rounded-2xl shadow-2xl overflow-hidden animate-in fade-in zoom-in-95 duration-150"
        role="dialog"
        aria-modal="true"
      >
        <!-- Modal Header -->
        <div class="flex items-center justify-between px-5 py-3.5 border-b border-line bg-wash/50">
          <div>
            <h3 class="font-bold text-sm text-ink m-0">{{ t('social.title') }} Channels</h3>
            <p class="text-xs text-muted m-0 mt-0.5">Include platforms in your planning and manage connections.</p>
          </div>
          <button
            type="button"
            class="w-7 h-7 rounded-full flex items-center justify-center text-muted hover:text-ink hover:bg-wash transition cursor-pointer"
            aria-label="Close"
            @click="isOpen = false"
          >
            ✕
          </button>
        </div>

        <!-- Platform list grid -->
        <div class="p-4 grid grid-cols-1 sm:grid-cols-2 gap-2 max-h-[60vh] overflow-y-auto" role="group" :aria-label="t('social.title')">
          <div
            v-for="p in PLATFORMS"
            :key="p.id"
            class="flex items-center justify-between gap-2 p-2.5 rounded-xl border transition-colors"
            :class="activePlatforms.includes(p.id) ? 'bg-surface border-line shadow-2xs' : 'bg-wash/40 border-transparent text-muted opacity-80'"
          >
            <label class="flex items-center gap-2.5 cursor-pointer min-w-0 select-none">
              <input
                type="checkbox"
                class="accent-accent cursor-pointer"
                :checked="activePlatforms.includes(p.id)"
                :aria-label="`Include ${p.name}`"
                @change="onToggle(p.id, $event)"
              />
              <PlatformLogo :id="p.id" :size="18" :tile="true" />
              <div class="min-w-0">
                <div class="font-semibold text-xs text-ink truncate">{{ p.name }}</div>
                <div class="flex items-center gap-1.5 text-[10px] text-muted">
                  <span
                    class="inline-block w-1.5 h-1.5 rounded-full"
                    :class="statusOf(p.id) === 'connected' ? 'bg-emerald-500' : statusOf(p.id) === 'connecting' ? 'bg-amber-500' : 'bg-neutral-400'"
                  />
                  <span>{{ statusOf(p.id) }}</span>
                </div>
              </div>
            </label>

            <!-- Actions: link or connect button -->
            <div class="flex items-center gap-1 shrink-0">
              <template v-if="linkOf(p.id)">
                <a
                  class="socialname text-xs font-semibold text-muted hover:text-ink p-1"
                  :href="linkOf(p.id) as string"
                  target="_blank"
                  rel="noopener noreferrer"
                  :title="`${p.name} — ${t('social.openProfile')}`"
                >
                  ↗
                </a>
                <button
                  type="button"
                  class="text-xs font-semibold text-muted hover:text-ink p-1 cursor-pointer"
                  :aria-label="`${t('social.manage')} ${p.name}`"
                  :title="`${t('social.manage')} ${p.name}`"
                  @click="loginFor = p.id"
                >
                  ⚙
                </button>
              </template>
              <button
                v-else
                type="button"
                class="text-[11px] font-semibold text-muted hover:text-ink px-2 py-0.5 rounded bg-wash border border-line/60 hover:bg-surface cursor-pointer transition"
                :title="`${p.name} — ${t('social.connect')}`"
                @click="loginFor = p.id"
              >
                Connect
              </button>
            </div>
          </div>
        </div>

        <!-- Modal Footer -->
        <div class="px-5 py-3 border-t border-line bg-wash/30 flex items-center justify-between">
          <span class="text-xs text-muted">{{ activePlatforms.length }} of {{ PLATFORMS.length }} active</span>
          <button
            type="button"
            class="btn px-4 py-1.5 text-xs font-semibold"
            @click="isOpen = false"
          >
            Done
          </button>
        </div>
      </div>
    </div>

    <!-- Platform Login dialog -->
    <PlatformLogin :platform="loginFor" @close="loginFor = null" />
  </div>
</template>