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
  <div class="members-page max-w-4xl mx-auto px-4 sm:px-6 py-6 w-full text-left">
    <!-- Header -->
    <header class="mb-6 flex items-center justify-between flex-wrap gap-4">
      <div>
        <div class="text-xs font-bold uppercase tracking-wider text-muted mb-0.5">Brand &amp; Social Workspace</div>
        <div class="flex items-center gap-2">
          <h1 class="text-3xl sm:text-4xl font-extrabold uppercase tracking-tight m-0 font-display text-ink">
            {{ activeWs?.name || 'Workspace' }}
          </h1>
          <InfoTip :text="t('members.hint')" />
        </div>
        <p class="text-xs text-muted mt-1 mb-0">Manage team collaboration and permissions for this workspace.</p>
      </div>

      <div v-if="isOwner" class="flex items-center gap-2">
        <span class="inline-flex items-center gap-2 px-3.5 py-1.5 rounded-full text-xs font-semibold bg-wash border border-faint text-muted shadow-xs">
          <span class="w-2 h-2 rounded-full bg-success" />
          <span>{{ rows.length }} {{ rows.length === 1 ? 'member' : 'members' }}</span>
        </span>
      </div>
    </header>

    <p v-if="isError" class="card text-muted p-4 rounded-xl border border-faint bg-surface">{{ loadError?.message }}</p>
    <p v-else-if="workspacesPending" class="text-muted p-4">{{ t('common.loading') }}</p>

    <!-- Non-owner Notice -->
    <div v-else-if="!isOwner" class="card p-6 rounded-2xl border border-faint bg-surface text-center">
      <div class="w-12 h-12 rounded-full bg-wash border border-faint flex items-center justify-center mx-auto mb-3 text-muted">
        <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <rect width="18" height="11" x="3" y="11" rx="2" ry="2" />
          <path d="M7 11V7a5 5 0 0 1 10 0v4" />
        </svg>
      </div>
      <p class="text-muted text-sm font-medium mb-0">{{ t('members.onlyOwner') }}</p>
    </div>

    <template v-else>
      <!-- Add Member Card -->
      <div class="card p-5 border border-faint bg-surface rounded-2xl mb-6 shadow-xs">
        <div class="flex items-center gap-2 mb-3">
          <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="text-ink">
            <path d="M16 21v-2a4 4 0 0 0-4-4H6a4 4 0 0 0-4 4v2" />
            <circle cx="9" cy="7" r="4" />
            <line x1="19" y1="8" x2="19" y2="14" />
            <line x1="22" y1="11" x2="16" y2="11" />
          </svg>
          <h2 class="text-sm font-bold text-ink m-0">Invite Team Member</h2>
        </div>

        <form class="members-add flex flex-col sm:flex-row items-stretch sm:items-center gap-2.5" @submit.prevent="submitAdd">
          <input
            v-model="email"
            class="field flex-1 text-sm rounded-xl border border-line bg-wash/50 focus:bg-surface focus:border-ink transition-all px-3.5 py-2"
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
            class="btn btn-primary w-auto flex-none px-5 py-2 rounded-xl text-sm font-semibold inline-flex items-center justify-center gap-2 cursor-pointer transition-all shadow-xs"
            :disabled="addMember.isPending.value || !email.trim()"
          >
            {{ t('members.add') }}
          </button>
        </form>

        <p v-if="actionError" class="autherr text-xs text-danger mt-3 mb-0" role="alert">{{ actionError }}</p>
        <p v-else-if="notice" class="oknote text-xs text-success mt-3 mb-0" role="status">{{ notice }}</p>
      </div>

      <p v-if="isPending" class="text-muted p-4">{{ t('common.loading') }}</p>
      
      <!-- Members List Card -->
      <div v-else class="card p-5 border border-faint bg-surface rounded-2xl shadow-xs">
        <div class="flex items-center justify-between mb-4">
          <div class="flex items-center gap-2">
            <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="text-ink">
              <path d="M17 21v-2a4 4 0 0 0-4-4H5a4 4 0 0 0-4 4v2" />
              <circle cx="9" cy="7" r="4" />
              <path d="M23 21v-2a4 4 0 0 0-3-3.87" />
              <path d="M16 3.13a4 4 0 0 1 0 7.75" />
            </svg>
            <h2 class="text-sm font-bold text-ink m-0">Team Members</h2>
          </div>
          <span class="text-xs text-muted font-medium">{{ rows.length }} active</span>
        </div>

        <ul class="list-none m-0 p-0 flex flex-col gap-2">
          <li
            v-for="m in rows"
            :key="m.email"
            class="panel member-row flex items-center justify-between gap-3 p-3 sm:p-3.5 rounded-xl border border-faint bg-wash/40 hover:bg-wash/70 transition-colors"
          >
            <div class="flex items-center gap-3 min-w-0 mr-auto">
              <!-- Initials avatar -->
              <div class="w-9 h-9 rounded-full bg-surface border border-line flex items-center justify-center flex-none font-bold text-xs text-ink shadow-2xs">
                {{ (m.name || m.email).slice(0, 2).toUpperCase() }}
              </div>
              <div class="flex flex-col min-w-0 text-left">
                <div class="flex items-center gap-2">
                  <strong class="text-sm font-semibold text-ink truncate">{{ m.name || m.email }}</strong>
                  <span v-if="isYou(m)" class="chip dark text-2xs py-0.5 px-2 rounded-full font-medium">{{ t('members.you') }}</span>
                </div>
                <span class="text-muted text-xs truncate">{{ m.email }}</span>
              </div>
            </div>

            <div class="flex items-center gap-2 flex-none">
              <span v-if="isOwnerRow(m)" class="badge badge-success text-xs font-semibold py-1 px-2.5 rounded-full">{{ t('members.owner') }}</span>
              <span v-else-if="m.role" class="chip text-xs py-1 px-2.5 rounded-full font-medium">{{ m.role }}</span>

              <button
                v-if="!isOwnerRow(m)"
                type="button"
                class="btn member-remove w-auto flex-none text-xs py-1.5 px-3 rounded-full border border-faint text-muted hover:border-danger hover:text-danger hover:bg-danger-wash transition-colors cursor-pointer"
                :disabled="removeMember.isPending.value"
                @click="onRemove(m)"
              >
                {{ t('members.remove') }}
              </button>
            </div>
          </li>
        </ul>
        <p v-if="!(members?.members.length)" class="text-muted text-xs mt-3 mb-0">{{ t('members.empty') }}</p>
      </div>
    </template>
  </div>
</template>
