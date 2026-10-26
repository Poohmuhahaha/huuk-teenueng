// Cards that turn into a wide master–detail pane (configs | items | detail)
// when the card itself is wide enough. The layout follows the card's real
// width — via ResizeObserver — so it aligns progressively while the card
// expands instead of snapping when the deck flips to full page.
import { onBeforeUnmount, onMounted, ref, type Ref } from 'vue'

export function useWidePane(
  root: Ref<HTMLElement | null>,
  cls: string,
  at = 820,
): { wide: Ref<boolean> } {
  const wide = ref(false)
  let observer: ResizeObserver | undefined
  let rafId: number | null = null

  function sync(): void {
    const card = root.value?.closest<HTMLElement>('.slide-card')
    const pane = root.value?.closest<HTMLElement>('.card-scroll')
    if (!card || !pane) return
    const isWide = card.clientWidth >= at
    if (wide.value !== isWide) {
      wide.value = isWide
      pane.classList.toggle(cls, isWide)
    }
  }

  function debouncedSync(): void {
    if (rafId) return
    rafId = requestAnimationFrame(() => {
      rafId = null
      sync()
    })
  }

  onMounted(() => {
    sync()
    const card = root.value?.closest<HTMLElement>('.slide-card')
    if (card && typeof ResizeObserver !== 'undefined') {
      observer = new ResizeObserver(debouncedSync)
      observer.observe(card)
    }
  })

  onBeforeUnmount(() => {
    if (rafId) cancelAnimationFrame(rafId)
    observer?.disconnect()
    root.value?.closest<HTMLElement>('.card-scroll')?.classList.remove(cls)
  })

  return { wide }
}
