<script setup lang="ts">
// Card-slide navigation: every screen lives on one horizontal track.
// Drag / touch / arrow keys / dots slide the deck; the URL follows the slide.
import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { animate } from 'motion-v'
import { screens } from '@/core/screens'
import type { Screen } from '@/core/screens'
import { deckFull, deckPeek } from '@/core/deck'
import { currentRole } from '@/core/auth'
import { getRememberedTab } from '@/core/subnav'
import NotFoundPage from '@/pages/NotFoundPage.vue'

const route = useRoute()
const router = useRouter()

// Client accounts get a one-card deck (their Content Studio); staff see the
// whole journey. `currentRole` is plain session state, so this stays
// query-client free for unit tests.
const visible = computed<Screen[]>(() =>
  currentRole.value === 'Client'
    ? screens.filter((s) => s.match.some((m) => m.startsWith('/content')))
    : screens,
)

/** Segment match: '/plan' matches '/plan' and '/plan/…', never '/plans'. */
function matches(m: string, path: string): boolean {
  if (m === '/') return path === '/'
  return path === m || path.startsWith(m + '/')
}

function visibleIndexOf(path: string): number {
  return visible.value.findIndex((s) => s.match.some((m) => matches(m, path)))
}

/** True when the current path matches no screen (renders the 404 card). */
const notFound = computed(() => visibleIndexOf(route.path) < 0)

/** Route params a screen consumes. No screen takes params today (the planner
 * month travels via navDate + query), so this always returns {} — kept as the
 * single call-site for page props. */
function pageProps(_s: Screen): Record<string, unknown> {
  return {}
}
const track = ref<HTMLElement | null>(null)
// Start on the route's card (not card 0) so the first paint never flashes
// the wrong hub before the mount scroll runs.
const index = ref(notFound.value ? 0 : visibleIndexOf(route.path))
const full = ref(false)

// Performance optimization: only mount the active slide so inactive hubs
// never keep thousands of DOM nodes, SVG charts, and GPU layers alive at once.
function shouldRender(i: number): boolean {
  return i === index.value
}

// Publish the full-page state so pages can reflow (e.g. calendar 2 columns).
watch(full, (value) => { deckFull.value = value }, { immediate: true })
let syncing = false
let settle = 0

function prefersReduced(): boolean {
  return window.matchMedia?.('(prefers-reduced-motion: reduce)').matches ?? false
}

// Full-page toggle — GPU-composited container FLIP animation: transforms
// the card box from the deck slot into the full page (and back) via transform
// (scale + translate) to prevent layout reflows and stutter at 60-120fps.
type FullAnim = ReturnType<typeof animate>
let fullAnim: FullAnim | null = null
let fullBusy = false

function raf(): Promise<void> {
  return new Promise((resolve) => requestAnimationFrame(() => resolve()))
}

async function setFull(on: boolean, fromIndex?: number): Promise<void> {
  if (fullBusy || full.value === on) return
  const el = track.value
  const card = slides()[fromIndex ?? index.value]?.querySelector<HTMLElement>('.slide-card') ?? null
  if (!el || !card || prefersReduced()) {
    full.value = on
    await nextTick()
    if (!on) scrollToIndex(index.value, false)
    return
  }

  fullBusy = true
  fullAnim?.stop()
  fullAnim = null

  const first = card.getBoundingClientRect()
  const deckRect = el.getBoundingClientRect()

  const pin = (rect: { top: number; left: number; width: number; height: number }, initialTransform?: string): void => {
    card.style.position = 'fixed'
    card.style.top = `${rect.top}px`
    card.style.left = `${rect.left}px`
    card.style.width = `${rect.width}px`
    card.style.height = `${rect.height}px`
    card.style.margin = '0'
    card.style.zIndex = '20'
    card.style.transformOrigin = 'top left'
    card.style.willChange = 'transform, opacity'
    if (initialTransform) card.style.transform = initialTransform
  }
  const unpin = (): void => {
    for (const prop of ['position', 'top', 'left', 'width', 'height', 'margin', 'z-index', 'will-change', 'transform', 'transform-origin']) {
      card.style.removeProperty(prop)
    }
  }

  let startTransform = 'translate3d(0, 0, 0) scale(1, 1)'
  let targetTransform = 'translate3d(0, 0, 0) scale(1, 1)'

  if (on) {
    full.value = true
    const target = {
      top: deckRect.top,
      left: deckRect.left,
      width: Math.max(1, deckRect.width),
      height: Math.max(1, deckRect.height),
    }
    const dx = first.left - target.left
    const dy = first.top - target.top
    const sx = Math.max(0.01, first.width / target.width)
    const sy = Math.max(0.01, first.height / target.height)
    startTransform = `translate3d(${dx}px, ${dy}px, 0) scale(${sx}, ${sy})`
    targetTransform = 'translate3d(0, 0, 0) scale(1, 1)'
    pin(target, startTransform)
    el.classList.add('fading')
  } else {
    full.value = false
    await nextTick()
    scrollToIndex(index.value, false)
    await raf()
    const slot = card.getBoundingClientRect()
    const dx = slot.left - first.left
    const dy = slot.top - first.top
    const sx = Math.max(0.01, slot.width / Math.max(1, first.width))
    const sy = Math.max(0.01, slot.height / Math.max(1, first.height))
    startTransform = 'translate3d(0, 0, 0) scale(1, 1)'
    targetTransform = `translate3d(${dx}px, ${dy}px, 0) scale(${sx}, ${sy})`
    pin(first, startTransform)
  }

  await raf()
  const a = animate(
    card,
    {
      transform: [startTransform, targetTransform],
    },
    { duration: on ? 0.36 : 0.3, ease: [0.22, 1, 0.36, 1] },
  )
  fullAnim = a
  a.finished
    .then(async () => {
      if (fullAnim !== a) return
      fullAnim = null
      el.classList.remove('fading')
      unpin()
      fullBusy = false
    })
    .catch(() => {
      el.classList.remove('fading')
      unpin()
      fullBusy = false
    })
}

function slides(): HTMLElement[] {
  // Edge spacers are layout only — every index in this file addresses real
  // cards, so they are excluded here.
  return track.value ? Array.from(track.value.querySelectorAll<HTMLElement>('.slide:not(.spacer)')) : []
}

// scroll-padding keeps a 1-column peek on the leading edge (12.5vw on desktop, 10px on phone)
function padOf(el: HTMLElement): number {
  return Number.parseFloat(window.getComputedStyle(el).scrollPaddingLeft) || 0
}

function nearestIndex(): number {
  const el = track.value
  if (!el) return index.value
  const pad = padOf(el)
  let best = index.value
  let bestDist = Number.POSITIVE_INFINITY
  slides().forEach((s, i) => {
    if (s.offsetWidth === 0) return // collapsed slides share an offsetLeft
    // Mirror scrollToIndex's clamp: the first card snaps at 0, so without the
    // max() it could never be nearest and a drag there snapped back.
    const d = Math.abs(Math.max(0, s.offsetLeft - pad) - el.scrollLeft)
    if (d < bestDist) {
      bestDist = d
      best = i
    }
  })
  return best
}

let syncTimer = 0
let syncToken = 0

/** Marks a programmatic scroll as in-flight so `commit()` ignores it and
 * temporarily disengages native scroll-snap to prevent collision/stutter. */
function beginSync(ms: number): void {
  syncing = true
  const el = track.value
  if (el) el.style.scrollSnapType = 'none'
  const token = ++syncToken
  const done = (): void => {
    if (token === syncToken) {
      syncing = false
      if (el) el.style.scrollSnapType = ''
    }
  }
  window.clearTimeout(syncTimer)
  syncTimer = window.setTimeout(done, ms)
  if (el && 'onscrollend' in window) el.addEventListener('scrollend', done, { once: true })
}

function scrollToIndex(i: number, smooth = true): void {
  const el = track.value
  const target = slides()[i]
  if (!el || !target) return
  const left = Math.max(0, target.offsetLeft - padOf(el))
  const instant = !smooth || prefersReduced()
  if (instant) {
    el.scrollLeft = left
    return
  }
  const distance = Math.abs(left - el.scrollLeft)
  if (typeof el.scrollTo === 'function') el.scrollTo({ left, behavior: 'smooth' })
  else el.scrollLeft = left
  // Cover the smooth animation without fighting native snap or overshooting.
  beginSync(Math.round(Math.min(750, Math.max(260, 200 + distance * 0.3))))
}

function queryForScreen(s: Screen): Record<string, string> {
  const { tab: _currentTab, ...rest } = (route.query ?? {}) as Record<string, string>
  const remembered = getRememberedTab(s.path)
  if (remembered) {
    return { ...rest, tab: remembered }
  }
  return rest
}

async function goTo(i: number): Promise<void> {
  const clamped = Math.min(visible.value.length - 1, Math.max(0, i))
  index.value = clamped
  await nextTick()
  scrollToIndex(clamped)
  const s = visible.value[clamped]
  if (s && !s.match.some((m) => matches(m, route.path))) {
    router.replace({ path: s.to, query: queryForScreen(s) })
  }
}

function commit(): void {
  if (syncing || full.value) return
  const i = nearestIndex()
  index.value = i
  // Glue the peek to the settled card so the hub label never flashes back
  // to the pre-drag route while the router replace is in flight.
  deckPeek.value = visible.value[i]?.to ?? null
  const s = visible.value[i]
  if (s && !s.match.some((m) => matches(m, route.path))) {
    router.replace({ path: s.to, query: queryForScreen(s) })
  }
}

// Live leader: index of the card covering more than 51% of the deck frame
// right now (-1 when the drag sits between cards, keeping the previous
// peek so the label never flickers at the midpoint).
function leaderIndex(): number {
  const el = track.value
  if (!el || full.value || el.clientWidth === 0) return -1
  const viewL = el.scrollLeft
  const viewR = viewL + el.clientWidth
  let best = -1
  let bestCover = 0.51
  slides().forEach((s, i) => {
    if (s.offsetWidth === 0) return // collapsed slides (full-page mode) share an offsetLeft
    const overlap = Math.min(s.offsetLeft + s.offsetWidth, viewR) - Math.max(s.offsetLeft, viewL)
    const cover = overlap / el.clientWidth
    if (cover > bestCover) {
      bestCover = cover
      best = i
    }
  })
  return best
}

function onScroll(): void {
  // Synchronous peek: the hub label follows the finger mid-drag, the moment
  // the incoming card owns the frame — no waiting for release/settle.
  if (!full.value) {
    const leader = leaderIndex()
    if (leader >= 0) {
      const to = visible.value[leader]?.to ?? null
      if (to !== deckPeek.value) deckPeek.value = to
    }
  }
  window.clearTimeout(settle)
  settle = window.setTimeout(commit, 150)
}

onMounted(() => {
  if (!notFound.value) {
    index.value = visibleIndexOf(route.path)
    // Snap the track synchronously — before the first paint — so a deep
    // route never flashes card 0 for a frame; the rAF pass re-asserts
    // after fonts/layout settle.
    scrollToIndex(index.value, false)
    requestAnimationFrame(() => scrollToIndex(index.value, false))
  }
  // Non-passive so onWheel may preventDefault horizontal gestures; the
  // template keeps no @wheel binding (a .passive template listener could
  // never preventDefault and the console warns on every swipe).
  track.value?.addEventListener('wheel', onWheel, { passive: false })
  window.addEventListener('resize', onResize)
  window.addEventListener('keydown', onKeyGlobal)
})

// A role change can shrink the deck (staff 6 → Client 1): clamp the index so
// it never points past the last card, then snap the track to it.
watch(visible, (list) => {
  if (!list.length) return
  const clamped = Math.min(index.value, list.length - 1)
  if (clamped !== index.value) {
    index.value = clamped
    scrollToIndex(clamped, false)
  }
})

function onKeyGlobal(e: KeyboardEvent): void {
  if (e.key === 'Escape' && full.value) void setFull(false)
}

let resizeRaf: number | null = null
function onResize(): void {
  if (resizeRaf) return
  resizeRaf = requestAnimationFrame(() => {
    resizeRaf = null
    scrollToIndex(index.value, false)
  })
}

onBeforeUnmount(() => {
  if (resizeRaf) cancelAnimationFrame(resizeRaf)
  track.value?.removeEventListener('wheel', onWheel)
  window.removeEventListener('resize', onResize)
  window.removeEventListener('keydown', onKeyGlobal)
  window.clearTimeout(settle)
})

// Route changes can arrive from the outside (deep link resolution, back/
// forward). The first one after boot must snap, so a deep link does not play
// a slide show; later ones animate.
let routeApplied = false
watch(() => route.path, async (p) => {
  const i = visibleIndexOf(p)
  if (i >= 0 && i !== index.value) {
    index.value = i
    // External navigation (deep link, back/forward): drop any drag peek so
    // the label reads the route; the scroll animation's own scroll events
    // re-peek live if it travels across cards.
    deckPeek.value = null
    await nextTick()
    scrollToIndex(i, routeApplied)
  }
  routeApplied = true
})

function isEditable(el: HTMLElement): boolean {
  const tag = el.tagName
  return tag === 'INPUT' || tag === 'TEXTAREA' || tag === 'SELECT' || el.isContentEditable
}

function onSlideClick(i: number): void {
  if (i !== index.value && !full.value) {
    goTo(i)
  }
}

// Touchpad two-finger swipe (deltaX) and shift+wheel flip one card per
// gesture. Runs in the capture phase so a horizontal swipe is picked up even
// when the pointer is on top of a card; vertical wheel/touchpad is left
// untouched so the card's own content keeps scrolling up and down.
let wheelAt = 0
function onWheel(e: WheelEvent): void {
  if (full.value) return
  const horizontal = Math.abs(e.deltaX) >= Math.abs(e.deltaY) || e.shiftKey
  if (!horizontal) return
  e.preventDefault()
  const delta = e.deltaX || e.deltaY
  if (Math.abs(delta) < 4) return
  const now = Date.now()
  if (now < wheelAt) return
  wheelAt = now + 450
  goTo(index.value + (delta > 0 ? 1 : -1))
}

function onKey(e: KeyboardEvent): void {
  if (e.key !== 'ArrowRight' && e.key !== 'ArrowLeft') return
  // Never steal arrow keys from form fields or nested controls — only react
  // when the deck itself is the focused target.
  const target = e.target as HTMLElement | null
  if (target !== e.currentTarget || (target && isEditable(target))) return
  e.preventDefault()
  goTo(index.value + (e.key === 'ArrowRight' ? 1 : -1))
}

</script>

<template>
  <NotFoundPage v-if="notFound" />
  <div v-else class="deck-container">
    <div ref="track" class="deck" :class="{ full }" role="region" aria-roledescription="carousel" aria-label="Screens" tabindex="0"
      @scroll.passive="onScroll" @keydown="onKey">
      <div class="slide spacer" aria-hidden="true" />
        <section v-for="(s, i) in visible" :key="s.path" class="slide"
        :class="{ active: i === index }" :aria-label="`${i + 1} of ${visible.length}`"
        @click="onSlideClick(i)">
        <div class="slide-card">
          <div class="card-scroll" :class="{ 'card-scroll-brand': s.path === '/branding' }">
            <component v-if="shouldRender(i)" :is="s.component" v-bind="pageProps(s)" />
          </div>
        </div>
        <div class="slide-dock" />
      </section>
      <div class="slide spacer" aria-hidden="true" />
    </div>
  </div>
</template>
