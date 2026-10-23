<script setup lang="ts">
// MainNavPicker — one long button in the topbar (current hub + caret).
// Phones: toggles the Framer-style "Phone Open" expansion — the topbar pill
// itself grows (radius 999px → 30px) and the hub links stack full-width
// inside it, no floating dropdown. Desktop (brand pages): opens the
// dropdown menu instead. Reuses the global .tabselect skin (no <style>).
import { computed, onBeforeUnmount, ref, useTemplateRef, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { deckPeek } from '@/core/deck'
import { getRememberedTab } from '@/core/subnav'

interface NavLink {
  to: string
  label: string
}

const props = defineProps<{ links: NavLink[]; expanded: boolean }>()
const emit = defineEmits<{ toggle: [] }>()

const route = useRoute()
const router = useRouter()

function isActive(to: string, path: string = route.path): boolean {
  // Segment match: '/plan' must not match '/plans'. '/' matches only itself.
  if (to === '/') return path === '/'
  return path === to || path.startsWith(to + '/')
}

// The label follows the deck's live peek mid-drag (card covering >51% of
// the frame) and falls back to the route when the deck is at rest.
const activeLabel = computed(() => {
  const path = deckPeek.value ?? route.path
  return props.links.find((l) => isActive(l.to, path))?.label ?? props.links[0]?.label ?? ''
})

// Desktop dropdown state — phones use the expanding panel, never the menu.
const menuOpen = ref(false)
const root = useTemplateRef<HTMLElement>('root')
const isPhone = () => window.matchMedia('(max-width: 900px)').matches

function onBtn(): void {
  if (isPhone()) {
    menuOpen.value = false
    emit('toggle')
    return
  }
  menuOpen.value = !menuOpen.value
}

function go(to: string): void {
  menuOpen.value = false
  if (isActive(to)) return
  const remembered = getRememberedTab(to)
  if (remembered) {
    const { tab: _t, ...rest } = (route.query ?? {}) as Record<string, string>
    void router.push({ path: to, query: { ...rest, tab: remembered } })
  } else {
    void router.push(to)
  }
}

watch(
  () => route.path,
  () => { menuOpen.value = false },
)

function onDocClick(e: MouseEvent): void {
  if (menuOpen.value && root.value && !root.value.contains(e.target as Node)) menuOpen.value = false
}
function onDocKey(e: KeyboardEvent): void {
  if (e.key === 'Escape') menuOpen.value = false
}
watch(menuOpen, (v) => {
  if (v) {
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
</script>

<template>
  <div ref="root" class="tabselect in-topbar">
    <button type="button" class="tabselect-btn w-auto" :class="{ expanded: expanded || menuOpen }"
      aria-haspopup="listbox" :aria-expanded="expanded || menuOpen"
      aria-label="Main menu" @click.stop="onBtn">
      <span class="tabselect-label">{{ activeLabel }}</span>
      <span class="tabselect-caret" aria-hidden="true">▾</span>
    </button>
    <div v-if="menuOpen" class="tabselect-menu" role="listbox" aria-label="Main menu">
      <button v-for="l in links" :key="l.to" type="button" role="option" class="tabselect-item w-auto"
        :aria-selected="isActive(l.to)" :class="{ active: isActive(l.to) }"
        @click.stop="go(l.to)">
        <span>{{ l.label }}</span>
        <span v-if="isActive(l.to)" class="tabselect-check" aria-hidden="true">✓</span>
      </button>
    </div>
  </div>
</template>
