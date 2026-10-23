<script setup lang="ts">
// Owner-only workspace members: the owner plus the accounts the owner added.
// Writes are gated on the active workspace summary's `isOwner`; the server
// enforces the same rule and answers 400/403/404/409 for the rest.
import { computed, ref } from 'vue'
import {
  useWorkspaces, useWorkspaceMembers, useAddWorkspaceMember, useRemoveWorkspaceMember,
} from '@/core/queries'
import { activeWorkspaceId } from '@/core/workspace'
import { currentUser } from '@/core/session'
import { t } from '@/core/i18n'
import InfoTip from '@/components/ui/InfoTip.vue'
import DockBar from '@/components/ui/DockBar.vue'
import type { WorkspaceMember } from '@/mock/db'

const { data: workspaces, isPending: workspacesPending } = useWorkspaces()

const activeWs = computed(() => {
  const list = workspaces.value ?? []
  return list.find((w) => w.id === activeWorkspaceId.value) ?? list[0] ?? null
})
const isOwner = computed(() => activeWs.value?.isOwner === true)
const wsId = computed(() => activeWs.value?.id ?? '')

// The member list is owner-only on the server, so others do not even fetch it.
const { data: members, isPending, isError, error: loadError } = useWorkspaceMembers(
  computed(() => (isOwner.value ? wsId.value : '')),
)

const addMember = useAddWorkspaceMember()
const removeMember = useRemoveWorkspaceMember()

const email = ref('')
const notice = ref('')
const actionError = ref('')

const rows = computed<WorkspaceMember[]>(() => {
  const view = members.value
  if (!view) return []
  return [view.owner, ...view.members]
})

function isOwnerRow(member: WorkspaceMember): boolean {
  const owner = members.value?.owner.email ?? ''
  return owner !== '' && member.email.toLowerCase() === owner.toLowerCase()
}

function isYou(member: WorkspaceMember): boolean {
  return Boolean(currentUser.value?.name) && member.name === currentUser.value?.name
}

async function submitAdd(): Promise<void> {
  const value = email.value.trim()
  if (!value || addMember.isPending.value || !isOwner.value) return
  notice.value = ''
  actionError.value = ''
  try {
    await addMember.mutateAsync({ id: wsId.value, email: value })
    email.value = ''
    notice.value = t('members.added')
  } catch (e) {
    actionError.value = e instanceof Error ? e.message : String(e)
  }
}

async function onRemove(member: WorkspaceMember): Promise<void> {
  if (!isOwner.value || removeMember.isPending.value) return
  if (typeof window !== 'undefined' && !window.confirm(`${t('members.remove')}: ${member.email}?`)) return
  notice.value = ''
  actionError.value = ''
  try {
    await removeMember.mutateAsync({ id: wsId.value, email: member.email })
    notice.value = t('members.removed')
  } catch (e) {
    actionError.value = e instanceof Error ? e.message : String(e)
  }
}
</script>

<template>
  <div class="members-page w-full px-6 sm:px-8 py-6 text-left font-sans flex flex-col flex-1">
    <!-- Header -->
    <div class="flex items-center justify-between flex-wrap gap-4 mb-5">
      <div>
        <div class="flex items-center gap-2">
          <div role="heading" aria-level="2" class="text-base sm:text-lg font-bold text-ink tracking-tight">
            {{ t('members.title') }}
          </div>
          <InfoTip :text="t('members.hint')" />
        </div>
        <p class="text-xs text-muted mt-1 mb-0 font-sans">
          {{ t('members.subtitle') }}
        </p>
      </div>

      <div v-if="isOwner" class="flex items-center gap-2">
        <span class="inline-flex items-center gap-2 px-3 py-1 rounded-full text-xs font-semibold bg-wash border border-faint text-muted shadow-2xs">
          <span class="w-2 h-2 rounded-full bg-success" />
          <span>{{ rows.length }} {{ rows.length === 1 ? 'member' : 'members' }}</span>
        </span>
      </div>
    </div>

    <p v-if="isError" class="card text-muted p-4 rounded-xl border border-faint bg-surface">{{ loadError?.message }}</p>
    <p v-else-if="workspacesPending" class="text-muted p-4">{{ t('common.loading') }}</p>

    <!-- Non-owner Notice -->
    <div v-else-if="!isOwner" class="p-6 rounded-xl border border-line bg-surface text-center">
      <p class="text-muted text-sm font-medium mb-0">{{ t('members.onlyOwner') }}</p>
    </div>

    <template v-else>
      <!-- Add Member Row -->
      <form class="members-add flex items-center gap-2.5 mb-5" @submit.prevent="submitAdd">
        <input
          v-model="email"
          class="field flex-1 text-sm rounded-xl border border-line bg-surface focus:border-ink transition-colors px-3.5 py-2.5"
          type="text"
          inputmode="email"
          autocomplete="email"
          maxlength="254"
          :placeholder="t('members.emailPlaceholder')"
          :aria-label="t('members.emailPlaceholder')"
          :disabled="addMember.isPending.value"
        />
        <button
          type="submit"
          class="btn btn-primary w-auto flex-none px-5 py-2.5 rounded-xl text-sm font-semibold inline-flex items-center justify-center cursor-pointer transition-all shadow-xs"
          :disabled="addMember.isPending.value || !email.trim()"
        >
          {{ t('members.add') }}
        </button>
      </form>

      <p v-if="actionError" class="autherr text-xs text-danger mb-3" role="alert">{{ actionError }}</p>
      <p v-else-if="notice" class="oknote text-xs text-success mb-3" role="status">{{ notice }}</p>

      <p v-if="isPending" class="text-muted">{{ t('common.loading') }}</p>

      <!-- Members Table Frame -->
      <div v-else class="tblwrap rounded-xl border border-line bg-surface overflow-hidden shadow-2xs w-full">
        <div class="px-5 py-3 bg-surface-2/60 border-b border-faint flex items-center justify-between text-2xs font-bold uppercase tracking-wider text-muted font-sans">
          <span>Member</span>
          <div class="flex items-center gap-12 sm:gap-16">
            <span class="w-20 text-center">Role</span>
            <span class="w-16 text-right">Action</span>
          </div>
        </div>

        <ul class="list-none m-0 p-0 divide-y divide-faint">
          <li
            v-for="m in rows"
            :key="m.email"
            class="panel member-row flex items-center justify-between gap-3 px-5 py-3.5 hover:bg-wash/50 transition-colors"
          >
            <div class="flex items-center gap-3 min-w-0 mr-auto">
              <div class="w-8 h-8 rounded-full bg-wash border border-line flex items-center justify-center font-bold text-xs text-ink shrink-0 font-sans">
                {{ (m.name || m.email).slice(0, 2).toUpperCase() }}
              </div>
              <div class="flex flex-col min-w-0 text-left font-sans">
                <div class="flex items-center gap-2">
                  <strong class="text-sm font-medium text-ink truncate">{{ m.name || m.email }}</strong>
                  <span v-if="isYou(m)" class="chip dark text-2xs py-0.5 px-2 rounded-full font-medium">{{ t('members.you') }}</span>
                </div>
                <span class="text-muted text-xs truncate">{{ m.email }}</span>
              </div>
            </div>

            <div class="flex items-center gap-12 sm:gap-16 shrink-0">
              <div class="w-20 flex justify-center">
                <span v-if="isOwnerRow(m)" class="badge badge-success text-xs font-semibold py-0.5 px-2.5 rounded-full">{{ t('members.owner') }}</span>
                <span v-else-if="m.role" class="chip text-xs py-0.5 px-2.5 rounded-full font-medium">{{ m.role }}</span>
              </div>

              <div class="w-16 flex justify-end">
                <button
                  v-if="!isOwnerRow(m)"
                  type="button"
                  class="btn member-remove w-auto text-xs py-1 px-2.5 rounded-md border border-faint text-muted hover:border-danger hover:text-danger hover:bg-danger-wash transition-colors cursor-pointer"
                  :disabled="removeMember.isPending.value"
                  @click="onRemove(m)"
                >
                  {{ t('members.remove') }}
                </button>
              </div>
            </div>
          </li>
        </ul>
      </div>

      <p v-if="!(members?.members.length)" class="text-muted text-xs mt-3">{{ t('members.empty') }}</p>

      <DockBar label="Members controls">
        <div class="flex items-center justify-between w-full gap-3">
          <div class="flex items-center gap-2">
            <span class="inline-flex items-center gap-2 px-3 py-1 rounded-full text-xs font-semibold bg-wash border border-faint text-muted shadow-2xs">
              <span class="w-2 h-2 rounded-full bg-success" />
              <span>{{ rows.length }} {{ rows.length === 1 ? 'member' : 'members' }}</span>
            </span>
          </div>
          <div class="text-xs text-muted">
            {{ isOwner ? t('members.hint') : t('members.onlyOwner') }}
          </div>
        </div>
      </DockBar>
    </template>
  </div>
</template>
