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
const id = ref(typeof route.query.id === 'string' ? (route.query.id as string) : '')

watch(id, () => {
  void router.replace({
    query: { ...route.query, tab: tab.value, ...(id.value ? { id: id.value } : {}) },
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
