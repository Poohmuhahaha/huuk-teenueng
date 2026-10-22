<script setup lang="ts">
// Branding hub — brand identity, members and platform connections behind one screen.
import { useHubTab } from '@/core/subnav'
import BrandPage from '@/pages/BrandPage.vue'
import MembersPage from '@/pages/MembersPage.vue'
import ConnectionsPage from '@/pages/ConnectionsPage.vue'

const { active: tab } = useHubTab('/branding')
</script>

<template>
  <div class="hubpage" :class="{ 'hubpage-brand': tab === 'brand' }">
    <BrandPage v-if="tab === 'brand'" />
    <MembersPage v-else-if="tab === 'members'" />
    <ConnectionsPage v-else />
  </div>
</template>
