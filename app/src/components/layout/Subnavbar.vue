<script setup lang="ts">
// Subnavbar — router glue for the shared HubTabs component: shows the tabs of
// the hub in view (or of the deck card being hovered) and routes between them.
import { computed, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { screenOf, screens } from '@/core/screens'
import { previewScreen, previewedScreen } from '@/core/subnav'
import HubTabs from '@/components/ui/HubTabs.vue'

const route = useRoute()
const router = useRouter()

const activeScreen = computed(() => screenOf(route.path))
const previewed = computed(() => screens.find((s) => s.path === previewedScreen.value))
const targetScreen = computed(() => previewed.value ?? activeScreen.value)
const tabs = computed(() => targetScreen.value?.tabs ?? [])

const activeTab = computed(() => {
  const q = route.query.tab
  if (typeof q === 'string' && tabs.value.some((t) => t.id === q)) return q
  return tabs.value[0]?.id ?? ''
})

// True while the pointer previews a hub other than the one on screen: tabs are
// shown for it but nothing is marked as the current selection yet.
const previewing = computed(
  () => !!previewed.value && previewed.value.path !== activeScreen.value?.path,
)

function go(id: string): void {
  const screen = targetScreen.value
  if (!screen) return
  if (screen.path === activeScreen.value?.path) {
    void router.replace({ query: { ...route.query, tab: id } })
  } else {
    void router.push({ path: screen.to, query: { tab: id } })
  }
}

watch(() => route.path, () => previewScreen(null))

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
  <nav v-if="tabs.length" class="subnavbar" :aria-label="`${targetScreen?.label ?? ''} sections`" @wheel="onWheel">
    <HubTabs :tabs="tabs" :model-value="previewing ? '' : activeTab" @update:model-value="go" />
  </nav>
</template>

<style scoped>
.subnavbar {
  padding: 10px 28px;
  border-bottom: 1px solid var(--line, #e4e4e4);
  background: rgba(255, 255, 255, 0.92);
  backdrop-filter: saturate(1.4) blur(10px);
  touch-action: pan-y;
}
.subnavbar :deep(.hubtabs) {
  margin: 0;
  justify-content: center;
}
@media (max-width: 720px) {
  .subnavbar { padding: 8px 18px; }
}
</style>
