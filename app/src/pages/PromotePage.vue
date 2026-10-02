<script setup lang="ts">
// Promote hub — content campaigns and Meta Ads management.
import { ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import HubTabs from '@/components/ui/HubTabs.vue'
import CampaignsPage from '@/pages/CampaignsPage.vue'
import AdsPage from '@/pages/AdsPage.vue'

const route = useRoute()
const router = useRouter()

const TABS = [
  { id: 'campaigns', label: 'Campaigns' },
  { id: 'ads', label: 'Meta Ads' },
]
const tab = ref(
  typeof route.query.tab === 'string' && TABS.some((t) => t.id === route.query.tab)
    ? (route.query.tab as string)
    : 'campaigns',
)
const id = ref(typeof route.query.id === 'string' ? (route.query.id as string) : '')

watch([tab, id], () => {
  void router.replace({
    query: { ...route.query, tab: tab.value, ...(id.value ? { id: id.value } : {}) },
  })
})
</script>

<template>
  <div class="hubpage">
    <HubTabs :tabs="TABS" v-model="tab" />
    <CampaignsPage
      v-if="tab === 'campaigns'"
      :id="id || undefined"
      embedded
      @update:id="id = $event"
    />
    <AdsPage v-else />
  </div>
</template>
