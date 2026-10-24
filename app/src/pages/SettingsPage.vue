<script setup lang="ts">
// Branding hub — brand identity, members and platform connections behind one screen.
import { computed } from 'vue'
import { useHubTab } from '@/core/subnav'
import BrandPage from '@/pages/BrandPage.vue'
import MembersPage from '@/pages/MembersPage.vue'
import ConnectionsPage from '@/pages/ConnectionsPage.vue'

const { active: tab } = useHubTab('/branding')

const activeView = computed(() => {
  if (tab.value === 'members') return MembersPage
  if (tab.value === 'connections') return ConnectionsPage
  return BrandPage
})
</script>

<template>
  <div class="hubpage" :class="{ 'hubpage-brand': tab === 'brand' }">
    <component :is="activeView" />
  </div>
</template>
