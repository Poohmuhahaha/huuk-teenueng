<script setup lang="ts">
import { computed } from 'vue'
import { useTags, useAddTag, usePermission } from '@/core/queries'
import { t } from '@/core/i18n'
import HashtagGroups from '@/components/ui/HashtagGroups.vue'

const { data: groups, isPending, isError, error: loadError, refetch } = useTags()
const addTag = useAddTag()
const { can } = usePermission()
const canWrite = computed(() => can('hashtags.write'))

async function onAdd(groupId: string, tag: string): Promise<void> {
  if (!canWrite.value) throw new Error(t('auth.noPerm'))
  await addTag.mutateAsync({ groupId, tag })
}
</script>

<template>
  <h1>Hashtag Library</h1>
  <div v-if="isPending" class="muted">Loading groups…</div>
  <div v-else-if="isError" class="card">
    <p class="muted">Could not load hashtag groups: {{ loadError?.message }}</p>
    <button class="btn" @click="() => refetch()">{{ t('common.tryAgain') }}</button>
  </div>
  <fieldset v-else class="permfield" :disabled="!canWrite" :title="canWrite ? '' : t('auth.noPerm')">
    <HashtagGroups :groups="groups ?? []" :add="onAdd" />
  </fieldset>
</template>

<style scoped>
.permfield {
  border: 0;
  padding: 0;
  margin: 0;
  min-width: 0;
}
</style>
