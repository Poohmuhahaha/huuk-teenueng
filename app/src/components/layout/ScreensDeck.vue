<script setup lang="ts">
// Card-slide navigation: every screen lives on one horizontal track.
// Drag / touch / arrow keys / dots slide the deck; the URL follows the slide.
import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { animate } from 'motion-v'
import { screens } from '@/core/screens'
import type { Screen } from '@/core/screens'
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
let syncing = false
let settle = 0

function toggleFull(): void {
  void setFull(!full.value)
}

function prefersReduced(): boolean {
  return window.matchMedia?.('(prefers-reduced-motion: reduce)').matches ?? false
}

function activeCard(): HTMLElement | null {
  return slides()[index.value]?.querySelector<HTMLElement>('.slide-card') ?? null
}

// Full-page toggle — the card grows/shrinks (animated width + left, content
// reflows with it) while the other cards cross-fade at the same time.
// The card is pinned as a fixed overlay so only its own content reflows.
type FullAnim = ReturnType<typeof animate>
let fullAnim: FullAnim | null = null
let fullBusy = false

function raf(): Promise<void> {
  return new Promise((resolve) => requestAnimationFrame(() => resolve()))
}

async function setFull(on: boolean): Promise<void> {
  if (fullBusy || full.value === on) return
  const el = track.value
  const card = activeCard()
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

  // Pin the card as an overlay at its current rect before anything moves.
  card.style.position = 'fixed'
  card.style.top = `${first.top}px`
  card.style.left = `${first.left}px`
  card.style.width = `${first.width}px`
  card.style.height = `${first.height}px`
  card.style.margin = '0'
  card.style.zIndex = '20'
  card.style.willChange = 'width, left'

  let target: { left: number; width: number }
  if (on) {
    // Grow to the deck's viewport — exactly, with no leftover padding.
    target = { left: deckRect.left, width: Math.max(1, deckRect.width) }
    el.classList.add('fading') // other cards fade out while this one grows
  } else {
    // Shrink back into the real paged slot: release the layout first, snap the
    // deck instantly, then measure where the card actually sits.
    full.value = false
    await nextTick()
    scrollToIndex(index.value, false)
    await raf()
    const slot = slides()[index.value]?.getBoundingClientRect()
    target = slot
      ? { left: slot.left, width: Math.max(1, slot.width) }
      : { left: first.left, width: Math.max(1, first.width) }
  }

  await raf()
  const a = animate(
    card,
    { left: [first.left, target.left], width: [first.width, target.width] },
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
      card.style.removeProperty('position')
      card.style.removeProperty('top')
      card.style.removeProperty('left')
      card.style.removeProperty('width')
      card.style.removeProperty('height')
      card.style.removeProperty('margin')
      card.style.removeProperty('z-index')
      card.style.removeProperty('will-change')
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

// drag + touch. Pointer capture is deferred until the drag actually starts,
// so plain clicks (buttons, tables, forms) keep their real target.
let dragging = false
let moved = false
let startX = 0
let startLeft = 0

function onDown(e: PointerEvent): void {
  // Touch scrolls natively (momentum + snap) — the custom drag is a
  // mouse/pen-only enhancement. Hijacking touch kills momentum, fights the
  // CSS snap, and dies on pointercancel, which made mobile sliding feel stuck.
  if (e.pointerType === 'touch') return
  if (e.button !== 0) return
  const el = track.value
  if (!el) return
  dragging = true
  moved = false
  startX = e.clientX
  startLeft = el.scrollLeft
  el.classList.add('dragging')
}

function onMove(e: PointerEvent): void {
  const el = track.value
  if (!dragging || !el) return
  const dx = e.clientX - startX
  if (!moved && Math.abs(dx) > 8) {
    moved = true
    el.style.scrollBehavior = 'auto'
    el.style.scrollSnapType = 'none'
    try { el.setPointerCapture(e.pointerId) } catch { /* no-op */ }
  }
  if (moved) el.scrollLeft = startLeft - dx
}

function onUp(): void {
  const el = track.value
  if (!dragging || !el) return
  dragging = false
  el.classList.remove('dragging')
  el.style.removeProperty('scroll-behavior')
  el.style.removeProperty('scroll-snap-type')
  if (!moved) return
  const target = slides()[nearestIndex()]
  if (target) el.scrollLeft = Math.max(0, target.offsetLeft - padOf(el))
  commit()
  window.setTimeout(() => { moved = false }, 0)
}

function onClickCapture(e: Event): void {
  if (moved) {
    e.preventDefault()
    e.stopPropagation()
    moved = false
  }
}

function isEditable(el: HTMLElement): boolean {
  const tag = el.tagName
  return tag === 'INPUT' || tag === 'TEXTAREA' || tag === 'SELECT' || el.isContentEditable
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

// Tap a peeking card to slide it fully into the main slot. Capture phase so
// controls inside a peeking (non-active) card never fire on the way in.
function onSlideClick(i: number, e: MouseEvent): void {
  if (i === index.value) return
  e.preventDefault()
  e.stopPropagation()
  const wantsFull = (e.target as HTMLElement | null)?.closest?.('.expand-btn') != null
  if (wantsFull) {
    index.value = i
    scrollToIndex(i, false)
    void setFull(true)
  } else {
    goTo(i)
  }
}
</script>

<template>
  <NotFoundPage v-if="notFound" />
  <div v-else>
    <div ref="track" class="deck" :class="{ full }" role="region" aria-label="Screens" tabindex="0"
      @scroll.passive="onScroll" @pointerdown="onDown" @pointermove="onMove"
      @pointerup="onUp" @pointercancel="onUp" @click.capture="onClickCapture" @keydown="onKey">
      <div class="slide spacer" aria-hidden="true" />
      <section v-for="(s, i) in visible" :key="s.path" class="slide"
        :class="{ active: i === index }"
        @click.capture="onSlideClick(i, $event)">
        <div class="slide-card">
          <button class="expand-btn" :title="full && i === index ? 'Exit full page' : 'Go full page'"
            @click.stop="toggleFull">
            {{ full && i === index ? 'Exit full page' : 'Go full page' }}
          </button>
          <div class="card-scroll"><component :is="s.component" v-bind="pageProps(s)" /></div>
        </div>
      </section>
      <div class="slide spacer" aria-hidden="true" />
    </div>
  </div>
</template>
