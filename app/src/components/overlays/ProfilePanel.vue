<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { usePermission, useUpdateProfile } from '@/core/queries'
import { currentUser } from '@/core/auth'
import { t } from '@/core/i18n'

const { isLoggedIn } = usePermission()
const updateProfile = useUpdateProfile()

const name = ref(currentUser.value?.name ?? '')
const changed = computed(() => name.value.trim() !== (currentUser.value?.name ?? ''))

watch(currentUser, (u) => { name.value = u?.name ?? '' }, { immediate: true })

// A stale "Saved" label must not sit next to unsaved edits.
watch(name, () => {
  if (updateProfile.isSuccess.value) updateProfile.reset()
})

function save(): void {
  if (!isLoggedIn.value || !changed.value) return
  if (!name.value.trim()) return
  updateProfile.mutate(name.value.trim())
}
</script>

<template>
  <div>
    <label class="lbl" for="profile-name" style="margin-top: 0;">{{ t('profile.title') }}</label>
    <template v-if="isLoggedIn">
      <input id="profile-name" class="field" v-model="name" maxlength="80" autocomplete="nickname" @keyup.enter="save" />
      <div class="row mt">
        <button class="btn btn-primary" :disabled="!changed || !name.trim() || updateProfile.isPending.value" @click="save">
          {{ t('profile.save') }}
        </button>
        <span v-if="updateProfile.isSuccess.value" role="status" class="muted">{{ t('profile.saved') }}</span>
      </div>
      <p v-if="updateProfile.error.value" class="autherr" role="alert" style="margin: 6px 0 0;">
        {{ updateProfile.error.value?.message }}
      </p>
    </template>
    <p v-else class="muted" style="margin: 0;">{{ t('profile.loginHint') }}</p>
  </div>
</template>
