<script setup lang="ts">
// Analyze hub — performance metrics and finance in one screen.
import { useHubTab } from '@/core/subnav'
import PerformancePage from '@/pages/PerformancePage.vue'
import FinancePage from '@/pages/FinancePage.vue'

const { active: tab } = useHubTab('/analyze')
</script>

<template>
  <div class="hubpage">
    <PerformancePage v-if="tab === 'performance'" />
    <FinancePage v-else />
  </div>
</template>
