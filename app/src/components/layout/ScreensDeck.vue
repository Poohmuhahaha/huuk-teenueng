<script setup lang="ts">
// Card-slide navigation: every screen lives on one horizontal track.
// Drag / touch / arrow keys / dots slide the deck; the URL follows the slide.
import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { animate } from 'motion-v'
import { screens } from '@/core/screens'
import type { Screen } from '@/core/screens'
import { previewScreen } from '@/core/subnav'
import { deckFull } from '@/core/deck'
import { currentRole } from '@/core/auth'
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

function visibleIndexOf(path: string): number {
  return visible.value.findIndex((s) => s.match.some((m) => path.startsWith(m)))
}

/** True when the current path matches no screen (renders the 404 card). */
const notFound = computed(() => visibleIndexOf(route.path) < 0)

/** Route params a screen consumes (e.g. the planner month). */
function pageProps(s: Screen): Record<string, unknown> {
  if (!s.path.includes(':month')) return {}
  const raw = Number(route.params.month)
  return Number.isInteger(raw) && raw >= 1 && raw <= 12 ? { month: String(raw) } : {}
}
const track = ref<HTMLElement | null>(null)
const index = ref(0)
const full = ref(false)

// Publish the full-page state so pages can reflow (e.g. calendar 2 columns).
watch(full, (value) => { deckFull.value = value }, { immediate: true })
let syncing = false
let settle = 0

function prefersReduced(): boolean {
  return window.matchMedia?.('(prefers-reduced-motion: reduce)').matches ?? false
}

// Full-page toggle — only the container animates: its box grows from the deck
// slot into the full page (and back) by animating left/width/height, so the
// content reflows at its normal size and nothing inside is scaled.
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

  const pin = (rect: { top: number; left: number; width: number; height: number }): void => {
    card.style.position = 'fixed'
    card.style.top = `${rect.top}px`
    card.style.left = `${rect.left}px`
    card.style.width = `${rect.width}px`
    card.style.height = `${rect.height}px`
    card.style.margin = '0'
    card.style.zIndex = '20'
    card.style.willChange = 'left, width, height'
  }
  const unpin = (): void => {
    for (const prop of ['position', 'top', 'left', 'width', 'height', 'margin', 'z-index', 'will-change']) {
      card.style.removeProperty(prop)
    }
  }

  // Pin the card as an overlay at its current rect before anything moves.
  pin(first)

  let target: { left: number; width: number; height: number }
  if (on) {
    // Grow to the full-page card rect (edge to edge, full height).
    target = {
      left: deckRect.left,
      width: Math.max(1, deckRect.width),
      height: Math.max(1, deckRect.height),
    }
    el.classList.add('fading') // other cards fade out while this one grows
  } else {
    // Reset to the deck slot first, then shrink the full page back into it.
    full.value = false
    await nextTick()
    scrollToIndex(index.value, false)
    await raf()
    // Measure the natural deck slot: unpin, read, re-pin (no paint in between).
    unpin()
    const slot = card.getBoundingClientRect()
    pin(first)
    target = {
      left: slot.left,
      width: Math.max(1, slot.width),
      height: Math.max(1, slot.height),
    }
  }

  await raf()
  const a = animate(
    card,
    {
      left: [first.left, target.left],
      width: [first.width, target.width],
      height: [first.height, target.height],
    },
    { duration: 0.56, ease: [0.22, 1, 0.36, 1] },
  )
  fullAnim = a
  a.finished
    .then(async () => {
      if (fullAnim !== a) return
      fullAnim = null
      if (on) {
        full.value = true // collapse the rest (they are already faded out)
        await nextTick()
      }
      el.classList.remove('fading')
      unpin()
      fullBusy = false
    })
    .catch(() => {
      el.classList.remove('fading')
      fullBusy = false
    })
}

function slides(): HTMLElement[] {
  // Edge spacers are layout only — every index in this file addresses real
  // cards, so they are excluded here.
  return track.value ? Array.from(track.value.querySelectorAll<HTMLElement>('.slide:not(.spacer)')) : []
}

// scroll-padding keeps a 1-column peek on the leading edge (12.5vw = 1/8 on desktop, 0 on phone)
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
    if (s.offsetWidth === 0) return // collapsed slides (full-page mode) share an offsetLeft
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

/** Marks a programmatic scroll as in-flight so `commit()` ignores it. */
function beginSync(ms: number): void {
  syncing = true
  const token = ++syncToken
  const done = (): void => {
    if (token === syncToken) syncing = false
  }
  window.clearTimeout(syncTimer)
  syncTimer = window.setTimeout(done, ms)
  const el = track.value
  if (el && 'onscrollend' in window) el.addEventListener('scrollend', done, { once: true })
}

function scrollToIndex(i: number, smooth = true): void {
  const el = track.value
  const target = slides()[i]
  if (!el || !target) return
  const left = Math.max(0, target.offsetLeft - padOf(el))
  const instant = !smooth || prefersReduced()
  if (instant) {
    // CSS `scroll-behavior: smooth` would animate this assignment; force it off
    // for one frame so deep links, resizes and full-page transitions snap.
    const previous = el.style.scrollBehavior
    el.style.scrollBehavior = 'auto'
    el.scrollLeft = left
    requestAnimationFrame(() => { el.style.scrollBehavior = previous })
    beginSync(120)
    return
  }
  const distance = Math.abs(left - el.scrollLeft)
  if (typeof el.scrollTo === 'function') el.scrollTo({ left, behavior: 'smooth' })
  else el.scrollLeft = left
  // Cover the whole smooth animation, scaled by distance, so an in-flight
  // scroll never commits a half-way index (which made the UI jump).
  beginSync(Math.round(Math.min(900, Math.max(300, 250 + distance * 0.35))))
}

function goTo(i: number): void {
  const clamped = Math.min(visible.value.length - 1, Math.max(0, i))
  index.value = clamped
  scrollToIndex(clamped)
  const s = visible.value[clamped]
  if (s && !s.match.some((m) => route.path.startsWith(m))) {
    router.replace({ path: s.to, query: route.query })
  }
}

function commit(): void {
  if (syncing || full.value) return
  const i = nearestIndex()
  index.value = i
  const s = visible.value[i]
  if (s && !s.match.some((m) => route.path.startsWith(m))) {
    router.replace({ path: s.to, query: route.query })
  }
}

function onScroll(): void {
  window.clearTimeout(settle)
  settle = window.setTimeout(commit, 150)
}

onMounted(() => {
  if (!notFound.value) {
    index.value = visibleIndexOf(route.path)
    requestAnimationFrame(() => scrollToIndex(index.value, false))
  }
  window.addEventListener('resize', onResize)
  window.addEventListener('keydown', onKeyGlobal)
})

function onKeyGlobal(e: KeyboardEvent): void {
  if (e.key === 'Escape' && full.value) void setFull(false)
}

function onResize(): void {
  scrollToIndex(index.value, false)
}

onBeforeUnmount(() => {
  window.removeEventListener('resize', onResize)
  window.removeEventListener('keydown', onKeyGlobal)
  window.clearTimeout(settle)
})

// Route changes can arrive from the outside (deep link resolution, back/
// forward). The first one after boot must snap, so a deep link does not play
// a slide show; later ones animate.
let routeApplied = false
watch(() => route.path, (p) => {
  const i = visibleIndexOf(p)
  if (i >= 0 && i !== index.value) {
    index.value = i
    scrollToIndex(i, routeApplied)
  }
  routeApplied = true
})

function isEditable(el: HTMLElement): boolean {
  const tag = el.tagName
  return tag === 'INPUT' || tag === 'TEXTAREA' || tag === 'SELECT' || el.isContentEditable
}

// Expand the card under the button — works from peeking cards too: the card
// is brought into the main slot (and the route follows) before going full.
function onExpand(i: number): void {
  if (full.value && i === index.value) {
    void setFull(false)
    return
  }
  if (i !== index.value) {
    index.value = i
    const s = visible.value[i]
    if (s && !s.match.some((m) => route.path.startsWith(m))) {
      router.replace({ path: s.to, query: route.query })
    }
  }
  void setFull(true, i)
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
  wheelAt = now + 420
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
  <div v-else>
    <div ref="track" class="deck" :class="{ full }" role="region" aria-label="Screens" tabindex="0"
      @scroll.passive="onScroll" @wheel.capture="onWheel" @keydown="onKey">
      <div class="slide spacer" aria-hidden="true" />
      <section v-for="(s, i) in visible" :key="s.path" class="slide"
        :class="{ active: i === index }"
        @mouseenter="previewScreen(s.path)">
        <div class="slide-card">
          <button
            class="expand-btn"
            :title="full && i === index ? 'Exit full page' : 'Go full page'"
            :aria-label="full && i === index ? 'Exit full page' : 'Go full page'"
            @click.stop="onExpand(i)"
          >
            <svg v-if="full && i === index" viewBox="0 0 16 16" width="14" height="14" aria-hidden="true"
              fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round">
              <path d="M2 2h4v4M14 2h-4v4M2 14h4v-4M14 14h-4v-4" />
            </svg>
            <svg v-else viewBox="0 0 16 16" width="14" height="14" aria-hidden="true"
              fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round">
              <path d="M2 6V2h4M14 6V2h-4M2 10v4h4M14 10v4h-4" />
            </svg>
          </button>
          <div class="card-scroll"><component :is="s.component" v-bind="pageProps(s)" /></div>
        </div>
      </section>
      <div class="slide spacer" aria-hidden="true" />
    </div>
  </div>
</template>
