<script setup lang="ts">
import { nextTick, onBeforeUnmount, onMounted, ref } from 'vue'

defineProps<{ label: string }>()

const dockEl = ref<HTMLElement | null>(null)
const teleportTarget = ref<HTMLElement | null>(null)

function resolveTarget(): void {
  const slide = dockEl.value?.closest('.slide')
  if (slide) {
    const slot = slide.querySelector<HTMLElement>(':scope > .slide-dock')
    if (slot) {
      teleportTarget.value = slot
    }
  }
}

onMounted(async () => {
  await nextTick()
  resolveTarget()
})

onBeforeUnmount(() => {
  teleportTarget.value = null
})
</script>

<template>
  <Teleport :to="teleportTarget" :disabled="!teleportTarget">
    <aside ref="dockEl" class="dock">
      <div class="dock-bar" role="group" :aria-label="label"><slot /></div>
    </aside>
  </Teleport>
</template>
