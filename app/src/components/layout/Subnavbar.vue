<script setup lang="ts">
// Subnavbar — router glue for hub tabs: shows the tabs of the hub in view
// and routes between them. In `select` mode it collapses to a single button
// that opens a picker instead of a row of tab buttons.
import { computed, onBeforeUnmount, ref, useTemplateRef, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { screenOf } from '@/core/screens'
import HubTabs from '@/components/ui/HubTabs.vue'

withDefaults(defineProps<{ idPrefix?: string; select?: boolean }>(), { idPrefix: 'hubtab', select: false })

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

const activeLabel = computed(() => tabs.value.find((t) => t.id === activeTab.value)?.label ?? '')

// Single-button picker state (select mode only).
const selOpen = ref(false)
const selRoot = useTemplateRef<HTMLElement>('selRoot')
function pick(id: string): void {
  go(id)
  selOpen.value = false
}
function onDocClick(e: MouseEvent): void {
  if (selOpen.value && selRoot.value && !selRoot.value.contains(e.target as Node)) selOpen.value = false
}
function onDocKey(e: KeyboardEvent): void {
  if (e.key === 'Escape') selOpen.value = false
}
watch(selOpen, (open) => {
  if (open) {
    document.addEventListener('click', onDocClick)
    document.addEventListener('keydown', onDocKey)
  } else {
    document.removeEventListener('click', onDocClick)
    document.removeEventListener('keydown', onDocKey)
  }
})
onBeforeUnmount(() => {
  document.removeEventListener('click', onDocClick)
  document.removeEventListener('keydown', onDocKey)
})

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
    class="subnavbar py-2 px-4 sm:px-6 border-b border-line bg-surface/90 backdrop-blur-[10px] backdrop-saturate-[1.4] touch-pan-y flex justify-center items-center"
    :aria-label="`${activeScreen?.label ?? ''} sections`"
    @wheel="onWheel"
  >
    <div v-if="select" ref="selRoot" class="tabselect">
      <button type="button" class="tabselect-btn w-auto" aria-haspopup="listbox" :aria-expanded="selOpen"
        @click.stop="selOpen = !selOpen">
        <span class="tabselect-label">{{ activeLabel }}</span>
        <span class="tabselect-caret" aria-hidden="true">▾</span>
      </button>
      <div v-if="selOpen" class="tabselect-menu" role="listbox" :aria-label="`${activeScreen?.label ?? ''} sections`">
        <button v-for="t in tabs" :key="t.id" type="button" role="option" class="tabselect-item w-auto"
          :aria-selected="t.id === activeTab" :class="{ active: t.id === activeTab }"
          @click.stop="pick(t.id)">
          <span>{{ t.label }}</span>
          <span v-if="t.id === activeTab" class="tabselect-check" aria-hidden="true">✓</span>
        </button>
      </div>
    </div>
    <HubTabs v-else :tabs="tabs" :model-value="activeTab" :id-prefix="idPrefix" @update:model-value="go" />
  </nav>
</template>
