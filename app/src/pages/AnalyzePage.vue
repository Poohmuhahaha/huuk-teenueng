<script setup lang="ts">
// Analyze hub — performance metrics and finance in one screen.
import { ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import HubTabs from '@/components/ui/HubTabs.vue'
import PerformancePage from '@/pages/PerformancePage.vue'
import FinancePage from '@/pages/FinancePage.vue'

const route = useRoute()
const router = useRouter()

const TABS = [
  { id: 'performance', label: 'Performance' },
  { id: 'finance', label: 'Finance' },
]
const tab = ref(
  typeof route.query.tab === 'string' && TABS.some((t) => t.id === route.query.tab)
    ? (route.query.tab as string)
    : 'performance',
)

watch(tab, () => {
  void router.replace({ query: { ...route.query, tab: tab.value } })
})
</script>

<template>
  <div class="hubpage">
    <HubTabs :tabs="TABS" v-model="tab" />
    <PerformancePage v-if="tab === 'performance'" />
    <FinancePage v-else />
  </div>
</template>
