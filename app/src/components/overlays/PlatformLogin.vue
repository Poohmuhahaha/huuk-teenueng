<script setup lang="ts">
// Connect/manage a social platform.
// - OAuth-configured providers: hand the browser to the provider (redirect).
// - Otherwise: link the account's public profile URL/handle, which makes the
//   social bar entry a real link to the owner's account.
import { computed, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { useConnectPlatform, useDisconnectPlatform, usePlatforms, useSyncPlatform, usePermission } from '@/core/queries'
import { PLATFORM_META, profileUrl, validProfileHandle } from '@/core/platforms'
import type { PlatformId } from '@/core/platforms'
import api from '@/api'
import { beginOAuth } from '@/core/oauth'
import type { OauthMode } from '@/api/contract'
import { t } from '@/core/i18n'

const props = defineProps<{ platform: PlatformId | null }>()
const emit = defineEmits<{ close: [] }>()

const { data: connections } = usePlatforms()
const { can } = usePermission()
const connect = useConnectPlatform()
const disconnect = useDisconnectPlatform()
const sync = useSyncPlatform()

const meta = computed(() => (props.platform ? PLATFORM_META[props.platform] : null))
const conn = computed(() => connections.value?.find((c) => c.id === props.platform) ?? null)
const connected = computed(() => conn.value?.status === 'connected')
const mayManage = computed(() => can('platforms.manage'))

const manualHandle = ref('')
const busy = ref(false)
const statusReady = ref(false)
const mode = ref<OauthMode>('mock')
const authorizeHost = ref('')
const oauthError = ref('')

const HANDLE_HINTS: Record<PlatformId, string> = {
  meta: 'https://www.facebook.com/profile.php?id=… or https://facebook.com/yourpage',
  youtube: '@yourchannel or https://youtube.com/@yourchannel',
  tiktok: '@yourbrand or https://tiktok.com/@yourbrand',
}
const handleHint = computed(() => (props.platform ? HANDLE_HINTS[props.platform] : ''))
const profileHref = computed(() =>
  props.platform ? profileUrl(props.platform, conn.value?.handle) : null,
)

watch(
  () => props.platform,
  (p) => {
    manualHandle.value = ''
    busy.value = false
    statusReady.value = false
    mode.value = 'mock'
    authorizeHost.value = ''
    oauthError.value = ''
    if (!p) return
    api
      .oauthStatus(p)
      .then((s) => {
        if (props.platform !== p) return
        mode.value = s.mode
        authorizeHost.value = s.authorizeHost ?? ''
      })
      .catch(() => {
        if (props.platform === p) mode.value = 'mock'
      })
      .finally(() => {
        if (props.platform === p) statusReady.value = true
      })
  },
  { immediate: true },
)

// Prefill the field with the current handle once the connection is known.
watch(conn, (c) => {
  if (c?.handle && !manualHandle.value) manualHandle.value = c.handle
})

function onEscape(e: KeyboardEvent): void {
  if (e.key === 'Escape' && props.platform && !busy.value) emit('close')
}

onMounted(() => window.addEventListener('keydown', onEscape))
onBeforeUnmount(() => window.removeEventListener('keydown', onEscape))

async function connectManual(): Promise<void> {
  const platform = props.platform
  if (!platform || busy.value || !mayManage.value) return
  const handle = manualHandle.value.trim()
  if (!handle) {
    oauthError.value = t('slogin.profileRequired')
    return
  }
  if (!validProfileHandle(platform, handle)) {
    oauthError.value = t('slogin.profileInvalid')
    return
  }
  busy.value = true
  oauthError.value = ''
  try {
    await connect.mutateAsync({ id: platform, handle })
  } catch (e) {
    oauthError.value = e instanceof Error ? e.message : String(e)
  } finally {
    busy.value = false
  }
}

// Real OAuth: create state server-side, then hand the browser to the provider's own login page.
// Also the "Reconnect" path for an existing connection (expired or missing token).
async function continueToProvider(): Promise<void> {
  const platform = props.platform
  if (!platform || busy.value || !mayManage.value) return
  busy.value = true
  oauthError.value = ''
  try {
    const error = await beginOAuth(platform)
    if (error) oauthError.value = error
  } finally {
    busy.value = false
  }
}

async function onDisconnect(): Promise<void> {
  if (!props.platform || disconnect.isPending.value) return
  try {
    await disconnect.mutateAsync(props.platform)
    manualHandle.value = ''
    oauthError.value = ''
  } catch (e) {
    oauthError.value = e instanceof Error ? e.message : String(e)
  }
}

async function onSync(): Promise<void> {
  if (!props.platform || sync.isPending.value) return
  try {
    await sync.mutateAsync(props.platform)
    oauthError.value = ''
  } catch (e) {
    oauthError.value = e instanceof Error ? e.message : String(e)
  }
}
</script>

<template>
  <div v-if="platform && meta" class="plogin" role="dialog" aria-modal="true"
    :aria-label="`${meta.name} — ${t('social.manage')}`" @click.self="emit('close')">
    <div class="plogin-body">
      <div class="plogin-card">
        <button class="plogin-close" :aria-label="t('common.close')" :title="t('common.close')"
          :disabled="busy" @click="emit('close')">×</button>
        <div class="plogin-brand">{{ meta.name }}</div>

        <p v-if="!mayManage" class="plogin-note">{{ t('auth.noPerm') }}</p>

        <!-- connected: account state + maintenance -->
        <template v-if="connected">
          <h3 class="plogin-title">{{ t('slogin.connected') }}</h3>
          <p v-if="conn?.hasToken === false" class="autherr" role="alert">
            {{ t('slogin.reconnectNeeded') }}
          </p>
          <div class="grid2">
            <div class="panel plogin-full"><div class="muted">{{ t('slogin.account') }}</div><strong>{{ conn?.handle }}</strong></div>
            <div class="panel"><div class="muted">{{ meta.externalLabel }}</div><strong>{{ conn?.externalId }}</strong></div>
            <div class="panel"><div class="muted">Token</div><strong>{{ conn?.tokenType }}</strong></div>
            <div class="panel"><div class="muted">Expires</div><strong>{{ conn?.expiresAt ?? '—' }}</strong></div>
            <div class="panel"><div class="muted">Last sync</div><strong>{{ conn?.lastSync ?? '—' }}</strong></div>
            <div class="panel"><div class="muted">Media tracked</div><strong>{{ conn?.mediaCount }}</strong></div>
          </div>
          <div class="panel mt">
            <strong>Scopes granted</strong>
            <div class="mt"><span v-for="s in conn?.scopes ?? []" :key="s" class="chip">{{ s }}</span></div>
          </div>
          <p class="muted mt">{{ meta.constraint }}</p>
          <p v-if="mode === 'mock'" class="muted">{{ t('slogin.unverifiedNote') }}</p>
          <p v-if="oauthError" class="muted">{{ oauthError }}</p>
          <p v-if="sync.error.value || disconnect.error.value" class="muted">
            {{ (sync.error.value ?? disconnect.error.value)?.message }}
          </p>
          <div class="plogin-actions mt">
            <a v-if="profileHref" class="btn" :href="profileHref" target="_blank" rel="noopener noreferrer">
              {{ t('slogin.openProfile') }}
            </a>
            <button v-if="mode === 'redirect'" class="btn"
              :class="{ 'btn-primary': conn?.hasToken === false }"
              :disabled="!mayManage || busy" :title="mayManage ? '' : t('auth.noPerm')"
              @click="continueToProvider">
              {{ busy ? t('slogin.connecting') : t('slogin.reconnect') }}
            </button>
            <RouterLink v-if="conn?.mediaCount" class="btn" to="/live">{{ t('live.view') }}</RouterLink>
            <button class="btn" :disabled="!mayManage || sync.isPending.value"
              :title="mayManage ? '' : t('auth.noPerm')" @click="onSync">
              {{ sync.isPending.value ? t('slogin.syncing') : t('slogin.syncNow') }}
            </button>
            <button class="btn" :disabled="!mayManage || disconnect.isPending.value"
              :title="mayManage ? '' : t('auth.noPerm')" @click="onDisconnect">{{ t('slogin.disconnect') }}</button>
          </div>
        </template>

        <!-- provider credentials missing: link the public profile instead -->
        <template v-else-if="statusReady && mode === 'mock'">
          <h3 class="plogin-title">{{ t('slogin.connectTitle') }} · {{ meta.name }}</h3>
          <p class="muted">{{ t('slogin.connectHint') }}</p>
          <label class="lbl" for="plogin-handle">{{ t('slogin.profile') }}</label>
          <input id="plogin-handle" class="field" v-model="manualHandle" :placeholder="handleHint"
            :disabled="!mayManage" @keyup.enter="connectManual" />
          <div class="row mt">
            <button class="btn btn-primary" style="width: 100%;"
              :disabled="!mayManage || busy || !manualHandle.trim()" @click="connectManual">
              {{ busy ? t('slogin.connecting') : t('slogin.connect') }}
            </button>
          </div>
          <p v-if="oauthError" class="autherr" role="alert">{{ oauthError }}</p>
        </template>

        <!-- real OAuth: hand off to the provider -->
        <template v-else-if="statusReady">
          <h3 class="plogin-title">{{ t('slogin.title') }} · {{ meta.name }}</h3>
          <p class="muted">{{ t('slogin.redirectNote') }}</p>
          <p class="muted" v-if="authorizeHost">→ <code>{{ authorizeHost }}</code></p>
          <div class="row mt">
            <button class="btn btn-primary" style="width: 100%;" :disabled="!mayManage || busy"
              @click="continueToProvider">
              {{ t('slogin.continueTo') }} {{ meta.name }}
            </button>
          </div>
          <p v-if="oauthError" class="autherr" role="alert">{{ oauthError }}</p>
        </template>

        <p v-else class="muted">{{ t('common.loading') }}</p>
      </div>
    </div>
  </div>
</template>
