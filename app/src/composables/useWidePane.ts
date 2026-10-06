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

  function sync(): void {
    const card = root.value?.closest<HTMLElement>('.slide-card')
    const pane = root.value?.closest<HTMLElement>('.card-scroll')
    if (!card || !pane) return
    wide.value = card.clientWidth >= at
    pane.classList.toggle(cls, wide.value)
  }

  onMounted(() => {
    sync()
    const card = root.value?.closest<HTMLElement>('.slide-card')
    if (card && typeof ResizeObserver !== 'undefined') {
      observer = new ResizeObserver(sync)
      observer.observe(card)
    }
  })

  onBeforeUnmount(() => {
    observer?.disconnect()
    root.value?.closest<HTMLElement>('.card-scroll')?.classList.remove(cls)
  })

  return { wide }
}
