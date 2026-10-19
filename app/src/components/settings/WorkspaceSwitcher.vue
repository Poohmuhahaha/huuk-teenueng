<script setup lang="ts">
// Workspace switcher — moved out of the avatar popup onto the Settings
// workspace tab, where it renders as a full-width card section.
// Every workspace owns its own brand + connections, so switching invalidates
// all queries; the HTTP client sends the active id on every request.
import { computed, nextTick, ref, watch } from 'vue'
import { useQueryClient } from '@tanstack/vue-query'
import {
  useSetup, usePermission,
  useWorkspaces, useCreateWorkspace, useRenameWorkspace, useDeleteWorkspace,
} from '@/core/queries'
import { activeWorkspaceId, setActiveWorkspace } from '@/core/workspace'
import { t } from '@/core/i18n'

const qc = useQueryClient()
const { data: setup } = useSetup()
const { can } = usePermission()
const { data: workspaces } = useWorkspaces()
const createWorkspace = useCreateWorkspace()
const renameWorkspace = useRenameWorkspace()
const deleteWorkspace = useDeleteWorkspace()

const wsOpen = ref(false)
const wsError = ref('')
const wsNewName = ref('')
const wsRenameDraft = ref('')
const wsRenaming = ref(false)
const wsRenameInput = ref<HTMLInputElement | null>(null)
const canManageWs = computed(() => can('platforms.manage'))

const activeWs = computed(() => {
  const list = workspaces.value ?? []
  return list.find((w) => w.id === activeWorkspaceId.value) ?? list[0] ?? null
})
const activeWsName = computed(() => activeWs.value?.name ?? setup.value?.workspaceName ?? 'workspace')

// A stored id can point at a deleted workspace: fall back to the first one.
watch(
  () => workspaces.value,
  (list) => {
    if (!list?.length) return
    if (!list.some((w) => w.id === activeWorkspaceId.value)) {
      setActiveWorkspace(list[0].id)
    }
  },
  { immediate: true },
)

function toggleWsMenu(): void {
  wsOpen.value = !wsOpen.value
  wsError.value = ''
  wsRenaming.value = false
}

function switchWs(id: string): void {
  wsOpen.value = false
  if (id === activeWs.value?.id) return
  setActiveWorkspace(id)
  // Everything the app shows is workspace-scoped — refetch it all.
  qc.invalidateQueries()
}

async function submitWsCreate(): Promise<void> {
  const name = wsNewName.value.trim()
  if (!name) return
  wsError.value = ''
  try {
    const ws = await createWorkspace.mutateAsync(name)
    wsNewName.value = ''
    setActiveWorkspace(ws.id)
    qc.invalidateQueries()
    wsOpen.value = false
  } catch (e) {
    wsError.value = e instanceof Error ? e.message : String(e)
  }
}

function startRenameWs(): void {
  wsRenaming.value = true
  wsRenameDraft.value = activeWs.value?.name ?? ''
  void nextTick(() => {
    wsRenameInput.value?.focus()
    wsRenameInput.value?.select()
  })
}

async function submitRenameWs(): Promise<void> {
  const name = wsRenameDraft.value.trim()
  const current = activeWs.value
  if (!current || !name || name === current.name) {
    wsRenaming.value = false
    return
  }
  wsError.value = ''
  try {
    await renameWorkspace.mutateAsync({ id: current.id, name })
    wsRenaming.value = false
  } catch (e) {
    wsError.value = e instanceof Error ? e.message : String(e)
  }
}

async function removeWs(): Promise<void> {
  const current = activeWs.value
  if (!current) return
  wsError.value = ''
  try {
    const res = await deleteWorkspace.mutateAsync(current.id)
    setActiveWorkspace(res.fallback || null)
    wsOpen.value = false
    qc.invalidateQueries()
  } catch (e) {
    wsError.value = e instanceof Error ? e.message : String(e)
  }
}
</script>

<template>
  <section class="settings-section ws-section">
    <div class="settings-section-head">
      <div class="ws-head">{{ t('ws.title') }}</div>
      <p class="settings-desc">Switch, rename, or create workspaces for your brand and client management.</p>
    </div>
    <div class="ws-wrap">
      <button type="button" class="wire ws-trigger" :title="t('ws.switch')"
        aria-haspopup="menu" :aria-expanded="wsOpen" @click="toggleWsMenu">
        <span class="flex items-center gap-2">
          <span class="ws-name font-semibold">{{ activeWsName }}</span>
        </span>
        <span class="flex items-center gap-2 text-muted text-xs">
          <span v-if="activeWs" class="chip py-0 px-2 m-0">{{ activeWs.connected }}/{{ activeWs.total }} connected</span>
          <span class="ws-caret" aria-hidden="true">▾</span>
        </span>
      </button>
      <div v-if="wsOpen" class="ws-menu" role="menu" :aria-label="t('ws.title')">
        <button v-for="w in workspaces ?? []" :key="w.id" type="button" class="ws-item" role="menuitem"
          :class="{ active: w.id === activeWs?.id }" @click="switchWs(w.id)">
          <span class="ws-name">{{ w.name }}</span>
          <span class="ws-count">{{ w.connected }}/{{ w.total }}</span>
          <span v-if="w.id === activeWs?.id" class="ws-check" aria-hidden="true">✓</span>
        </button>
        <p v-if="!(workspaces ?? []).length" class="muted" style="padding: 6px 10px;">{{ t('common.loading') }}</p>

        <div v-if="canManageWs" class="ws-actions">
          <template v-if="wsRenaming">
            <input ref="wsRenameInput" v-model="wsRenameDraft" class="field" maxlength="80"
              :aria-label="t('ws.rename')" @keydown.enter.prevent="submitRenameWs"
              @keydown.esc.prevent="wsRenaming = false" @blur="submitRenameWs" />
          </template>
          <template v-else>
            <button type="button" class="menubtn" @click="startRenameWs">{{ t('ws.rename') }}</button>
            <button type="button" class="menubtn danger" :disabled="(workspaces ?? []).length <= 1"
              @click="removeWs">{{ t('ws.delete') }}</button>
          </template>
        </div>

        <form v-if="canManageWs" class="ws-form" @submit.prevent="submitWsCreate">
          <input v-model="wsNewName" class="field" maxlength="80" :placeholder="t('ws.newPlaceholder')"
            :aria-label="t('ws.new')" />
          <button type="submit" class="btn btn-primary" :disabled="!wsNewName.trim()">{{ t('ws.new') }}</button>
        </form>
        <p v-if="wsError" class="autherr" role="alert" style="margin: 6px 8px 0;">{{ wsError }}</p>
      </div>
    </div>
  </section>
</template>
