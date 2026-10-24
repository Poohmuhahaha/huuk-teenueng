<script setup lang="ts">
// Analyze hub — performance metrics and finance in one screen.
import { computed } from 'vue'
import { useHubTab } from '@/core/subnav'
import PerformancePage from '@/pages/PerformancePage.vue'
import FinancePage from '@/pages/FinancePage.vue'

const { active: tab } = useHubTab('/analyze')
const activeView = computed(() => (tab.value === 'finance' ? FinancePage : PerformancePage))
</script>

<template>
  <div class="hubpage">
    <component :is="activeView" />
  </div>
</template>
