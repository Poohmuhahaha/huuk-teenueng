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

// Hidden hub cards stay mounted in the deck, so only the active hub may write
// its selection into the URL — otherwise Content and Promote overwrite each
// other's `?id`/`?tab` on load.
function onHub(): boolean {
  return route.path === '/content' || route.path.startsWith('/content/')
}

const id = ref(onHub() && typeof route.query.id === 'string' ? (route.query.id as string) : '')

watch(
  () => route.query.id,
  (value) => {
    if (!onHub()) return
    const next = typeof value === 'string' ? value : ''
    if (next && next !== id.value) id.value = next
  },
)

watch(id, () => {
  if (!onHub()) return
  const nextId = id.value || ''
  const currentId = typeof route.query.id === 'string' ? route.query.id : ''
  if (nextId === currentId && route.query.tab === tab.value) return
  void router.replace({
    query: { ...route.query, tab: tab.value, ...(nextId ? { id: nextId } : {}) },
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
