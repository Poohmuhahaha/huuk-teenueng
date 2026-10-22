<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import {
  useSetup, useSaveSetup, useAddOption, useAddUser, useRemoveUser,
  useAddRole, useRemoveRole, usePermission, useAccounts, useDeleteAccount,
  useCreateAccount,
} from '@/core/queries'
import { currentUser } from '@/core/auth'
import { t } from '@/core/i18n'
import type { OptionList } from '@/api/contract'
import { PERMISSIONS } from '@/mock/db'
import type { Permission, Role } from '@/mock/db'
import Chip from '@/components/ui/Chip.vue'
import TableFrame from '@/components/tables/TableFrame.vue'
import InfoTip from '@/components/ui/InfoTip.vue'

const { data: setup, isPending } = useSetup()
const saveSetup = useSaveSetup()
const addOption = useAddOption()
const addUser = useAddUser()
const removeUser = useRemoveUser()
const addRole = useAddRole()
const removeRole = useRemoveRole()
const accountsQ = useAccounts()
const removeAccount = useDeleteAccount()
const { can, authRequired } = usePermission()

const canSetup = computed(() => can('setup.write'))
const canManage = computed(() => can('users.manage'))

// ---- scalar fields: local drafts so a rejected save can visibly revert ----
const yearDraft = ref<number | ''>('')
const ownerDraft = ref('')
const workspaceDraft = ref('')
const colorsDraft = ref(false)
const authDraft = ref(false)

watch(
  setup,
  (s) => {
    if (!s) return
    yearDraft.value = s.year
    ownerDraft.value = s.owner
    workspaceDraft.value = s.workspaceName ?? 'workspace'
    colorsDraft.value = s.showEditableColors
    authDraft.value = s.authRequired
  },
  { immediate: true },
)

// Double-click confirm arming state (replaces native window.confirm): the first
// click arms the button (its label flips to "Confirm?"), the second executes.
// Arming resets after 3s.
const confirmId = ref<string | null>(null)
let confirmTimer: number | undefined

async function confirmRemove(id: string, fn: () => Promise<void>): Promise<void> {
  if (confirmId.value !== id) {
    confirmId.value = id
    if (confirmTimer) clearTimeout(confirmTimer)
    confirmTimer = window.setTimeout(() => { confirmId.value = null }, 3000)
    return
  }
  if (confirmTimer) clearTimeout(confirmTimer)
  confirmId.value = null
  await fn()
}

async function saveYear(): Promise<void> {
  if (!canSetup.value) return
  const value = Number(yearDraft.value)
  if (!Number.isInteger(value) || value < 1970 || value > 9999) {
    yearDraft.value = setup.value?.year ?? ''
    return
  }
  if (value === setup.value?.year) return
  try {
    await saveSetup.mutateAsync({ year: value })
  } catch {
    yearDraft.value = setup.value?.year ?? ''
  }
}

async function saveOwner(): Promise<void> {
  if (!canSetup.value) return
  const value = ownerDraft.value.trim()
  if (!value || value === setup.value?.owner) {
    ownerDraft.value = setup.value?.owner ?? ''
    return
  }
  try {
    await saveSetup.mutateAsync({ owner: value })
  } catch {
    ownerDraft.value = setup.value?.owner ?? ''
  }
}

async function saveWorkspace(): Promise<void> {
  if (!canSetup.value) return
  const value = workspaceDraft.value.trim()
  if (!value || value.length > 80 || value === setup.value?.workspaceName) {
    workspaceDraft.value = setup.value?.workspaceName ?? 'workspace'
    return
  }
  try {
    await saveSetup.mutateAsync({ workspaceName: value })
  } catch {
    workspaceDraft.value = setup.value?.workspaceName ?? 'workspace'
  }
}

async function saveColors(): Promise<void> {
  if (!canSetup.value) return
  try {
    await saveSetup.mutateAsync({ showEditableColors: colorsDraft.value })
  } catch {
    colorsDraft.value = setup.value?.showEditableColors ?? false
  }
}

async function saveAuthRequired(): Promise<void> {
  if (!canManage.value) return
  try {
    await saveSetup.mutateAsync({ authRequired: authDraft.value })
  } catch {
    authDraft.value = setup.value?.authRequired ?? false
  }
}

// ---- users ----
const newName = ref('')
const newRole = ref('Editor')

async function addUsr(): Promise<void> {
  const name = newName.value.trim()
  if (!name || !canManage.value || addUser.isPending.value) return
  try {
    await addUser.mutateAsync({ name, role: newRole.value })
    newName.value = ''
  } catch {
    // error shown below
  }
}

async function removeUsr(name: string): Promise<void> {
  if (!canManage.value) return
  try {
    await removeUser.mutateAsync(name)
  } catch {
    // error shown below
  }
}

// ---- options ----
const draft = ref<Record<OptionList, string>>({
  pillars: '', formats: '', goals: '', statuses: '', platforms: '',
})

const LISTS: { key: OptionList; labelKey: string }[] = [
  { key: 'pillars', labelKey: 'setup.list.pillars' },
  { key: 'formats', labelKey: 'setup.list.formats' },
  { key: 'goals', labelKey: 'setup.list.goals' },
  { key: 'statuses', labelKey: 'setup.list.statuses' },
  { key: 'platforms', labelKey: 'setup.list.platforms' },
]

async function add(list: OptionList): Promise<void> {
  const value = draft.value[list].trim()
  if (!value || !canSetup.value || addOption.isPending.value) return
  try {
    await addOption.mutateAsync({ list, item: value })
    draft.value[list] = ''
  } catch {
    // error shown below
  }
}

// ---- roles ----
const roleDraft = ref<Role[]>([])
const roleSignature = ref('')
const newRoleName = ref('')

// Reseed only when the server-side role definition actually changed, so
// unrelated setup refetches never discard in-progress permission edits.
watch(
  setup,
  (s) => {
    const signature = JSON.stringify(s?.roles ?? [])
    if (signature === roleSignature.value) return
    roleSignature.value = signature
    roleDraft.value = (s?.roles ?? []).map((r) => ({ name: r.name, permissions: [...r.permissions] }))
  },
  { immediate: true },
)

function togglePerm(index: number, perm: Permission, on: boolean): void {
  const role = roleDraft.value[index]
  if (!role) return
  const permissions = new Set(role.permissions)
  if (on) permissions.add(perm)
  else permissions.delete(perm)
  role.permissions = [...permissions]
}

function roleInUse(name: string): boolean {
  return (setup.value?.users ?? []).some((u) => u.role === name)
}

async function saveRoles(): Promise<void> {
  if (!canSetup.value || saveSetup.isPending.value) return
  try {
    await saveSetup.mutateAsync({
      roles: roleDraft.value.map((r) => ({ name: r.name, permissions: [...r.permissions] })),
    })
  } catch {
    // error shown below
  }
}

async function addRl(): Promise<void> {
  const name = newRoleName.value.trim()
  if (!name || !canManage.value || addRole.isPending.value) return
  try {
    await addRole.mutateAsync({ name })
    newRoleName.value = ''
  } catch {
    // error shown below
  }
}

async function removeRl(name: string): Promise<void> {
  if (!canManage.value) return
  try {
    await removeRole.mutateAsync(name)
  } catch {
    // error shown below
  }
}

async function removeAcc(account: { name: string; email: string }): Promise<void> {
  if (!canManage.value || account.name === currentUser.value?.name) return
  try {
    await removeAccount.mutateAsync(account.email)
  } catch {
    // error shown below
  }
}

// ---- client onboarding (admin creates the account) ----
const createAccount = useCreateAccount()
const accName = ref('')
const accEmail = ref('')
const accRole = ref('')
const accPassword = ref('')
const tempPassword = ref('')
const accError = ref('')
const copiedTemp = ref(false)

watch(
  setup,
  (s) => {
    const roles = s?.roles ?? []
    if (!accRole.value || !roles.some((r) => r.name === accRole.value)) {
      accRole.value = roles.some((r) => r.name === 'Client') ? 'Client' : (roles[0]?.name ?? '')
    }
  },
  { immediate: true },
)

async function addAccount(): Promise<void> {
  if (!canManage.value || createAccount.isPending.value) return
  accError.value = ''
  tempPassword.value = ''
  const name = accName.value.trim()
  const email = accEmail.value.trim()
  if (!name || !email.includes('@')) {
    accError.value = t('settings.accountInvalid')
    return
  }
  try {
    const result = await createAccount.mutateAsync({
      name,
      email,
      role: accRole.value,
      password: accPassword.value || undefined,
    })
    tempPassword.value = result.temporaryPassword ?? ''
    accName.value = ''
    accEmail.value = ''
    accPassword.value = ''
  } catch (e) {
    accError.value = e instanceof Error ? e.message : String(e)
  }
}

async function copyTempPassword(): Promise<void> {
  if (!tempPassword.value) return
  try {
    await navigator.clipboard.writeText(tempPassword.value)
    copiedTemp.value = true
    window.setTimeout(() => { copiedTemp.value = false }, 1500)
  } catch {
    accError.value = t('studio.copyFailed')
  }
}

const setupError = computed(() => {
  const e = saveSetup.error.value ?? addOption.error.value ?? addRole.error.value ?? removeRole.error.value
  return e ? (e instanceof Error ? e.message : String(e)) : ''
})
</script>

<template>
  <p v-if="authRequired && !canSetup" class="muted" style="margin-top: 0;">{{ t('setup.readOnlyHint') }}</p>
  <div v-if="isPending" class="muted">{{ t('common.loading') }}</div>
  <div v-else-if="setup" class="settings-body-flow">
    <!-- General Workspace Settings -->
    <section class="settings-section">
      <div class="settings-section-head">
        <h2 class="settings-title flex items-center">
          <span>{{ t('setup.workspace') }} Details</span>
          <InfoTip text="Core planning year and workspace display configuration." />
        </h2>
      </div>
      <div class="settings-grid3">
        <div class="settings-field-group">
          <label class="lbl" for="setup-year">{{ t('setup.year') }}</label>
          <input id="setup-year" class="field" type="number" v-model.number="yearDraft" :disabled="!canSetup || saveSetup.isPending.value"
            min="1970" max="9999" :title="canSetup ? '' : t('auth.noPerm')" @change="saveYear" />
        </div>
        <div class="settings-field-group">
          <label class="lbl" for="setup-owner">{{ t('setup.owner') }}</label>
          <input id="setup-owner" class="field" v-model="ownerDraft" :disabled="!canSetup || saveSetup.isPending.value"
            :title="canSetup ? '' : t('auth.noPerm')" @change="saveOwner" />
        </div>
        <div class="settings-field-group">
          <label class="lbl" for="setup-workspace">{{ t('setup.workspace') }}</label>
          <input id="setup-workspace" class="field" v-model="workspaceDraft" :disabled="!canSetup || saveSetup.isPending.value"
            maxlength="80" :title="canSetup ? '' : t('auth.noPerm')" @change="saveWorkspace" />
        </div>
      </div>
      <div class="settings-toggle-row">
        <label class="settings-checkbox-label">
          <input type="checkbox" v-model="colorsDraft" :disabled="!canSetup || saveSetup.isPending.value"
            :title="canSetup ? '' : t('auth.noPerm')" @change="saveColors" />
          <span>{{ t('setup.showEditableColors') }}</span>
        </label>
      </div>
    </section>

    <!-- Option Lists (Single Source of Truth) -->
    <section class="settings-section">
      <div class="settings-section-head">
        <h2 class="settings-title flex items-center">
          <span>{{ t('setup.optionLists') }}</span>
          <InfoTip text="Predefined taxonomy and categories used across planners and campaigns." />
        </h2>
      </div>
      <div class="settings-options-grid">
        <div v-for="l in LISTS" :key="l.key" class="settings-option-card">
          <div class="settings-option-title">{{ t(l.labelKey) }}</div>
          <div class="settings-option-chips">
            <Chip v-for="(v, i) in (setup[l.key] as string[])" :key="`${v}-${i}`" :label="v" />
          </div>
          <div class="row mt-auto pt-2">
            <input class="field" style="flex: 1;" v-model="draft[l.key]" :placeholder="`new ${t(l.labelKey).toLowerCase()}`"
              :aria-label="`New ${t(l.labelKey)}`" :disabled="!canSetup" :title="canSetup ? '' : t('auth.noPerm')"
              @keyup.enter="add(l.key)" />
            <button class="btn w-auto" :disabled="!canSetup || addOption.isPending.value || !(draft[l.key] ?? '').trim()"
              :title="canSetup ? '' : t('auth.noPerm')" @click="add(l.key)">{{ t('common.add') }}</button>
          </div>
        </div>
      </div>
    </section>

    <!-- Users Directory -->
    <section class="settings-section">
      <div class="settings-section-head">
        <h2 class="settings-title flex items-center">
          <span>{{ t('setup.users') }}</span>
          <InfoTip text="Workspace member roles and permissions hierarchy." />
        </h2>
      </div>
      <div class="settings-list-container">
        <div v-for="u in setup.users" :key="u.name" class="settings-list-row">
          <div class="settings-user-info">
            <span class="settings-item-name">{{ u.name }}</span>
            <span class="settings-role-badge">· {{ u.role }}</span>
          </div>
          <button class="settings-del-btn" :class="{ confirming: confirmId === `user:${u.name}` }"
            :disabled="setup.users.length <= 1 || !canManage || removeUser.isPending.value"
            :title="canManage ? 'Remove user' : t('auth.noPerm')" :aria-label="`Remove user ${u.name}`"
            @click="confirmRemove(`user:${u.name}`, () => removeUsr(u.name))">
            {{ confirmId === `user:${u.name}` ? 'Confirm?' : '×' }}
          </button>
        </div>
        <div class="settings-add-row">
          <input class="field" style="flex: 1;" v-model="newName" placeholder="new user" aria-label="New user name"
            :disabled="!canManage" :title="canManage ? '' : t('auth.noPerm')" @keyup.enter="addUsr" />
          <select class="field w-auto" v-model="newRole" aria-label="New user role" :disabled="!canManage"
            :title="canManage ? '' : t('auth.noPerm')">
            <option v-for="r in setup.roles" :key="r.name" :value="r.name">{{ r.name }}</option>
          </select>
          <button class="btn btn-primary w-auto" :disabled="!canManage || addUser.isPending.value || !newName.trim()"
            :title="canManage ? '' : t('auth.noPerm')" @click="addUsr">{{ t('common.add') }}</button>
        </div>
        <p v-if="addUser.error.value || removeUser.error.value" class="autherr" role="alert" style="margin: 8px 14px;">
          {{ (addUser.error.value ?? removeUser.error.value)?.message }}
        </p>
      </div>
    </section>

    <!-- Client Sign-In Accounts -->
    <template v-if="canManage">
      <section class="settings-section">
        <div class="settings-section-head">
          <h2 class="settings-title flex items-center">
            <span>{{ t('setup.accounts') }}</span>
            <InfoTip :text="t('setup.accountsHint')" />
          </h2>
        </div>
        <div class="settings-list-container">
          <div v-for="a in accountsQ.data.value ?? []" :key="a.email" class="settings-list-row">
            <div class="settings-account-info">
              <span class="settings-item-name">{{ a.name }}</span>
              <span class="settings-item-sub">· {{ a.email }}</span>
              <span class="settings-session-chip">{{ a.sessions }} {{ t('setup.sessions') }}</span>
            </div>
            <button class="settings-del-btn" :class="{ confirming: confirmId === `account:${a.email}` }"
              :disabled="!canManage || a.name === currentUser?.name || removeAccount.isPending.value"
              :title="a.name === currentUser?.name ? 'You cannot delete your own account' : t('setup.removeAccount')"
              :aria-label="`Remove account ${a.email}`" @click="confirmRemove(`account:${a.email}`, () => removeAcc(a))">
              {{ confirmId === `account:${a.email}` ? 'Confirm?' : '×' }}
            </button>
          </div>
          <p v-if="accountsQ.error.value || removeAccount.error.value" class="autherr" role="alert" style="margin: 8px 14px;">
            {{ (accountsQ.error.value ?? removeAccount.error.value)?.message }}
          </p>
        </div>

        <div class="settings-subblock mt-5">
          <h3 class="settings-subtitle flex items-center">
            <span>{{ t('setup.createAccount') }}</span>
            <InfoTip :text="t('setup.createAccountHint')" />
          </h3>
          <div class="settings-grid2">
            <div>
              <label class="lbl" for="acc-name">{{ t('setup.accountName') }}</label>
              <input id="acc-name" class="field" v-model="accName" />
            </div>
            <div>
              <label class="lbl" for="acc-email">{{ t('setup.accountEmail') }}</label>
              <input id="acc-email" class="field" type="email" v-model="accEmail" autocomplete="off" />
            </div>
            <div>
              <label class="lbl" for="acc-role">{{ t('setup.accountRole') }}</label>
              <select id="acc-role" class="field" v-model="accRole">
                <option v-for="r in setup.roles" :key="r.name" :value="r.name">{{ r.name }}</option>
              </select>
            </div>
            <div>
              <label class="lbl" for="acc-pass">{{ t('setup.accountPassword') }}</label>
              <input id="acc-pass" class="field" type="text" v-model="accPassword" autocomplete="new-password"
                :placeholder="t('setup.accountPasswordHint')" />
            </div>
          </div>
          <div class="row mt-3.5">
            <button class="btn btn-primary w-auto" :disabled="createAccount.isPending.value" @click="addAccount">
              {{ t('setup.createAccountBtn') }}
            </button>
          </div>
          <p v-if="accError" class="autherr" role="alert" style="margin-top: 8px;">{{ accError }}</p>
          <div v-if="tempPassword" role="status" class="settings-temp-box">
            <strong>{{ t('setup.tempPassword') }}</strong>
            <code class="settings-temp-code">{{ tempPassword }}</code>
            <button class="btn w-auto" @click="copyTempPassword">{{ copiedTemp ? t('studio.copied') : t('setup.copyPassword') }}</button>
          </div>
        </div>
      </section>
    </template>

    <!-- Permissions Matrix -->
    <section class="settings-section">
      <div class="settings-section-head">
        <h2 class="settings-title flex items-center">
          <span>{{ t('auth.perms') }}</span>
          <InfoTip text="Fine-grained access rights per role across features." />
        </h2>
      </div>
      <div class="settings-toggle-row mb-3.5">
        <label class="settings-checkbox-label">
          <input type="checkbox" v-model="authDraft" :disabled="!canManage || saveSetup.isPending.value"
            :title="canManage ? '' : t('auth.noPerm')" @change="saveAuthRequired" />
          <span>{{ t('auth.require') }}</span>
        </label>
      </div>
      <div class="settings-perms-wrap">
        <TableFrame>
          <thead>
            <tr>
              <th scope="col">{{ t('auth.perms') }}</th>
              <th scope="col" v-for="r in roleDraft" :key="r.name">
                <div class="flex items-center justify-between gap-1">
                  <span>{{ r.name }}</span>
                  <button class="settings-del-btn" style="width: 22px !important; height: 22px !important; min-width: 22px !important; font-size: 13px !important; margin-left: 6px;"
                    :class="{ confirming: confirmId === `role:${r.name}` }"
                    :disabled="!canManage || roleDraft.length <= 1 || roleInUse(r.name) || removeRole.isPending.value"
                    :title="canManage ? (roleInUse(r.name) ? 'Role is in use' : '') : t('auth.noPerm')"
                    :aria-label="`Remove role ${r.name}`" @click="confirmRemove(`role:${r.name}`, () => removeRl(r.name))">
                    {{ confirmId === `role:${r.name}` ? 'Confirm?' : '×' }}
                  </button>
                </div>
              </th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="perm in PERMISSIONS" :key="perm" style="cursor: default;">
              <td class="muted">{{ perm }}</td>
              <td v-for="(r, i) in roleDraft" :key="r.name">
                <input type="checkbox" :checked="r.permissions.includes(perm)" :disabled="!canManage"
                  :aria-label="`${r.name}: ${perm}`" :title="canManage ? '' : t('auth.noPerm')"
                  @change="togglePerm(i, perm, ($event.target as HTMLInputElement).checked)" />
              </td>
            </tr>
          </tbody>
        </TableFrame>
      </div>
      <div class="row mt-3.5">
        <input class="field" style="flex: 1;" v-model="newRoleName" placeholder="new role" aria-label="New role name"
          :disabled="!canManage" :title="canManage ? '' : t('auth.noPerm')" @keyup.enter="addRl" />
        <button class="btn w-auto" :disabled="!canManage || addRole.isPending.value || !newRoleName.trim()"
          :title="canManage ? '' : t('auth.noPerm')" @click="addRl">{{ t('auth.addRole') }}</button>
        <button class="btn btn-primary w-auto" :disabled="!canSetup || saveSetup.isPending.value"
          :title="canSetup ? '' : t('auth.noPerm')" @click="saveRoles">{{ t('auth.savePerms') }}</button>
      </div>
      <p v-if="setupError" class="autherr" role="alert" style="margin-top: 8px;">{{ setupError }}</p>
    </section>
  </div>
</template>
