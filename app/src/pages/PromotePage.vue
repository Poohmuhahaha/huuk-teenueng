<script setup lang="ts">
// Promote hub — content campaigns and Meta Ads management.
import { ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useHubTab } from '@/core/subnav'
import CampaignsPage from '@/pages/CampaignsPage.vue'
import AdsPage from '@/pages/AdsPage.vue'

const route = useRoute()
const router = useRouter()

const { active: tab } = useHubTab('/promote')

// Hidden hub cards stay mounted in the deck, so only the active hub may write
// its selection into the URL — otherwise Content and Promote overwrite each
// other's `?id`/`?tab` on load.
function onHub(): boolean {
  return route.path === '/promote' || route.path.startsWith('/promote/')
}

const id = ref(onHub() && typeof route.query.id === 'string' ? (route.query.id as string) : '')

watch(
  () => route.query.id,
  (value) => {
    if (!onHub()) return
    const next = typeof value === 'string' ? value : ''
    if (next && next !== id.value) id.value = next
  },
)

watch(id, () => {
  if (!onHub()) return
  const nextId = id.value || ''
  const currentId = typeof route.query.id === 'string' ? route.query.id : ''
  if (nextId === currentId && route.query.tab === tab.value) return
  void router.replace({
    query: { ...route.query, tab: tab.value, ...(nextId ? { id: nextId } : {}) },
  })
})
</script>

<template>
  <div class="hubpage">
    <CampaignsPage
      v-if="tab === 'campaigns'"
      :id="id || undefined"
      embedded
      @update:id="id = $event"
    />
    <AdsPage v-else />
  </div>
</template>
