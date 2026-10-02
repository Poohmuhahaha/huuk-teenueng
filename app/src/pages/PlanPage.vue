<script setup lang="ts">
// Plan hub — monthly planner + calendar + ideas + hashtags in one screen.
import { ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import HubTabs from '@/components/ui/HubTabs.vue'
import PlannerPage from '@/pages/PlannerPage.vue'
import CalendarPage from '@/pages/CalendarPage.vue'
import IdeasPage from '@/pages/IdeasPage.vue'
import HashtagsPage from '@/pages/HashtagsPage.vue'

const route = useRoute()
const router = useRouter()

const TABS = [
  { id: 'monthly', label: 'Monthly' },
  { id: 'calendar', label: 'Calendar' },
  { id: 'ideas', label: 'Ideas' },
  { id: 'hashtags', label: 'Hashtags' },
]
const tab = ref(
  typeof route.query.tab === 'string' && TABS.some((t) => t.id === route.query.tab)
    ? (route.query.tab as string)
    : 'monthly',
)
const month = ref(typeof route.query.m === 'string' ? (route.query.m as string) : '')

watch([tab, month], () => {
  void router.replace({
    query: { ...route.query, tab: tab.value, ...(month.value ? { m: month.value } : {}) },
  })
})
</script>

<template>
  <div class="hubpage">
    <HubTabs :tabs="TABS" v-model="tab" />
    <PlannerPage
      v-if="tab === 'monthly'"
      :month="month || undefined"
      embedded
      @update:month="month = $event"
    />
    <CalendarPage v-else-if="tab === 'calendar'" />
    <IdeasPage v-else-if="tab === 'ideas'" />
    <HashtagsPage v-else />
  </div>
</template>
