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
  <div class="mt-2">

    <p v-if="isError" class="card text-muted">{{ loadError?.message }}</p>
    <p v-else-if="workspacesPending" class="text-muted">{{ t('common.loading') }}</p>

    <p v-else-if="!isOwner" class="text-muted mt-4.5">{{ t('members.onlyOwner') }}</p>

    <template v-else>
      <form class="members-add flex items-center flex-wrap mt-3.5 gap-2" @submit.prevent="submitAdd">
        <input v-model="email" class="field flex-1 max-w-form" type="text" inputmode="email" autocomplete="email"
          maxlength="254" :placeholder="t('members.emailPlaceholder')"
          :aria-label="t('members.emailPlaceholder')" :disabled="addMember.isPending.value" />
        <button type="submit" class="btn btn-primary"
          :disabled="addMember.isPending.value || !email.trim()">
          {{ t('members.add') }}
        </button>
        <InfoTip :text="t('members.hint')" />
      </form>

      <p v-if="actionError" class="autherr text-xs text-danger" role="alert">{{ actionError }}</p>
      <p v-else-if="notice" class="oknote text-xs text-success" role="status">{{ notice }}</p>

      <p v-if="isPending" class="text-muted">{{ t('common.loading') }}</p>
      <template v-else>
        <ul class="list-none mt-4 p-0 flex flex-col gap-2">
          <li v-for="m in rows" :key="m.email" class="panel member-row flex items-center gap-2.5 p-3 rounded-lg border border-faint bg-surface">
            <div class="flex flex-col min-w-0 mr-auto">
              <strong class="text-sm font-semibold">{{ m.name || m.email }}</strong>
              <span class="text-muted text-xs-mid">{{ m.email }}</span>
            </div>
            <span v-if="m.role" class="chip">{{ m.role }}</span>
            <span v-if="isOwnerRow(m)" class="badge badge-success">{{ t('members.owner') }}</span>
            <span v-if="isYou(m)" class="chip dark">{{ t('members.you') }}</span>
            <button v-if="!isOwnerRow(m)" class="btn member-remove ml-0"
              :disabled="removeMember.isPending.value" @click="onRemove(m)">
              {{ t('members.remove') }}
            </button>
          </li>
        </ul>
        <p v-if="!(members?.members.length)" class="text-muted mt-2">{{ t('members.empty') }}</p>
      </template>
    </template>
  </div>
</template>
