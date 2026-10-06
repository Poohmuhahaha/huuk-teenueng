import { onBeforeUnmount, onMounted, type Ref } from 'vue'

export function useDismiss(target: Ref<HTMLElement | null>, close: () => void): void {
  function onPointer(event: MouseEvent): void {
    if (target.value && !target.value.contains(event.target as Node)) close()
  }
  function onKey(event: KeyboardEvent): void {
    if (event.key === 'Escape') close()
  }
  onMounted(() => {
    document.addEventListener('mousedown', onPointer)
    document.addEventListener('keydown', onKey)
  })
  onBeforeUnmount(() => {
    document.removeEventListener('mousedown', onPointer)
    document.removeEventListener('keydown', onKey)
  })
}
