<script setup lang="ts">
// Content hub — the CMS studio plus the feed preview.
import { ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import HubTabs from '@/components/ui/HubTabs.vue'
import StudioPage from '@/pages/StudioPage.vue'
import FeedPage from '@/pages/FeedPage.vue'

const route = useRoute()
const router = useRouter()

const TABS = [
  { id: 'studio', label: 'Studio' },
  { id: 'feed', label: 'Feed preview' },
]
const tab = ref(
  typeof route.query.tab === 'string' && TABS.some((t) => t.id === route.query.tab)
    ? (route.query.tab as string)
    : 'studio',
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
    <StudioPage
      v-if="tab === 'studio'"
      :id="id || undefined"
      embedded
      @update:id="id = $event"
    />
    <FeedPage v-else />
  </div>
</template>
