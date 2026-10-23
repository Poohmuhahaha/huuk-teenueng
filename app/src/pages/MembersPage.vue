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
    <div class="mb-5">
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

    <p v-if="isError" class="card text-muted p-4 rounded-xl border border-faint bg-surface">{{ loadError?.message }}</p>
    <p v-else-if="workspacesPending" class="text-muted p-4">{{ t('common.loading') }}</p>

    <!-- Non-owner Notice -->
    <div v-else-if="!isOwner" class="p-6 rounded-xl border border-line bg-surface text-center flex flex-col items-center justify-center">
      <div class="w-10 h-10 rounded-full bg-wash border border-faint flex items-center justify-center mb-2.5 text-muted">
        <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
          <rect width="18" height="11" x="3" y="11" rx="2" ry="2" />
          <path d="M7 11V7a5 5 0 0 1 10 0v4" />
        </svg>
      </div>
      <p class="text-muted text-sm font-medium mb-0">{{ t('members.onlyOwner') }}</p>
    </div>

    <template v-else>
      <!-- Add Member Row -->
      <form class="members-add flex items-center gap-2.5 mb-5" @submit.prevent="submitAdd">
        <div class="relative flex-1">
          <svg
            class="absolute left-3.5 top-1/2 -translate-y-1/2 text-muted pointer-events-none"
            width="15"
            height="15"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
            aria-hidden="true"
          >
            <rect width="20" height="16" x="2" y="4" rx="2" />
            <path d="m22 7-8.97 5.7a1.94 1.94 0 0 1-2.06 0L2 7" />
          </svg>
          <input
            v-model="email"
            class="field w-full pl-10 text-sm rounded-xl border border-line bg-surface focus:border-ink transition-colors pr-3.5 py-2.5"
            type="text"
            inputmode="email"
            autocomplete="email"
            maxlength="254"
            :placeholder="t('members.emailPlaceholder')"
            :aria-label="t('members.emailPlaceholder')"
            :disabled="addMember.isPending.value"
          />
        </div>
        <button
          type="submit"
          class="btn btn-primary w-auto flex-none px-5 py-2.5 rounded-xl text-sm font-semibold inline-flex items-center gap-1.5 justify-center cursor-pointer transition-all shadow-xs"
          :disabled="addMember.isPending.value || !email.trim()"
        >
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
            <line x1="12" y1="5" x2="12" y2="19" />
            <line x1="5" y1="12" x2="19" y2="12" />
          </svg>
          <span>{{ t('members.add') }}</span>
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
            <span class="w-20 text-right">Action</span>
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
                <span v-if="isOwnerRow(m)" class="badge badge-success text-xs font-semibold py-0.5 px-2 rounded-full inline-flex items-center gap-1">
                  <svg width="11" height="11" viewBox="0 0 24 24" fill="currentColor" aria-hidden="true">
                    <path d="M12 2l3.09 6.26L22 9.27l-5 4.87 1.18 6.88L12 17.77l-6.18 3.25L7 14.14 2 9.27l6.91-1.01L12 2z" />
                  </svg>
                  <span>{{ t('members.owner') }}</span>
                </span>
                <span v-else-if="m.role" class="chip text-xs py-0.5 px-2.5 rounded-full font-medium">{{ m.role }}</span>
              </div>

              <div class="w-20 flex justify-end">
                <button
                  v-if="!isOwnerRow(m)"
                  type="button"
                  class="btn member-remove w-auto text-xs py-1 px-2.5 rounded-md border border-faint text-muted hover:border-danger hover:text-danger hover:bg-danger-wash inline-flex items-center gap-1 transition-colors cursor-pointer"
                  :disabled="removeMember.isPending.value"
                  :title="t('members.remove')"
                  :aria-label="t('members.remove')"
                  @click="onRemove(m)"
                >
                  <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
                    <polyline points="3 6 5 6 21 6" />
                    <path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2" />
                  </svg>
                  <span>{{ t('members.remove') }}</span>
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
            <span class="inline-flex items-center gap-1.5 px-3 py-1 rounded-full text-xs font-semibold bg-wash border border-faint text-muted shadow-2xs">
              <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="text-ink shrink-0" aria-hidden="true">
                <path d="M19 21v-2a4 4 0 0 0-4-4H9a4 4 0 0 0-4 4v2" />
                <circle cx="12" cy="7" r="4" />
              </svg>
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
