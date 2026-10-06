<script setup lang="ts">
// Settings hub — brand identity, workspace config + permissions, members and
// platform connections behind one screen.
import { useHubTab } from '@/core/subnav'
import BrandPage from '@/pages/BrandPage.vue'
import MembersPage from '@/pages/MembersPage.vue'
import SettingsPanel from '@/components/overlays/SettingsPanel.vue'
import SocialMediaBar from '@/components/ui/SocialMediaBar.vue'

const { active: tab } = useHubTab('/settings')
</script>

<template>
  <div class="hubpage">
    <BrandPage v-if="tab === 'brand'" />
    <SettingsPanel v-else-if="tab === 'workspace'" />
    <MembersPage v-else-if="tab === 'members'" />
    <div v-else class="card">
      <p class="muted" style="margin-top: 0;">Connect the social accounts this workspace publishes to.</p>
      <SocialMediaBar />
    </div>
  </div>
</template>
