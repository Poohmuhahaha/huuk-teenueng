<script setup lang="ts">
// Subnavbar — router glue for the shared HubTabs component: shows the tabs of
// the hub in view and routes between them.
import { computed } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { screenOf } from '@/core/screens'
import HubTabs from '@/components/ui/HubTabs.vue'

const route = useRoute()
const router = useRouter()

const activeScreen = computed(() => screenOf(route.path))
const tabs = computed(() => activeScreen.value?.tabs ?? [])

const activeTab = computed(() => {
  const q = route.query.tab
  if (typeof q === 'string' && tabs.value.some((t) => t.id === q)) return q
  return tabs.value[0]?.id ?? ''
})

function go(id: string): void {
  if (!activeScreen.value) return
  void router.replace({ query: { ...route.query, tab: id } })
}

// The bar is chrome, not a carousel: a horizontal two-finger swipe over the
// tabs must do nothing at all (no browser back/forward, no deck flip). The
// root sets overscroll-behavior-x: none; swallowing the wheel here keeps the
// gesture dead even on browsers that ignore that, and never touches vertical
// scrolling.
function onWheel(e: WheelEvent): void {
  if (Math.abs(e.deltaX) >= Math.abs(e.deltaY)) e.preventDefault()
}
</script>

<template>
  <nav
    v-if="tabs.length"
    class="py-2 px-4 sm:px-6 border-b border-line bg-surface/90 backdrop-blur-[10px] backdrop-saturate-[1.4] touch-pan-y flex justify-center items-center"
    :aria-label="`${activeScreen?.label ?? ''} sections`"
    @wheel="onWheel"
  >
    <HubTabs :tabs="tabs" :model-value="activeTab" @update:model-value="go" />
  </nav>
</template>
