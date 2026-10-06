<script setup lang="ts">
// Plan hub — monthly planner + calendar + ideas + hashtags in one screen.
import { ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useHubTab } from '@/core/subnav'
import PlannerPage from '@/pages/PlannerPage.vue'
import CalendarPage from '@/pages/CalendarPage.vue'
import IdeasPage from '@/pages/IdeasPage.vue'
import HashtagsPage from '@/pages/HashtagsPage.vue'

const route = useRoute()
const router = useRouter()

const { active: tab } = useHubTab('/plan')
const month = ref(typeof route.query.m === 'string' ? (route.query.m as string) : '')

watch(month, () => {
  void router.replace({
    query: { ...route.query, tab: tab.value, ...(month.value ? { m: month.value } : {}) },
  })
})
</script>

<template>
  <div class="hubpage">
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
