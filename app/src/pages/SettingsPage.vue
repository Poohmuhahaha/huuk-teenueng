<script setup lang="ts">
// Settings hub — brand identity, workspace config + permissions, members and
// platform connections behind one screen.
import { ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import HubTabs from '@/components/ui/HubTabs.vue'
import BrandPage from '@/pages/BrandPage.vue'
import MembersPage from '@/pages/MembersPage.vue'
import SettingsPanel from '@/components/overlays/SettingsPanel.vue'
import SocialMediaBar from '@/components/ui/SocialMediaBar.vue'

const route = useRoute()
const router = useRouter()

const TABS = [
  { id: 'brand', label: 'Brand' },
  { id: 'workspace', label: 'Workspace' },
  { id: 'members', label: 'Members' },
  { id: 'connections', label: 'Connections' },
]
const tab = ref(
  typeof route.query.tab === 'string' && TABS.some((t) => t.id === route.query.tab)
    ? (route.query.tab as string)
    : 'brand',
)

watch(tab, () => {
  void router.replace({ query: { ...route.query, tab: tab.value } })
})
</script>

<template>
  <div class="hubpage">
    <HubTabs :tabs="TABS" v-model="tab" />
    <BrandPage v-if="tab === 'brand'" />
    <SettingsPanel v-else-if="tab === 'workspace'" />
    <MembersPage v-else-if="tab === 'members'" />
    <div v-else class="card">
      <p class="muted" style="margin-top: 0;">Connect the social accounts this workspace publishes to.</p>
      <SocialMediaBar />
    </div>
  </div>
</template>
