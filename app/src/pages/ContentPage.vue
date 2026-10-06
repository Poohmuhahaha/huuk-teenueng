<script setup lang="ts">
// Content hub — the CMS studio plus the feed preview.
import { ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useHubTab } from '@/core/subnav'
import StudioPage from '@/pages/StudioPage.vue'
import FeedPage from '@/pages/FeedPage.vue'

const route = useRoute()
const router = useRouter()

const { active: tab } = useHubTab('/content')
const id = ref(typeof route.query.id === 'string' ? (route.query.id as string) : '')

watch(id, () => {
  void router.replace({
    query: { ...route.query, tab: tab.value, ...(id.value ? { id: id.value } : {}) },
  })
})
</script>

<template>
  <div class="hubpage">
    <StudioPage
      v-if="tab === 'studio'"
      :id="id || undefined"
      embedded
      @update:id="id = $event"
    />
    <FeedPage v-else />
  </div>
</template>
