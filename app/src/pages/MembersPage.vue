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
  <div class="memberspage">
    <h1 style="margin: 0;">{{ t('members.title') }}<InfoTip :text="t('members.subtitle')" /></h1>

    <p v-if="isError" class="card muted">{{ loadError?.message }}</p>
    <p v-else-if="workspacesPending" class="muted">{{ t('common.loading') }}</p>

    <p v-else-if="!isOwner" class="muted members-only">{{ t('members.onlyOwner') }}</p>

    <template v-else>
      <form class="row members-add" @submit.prevent="submitAdd">
        <input v-model="email" class="field" type="text" inputmode="email" autocomplete="email"
          maxlength="254" :placeholder="t('members.emailPlaceholder')"
          :aria-label="t('members.emailPlaceholder')" :disabled="addMember.isPending.value" />
        <button type="submit" class="btn btn-primary"
          :disabled="addMember.isPending.value || !email.trim()">
          {{ t('members.add') }}
        </button>
        <InfoTip :text="t('members.hint')" />
      </form>

      <p v-if="actionError" class="autherr" role="alert">{{ actionError }}</p>
      <p v-else-if="notice" class="oknote" role="status">{{ notice }}</p>

      <p v-if="isPending" class="muted">{{ t('common.loading') }}</p>
      <template v-else>
        <ul class="members-list">
          <li v-for="m in rows" :key="m.email" class="panel member-row">
            <div class="member-id">
              <strong>{{ m.name || m.email }}</strong>
              <span class="muted">{{ m.email }}</span>
            </div>
            <span v-if="m.role" class="chip">{{ m.role }}</span>
            <span v-if="isOwnerRow(m)" class="badge badge-success">{{ t('members.owner') }}</span>
            <span v-if="isYou(m)" class="chip dark">{{ t('members.you') }}</span>
            <button v-if="!isOwnerRow(m)" class="btn member-remove"
              :disabled="removeMember.isPending.value" @click="onRemove(m)">
              {{ t('members.remove') }}
            </button>
          </li>
        </ul>
        <p v-if="!(members?.members.length)" class="muted">{{ t('members.empty') }}</p>
      </template>
    </template>
  </div>
</template>

<style scoped>
.memberspage {
  margin-top: 8px;
}
.members-add {
  margin: 14px 0 0;
  gap: 8px;
}
.members-add .field {
  flex: 1;
  max-width: 340px;
}
.members-hint {
  margin: 6px 0 0;
  font-size: 12.5px;
}
.members-only {
  margin-top: 18px;
}
.members-list {
  list-style: none;
  margin: 16px 0 0;
  padding: 0;
  display: flex;
  flex-direction: column;
  gap: 8px;
}
.member-row {
  display: flex;
  align-items: center;
  gap: 10px;
}
.member-id {
  display: flex;
  flex-direction: column;
  min-width: 0;
}
.member-id strong {
  font-size: 14px;
}
.member-id .muted {
  font-size: 12.5px;
}
.member-remove {
  margin-left: auto;
}
</style>
