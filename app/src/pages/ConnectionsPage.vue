<script setup lang="ts">
// Connections page — manage social platform accounts (Meta, YouTube, TikTok)
// connected to this workspace without redundant nested cards.
import { computed, ref } from 'vue'
import { usePlatforms, useLive, useWorkspaces, useDisconnectPlatform, usePermission } from '@/core/queries'
import { activeWorkspaceId } from '@/core/workspace'
import {
  PLATFORMS, profileUrl,
  activePlatforms, togglePlatform,
} from '@/core/platforms'
import type { PlatformId } from '@/core/platforms'
import { t } from '@/core/i18n'
import PlatformLogo from '@/components/ui/PlatformLogo.vue'
import PlatformLogin from '@/components/overlays/PlatformLogin.vue'
import InfoTip from '@/components/ui/InfoTip.vue'

const { data: workspaces } = useWorkspaces()
const activeWs = computed(() =>
  workspaces.value?.find((w) => w.id === activeWorkspaceId.value) ?? workspaces.value?.[0],
)
const { data: connections } = usePlatforms()
const { data: live } = useLive()
const disconnect = useDisconnectPlatform()
const { can } = usePermission()
const canManage = computed(() => can('platforms.manage'))

const loginFor = ref<PlatformId | null>(null)
const confirmDisconnectId = ref<PlatformId | null>(null)
const actionError = ref('')

function connectionOf(id: PlatformId) {
  return connections.value?.find((c) => c.id === id) ?? null
}

function statusOf(id: PlatformId): 'connected' | 'connecting' | 'error' | 'disconnected' {
  const s = connectionOf(id)?.status
  if (s === 'connected' || s === 'connecting' || s === 'error') return s
  return 'disconnected'
}

function pageNameOf(id: PlatformId): string {
  const conn = connectionOf(id)
  if (!conn) return ''
  // 1. Look in live accounts fetched from the platform Graph API
  const acc = live.value?.accounts?.find(
    (a) => (a.platform === id || (conn.externalId && a.id === conn.externalId)) && a.username,
  )
  if (acc?.username) return acc.username

  // 2. If handle is not a URL, it is already a Page name / username
  const handle = conn.handle.trim()
  if (handle && !handle.startsWith('http://') && !handle.startsWith('https://')) {
    return handle.startsWith('@') ? handle : `@${handle}`
  }

  // 3. Fallback to platform-friendly name
  if (id === 'meta') return 'Facebook Page'
  if (id === 'youtube') return 'YouTube Channel'
  if (id === 'tiktok') return 'TikTok Account'
  return conn.handle || 'Connected Account'
}

function linkedInstagram(): string | null {
  const ig = live.value?.accounts?.find((a) => a.platform === 'instagram' && a.username)
  return ig?.username ? `@${ig.username.replace(/^@/, '')}` : null
}

const connectedCount = computed(() =>
  PLATFORMS.filter((p) => statusOf(p.id) === 'connected').length,
)

const DESCRIPTIONS: Record<PlatformId, string> = {
  meta: 'Publish and schedule posts directly to Facebook Pages and connected Instagram accounts.',
  youtube: 'Track channel views, subscribers, and analyze video reach and audience retention.',
  tiktok: 'Schedule short-form videos and measure trends, engagement, and follower growth.',
}

async function handleDisconnect(id: PlatformId): Promise<void> {
  if (!canManage.value) return
  if (confirmDisconnectId.value !== id) {
    confirmDisconnectId.value = id
    return
  }
  confirmDisconnectId.value = null
  actionError.value = ''
  try {
    await disconnect.mutateAsync(id)
  } catch (e) {
    actionError.value = e instanceof Error ? e.message : String(e)
  }
}

function onToggleActive(id: PlatformId, event: Event): void {
  togglePlatform(id)
  const input = event.target as HTMLInputElement
  input.checked = activePlatforms.value.includes(id)
}
</script>

<template>
  <div class="connections-page">
    <!-- Header -->
    <header class="mb-6">
      <div class="flex items-center justify-between flex-wrap gap-4">
        <div>
          <div class="text-xs font-bold uppercase tracking-wider text-muted mb-0.5">Brand &amp; Social Workspace</div>
          <div class="flex items-center">
            <h1 class="text-3xl sm:text-4xl font-extrabold uppercase tracking-tight m-0 font-display text-ink">
              {{ activeWs?.name || 'Studio' }}
            </h1>
            <InfoTip text="Link social media accounts to publish content, schedule campaigns, and track live analytics." />
          </div>
        </div>

        <div class="flex items-center gap-2">
          <span class="inline-flex items-center gap-2 px-3.5 py-1.5 rounded-full text-xs font-semibold bg-wash border border-faint text-muted shadow-xs">
            <span class="w-2 h-2 rounded-full" :class="connectedCount > 0 ? 'bg-success' : 'bg-muted'" />
            <span>{{ connectedCount }} of {{ PLATFORMS.length }} connected</span>
          </span>
        </div>
      </div>
    </header>

    <p v-if="actionError" class="autherr text-xs text-danger mb-4" role="alert">{{ actionError }}</p>

    <!-- Platform Cards Grid -->
    <div class="grid grid-cols-1 md:grid-cols-3 gap-5">
      <div
        v-for="p in PLATFORMS"
        :key="p.id"
        class="border border-faint bg-surface rounded-xl p-5 flex flex-col justify-between transition-all hover:border-line hover:shadow-xs"
      >
        <!-- Top row: Logo, Name, and Status Badge -->
        <div>
          <div class="flex items-center justify-between gap-2 mb-3">
            <div class="flex items-center gap-2.5">
              <PlatformLogo :id="p.id" :size="24" />
              <strong class="text-base font-bold text-ink">{{ p.name }}</strong>
            </div>

            <!-- Status chip -->
            <span
              class="inline-flex items-center gap-1.5 px-2.5 py-0.5 rounded-full text-xs font-medium border"
              :class="{
                'bg-success-wash border-success text-success': statusOf(p.id) === 'connected',
                'bg-warning-wash border-warning text-warning': statusOf(p.id) === 'connecting',
                'bg-danger-wash border-danger text-danger': statusOf(p.id) === 'error',
                'bg-wash border-faint text-muted': statusOf(p.id) === 'disconnected',
              }"
            >
              <span
                class="w-1.5 h-1.5 rounded-full"
                :class="{
                  'bg-success': statusOf(p.id) === 'connected',
                  'bg-warning': statusOf(p.id) === 'connecting',
                  'bg-danger': statusOf(p.id) === 'error',
                  'bg-muted': statusOf(p.id) === 'disconnected',
                }"
              />
              <span>
                {{
                  statusOf(p.id) === 'connected'
                    ? 'Connected'
                    : statusOf(p.id) === 'connecting'
                      ? 'Connecting'
                      : statusOf(p.id) === 'error'
                        ? 'Error'
                        : 'Disconnected'
                }}
              </span>
            </span>
          </div>

          <!-- Middle content: Connected Page Name vs Description -->
          <div class="my-4 min-h-[72px]">
            <template v-if="statusOf(p.id) === 'connected'">
              <div class="flex flex-col gap-1.5">
                <div class="text-2xs uppercase font-extrabold tracking-wider text-muted">Brand Channel</div>
                <div class="flex items-center gap-1.5 flex-wrap">
                  <a
                    v-if="profileUrl(p.id, connectionOf(p.id)?.handle)"
                    :href="profileUrl(p.id, connectionOf(p.id)?.handle) as string"
                    target="_blank"
                    rel="noopener noreferrer"
                    class="text-2xl sm:text-[26px] font-extrabold text-ink font-display tracking-tight leading-tight hover:underline inline-flex items-center gap-2"
                    :title="`${pageNameOf(p.id)} — open profile`"
                  >
                    <span>{{ pageNameOf(p.id) }}</span>
                    <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round" class="text-muted flex-none" aria-hidden="true">
                      <path d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6" />
                      <polyline points="15 3 21 3 21 9" />
                      <line x1="10" y1="14" x2="21" y2="3" />
                    </svg>
                  </a>
                  <span v-else class="text-2xl sm:text-[26px] font-extrabold text-ink font-display tracking-tight leading-tight">{{ pageNameOf(p.id) }}</span>
                </div>

                <div class="flex items-center gap-2 text-xs text-muted flex-wrap mt-0.5">
                  <span v-if="connectionOf(p.id)?.externalId" class="font-mono">ID: {{ connectionOf(p.id)?.externalId }}</span>
                  <span v-if="p.id === 'meta' && linkedInstagram()" class="font-semibold text-accent">
                    • Linked IG: {{ linkedInstagram() }}
                  </span>
                </div>
                <div class="text-xs text-muted mt-0.5">{{ p.auth }}</div>
              </div>
            </template>
            <template v-else>
              <div class="text-sm text-muted leading-relaxed">
                {{ DESCRIPTIONS[p.id] }}
              </div>
            </template>
          </div>

          <!-- Active in Workspace Toggle (only when connected) -->
          <div v-if="statusOf(p.id) === 'connected'" class="pt-3 border-t border-faint flex items-center justify-between">
            <label class="flex items-center gap-2 text-xs text-muted cursor-pointer select-none hover:text-ink">
              <input
                type="checkbox"
                class="accent-accent"
                :checked="activePlatforms.includes(p.id)"
                @change="onToggleActive(p.id, $event)"
              />
              <span>Active in this workspace</span>
            </label>
          </div>
        </div>

        <!-- Footer Actions -->
        <div class="pt-4 mt-3 border-t border-faint flex items-center gap-2">
          <template v-if="statusOf(p.id) === 'connected'">
            <button
              type="button"
              class="btn text-xs py-1.5 px-3.5 flex-1 inline-flex items-center justify-center gap-1.5"
              :disabled="!canManage"
              :title="canManage ? 'Manage connection settings' : t('auth.noPerm')"
              :aria-label="`Manage ${p.name}`"
              @click="loginFor = p.id"
            >
              <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
                <circle cx="12" cy="12" r="3" />
                <path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z" />
              </svg>
              <span>{{ t('social.manageShort') }}</span>
            </button>
            <button
              type="button"
              class="btn text-xs py-1.5 px-3 text-danger border-faint hover:border-danger hover:text-danger"
              :class="{ 'btn-danger': confirmDisconnectId === p.id }"
              :disabled="!canManage || disconnect.isPending.value"
              :title="canManage ? 'Disconnect this account' : t('auth.noPerm')"
              @click="handleDisconnect(p.id)"
            >
              {{ confirmDisconnectId === p.id ? 'Confirm?' : 'Disconnect' }}
            </button>
          </template>
          <template v-else>
            <button
              type="button"
              class="btn btn-primary text-xs py-2 w-full"
              :disabled="!canManage"
              :title="canManage ? '' : t('auth.noPerm')"
              @click="loginFor = p.id"
            >
              Connect {{ p.name }}
            </button>
          </template>
        </div>
      </div>
    </div>

    <!-- Security & Workspace Integration Info Box -->
    <div class="mt-6 p-4 rounded-xl border border-faint bg-wash flex items-center justify-between flex-wrap gap-3">
      <div class="flex items-center gap-3">
        <span class="w-8 h-8 rounded-full bg-surface border border-faint flex items-center justify-center flex-none text-muted font-bold text-xs italic">
          i
        </span>
        <div class="text-xs text-muted leading-relaxed">
          Connected channels are shared across authorized members of this workspace.
          Tokens and permissions are encrypted and refreshed automatically.
        </div>
      </div>
    </div>

    <!-- Platform Login Modal -->
    <PlatformLogin :platform="loginFor" @close="loginFor = null" />
  </div>
</template>
