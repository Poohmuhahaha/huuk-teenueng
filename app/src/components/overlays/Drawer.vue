<script setup lang="ts">
import { onBeforeUnmount, onMounted, useId } from 'vue'
import { t } from '@/core/i18n'

const open = defineModel<boolean>({ default: false })
const props = withDefaults(
  defineProps<{
    side?: 'right' | 'left' | 'bottom'
    title?: string
    width?: string
    /** Anchor to the nearest positioned ancestor (the card) instead of the viewport. */
    contained?: boolean
    /** Render as a plain in-flow panel (wide master–detail layouts). */
    inline?: boolean
    /** Allow backdrop click / ESC to dismiss. */
    dismissable?: boolean
  }>(),
  { side: 'right', width: '360px', dismissable: true },
)

const uid = useId()
const titleId = `${uid}-title`

function onKey(e: KeyboardEvent): void {
  if (e.key === 'Escape' && open.value) open.value = false
}
onMounted(() => window.addEventListener('keydown', onKey))
onBeforeUnmount(() => window.removeEventListener('keydown', onKey))
</script>

<template>
  <aside
    v-if="inline && open"
    class="p-drawer inline static w-full max-w-none min-h-0 bg-transparent border-0 shadow-none overflow-hidden flex flex-col"
    role="region"
    :aria-label="title"
    :aria-labelledby="title ? titleId : undefined"
  >
    <header class="flex items-center justify-between gap-3 p-3.5 px-4 border-b border-line">
      <h3 v-if="title" :id="titleId" class="m-0 text-lg font-display text-ink truncate">{{ title }}</h3>
      <button type="button" class="p-drawer-x w-auto bg-transparent border-0 text-[22px] leading-none text-muted cursor-pointer p-0 hover:text-ink" :aria-label="t('common.close')" @click="open = false">×</button>
    </header>
    <div class="flex-1 overflow-y-auto p-4"><slot /></div>
    <footer v-if="$slots.footer" class="flex justify-end gap-2.5 p-3 px-4 border-t border-line"><slot name="footer" /></footer>
  </aside>

  <Transition name="p-drawer-anim" :duration="280">
    <div
      v-if="!inline && open"
      class="p-drawer-backdrop z-[1000] bg-black/45 backdrop-blur-[2px] flex flex-col justify-end overflow-hidden"
      :class="[
        contained ? 'absolute inset-0' : 'fixed inset-0',
        side === 'bottom' ? 'pt-3' : '',
        `p-drawer-side-${side}`,
      ]"
      @click.self="dismissable ? open = false : null"
    >
      <aside
        class="p-drawer flex flex-col bg-surface border border-line shadow-[0_-12px_40px_rgba(0,0,0,0.22)] overflow-hidden"
        :class="[
          side === 'right' ? 'ml-auto h-full rounded-l-2xl max-w-[94vw]' : '',
          side === 'left' ? 'mr-auto h-full rounded-r-2xl max-w-[94vw]' : '',
          side === 'bottom'
            ? 'w-full max-w-none rounded-t-[20px] h-[calc(100%-12px)] max-h-[calc(100%-12px)] border-b-0'
            : '',
        ]"
        :style="side === 'bottom' ? undefined : { width }"
        role="dialog"
        aria-modal="true"
        :aria-labelledby="title ? titleId : undefined"
      >
        <!-- Pull / drag handle indicator at top of bottom sheet -->
        <div v-if="side === 'bottom'" class="w-10 h-1 bg-muted/40 rounded-full mx-auto mt-2.5 mb-1 flex-none" />
        <header class="flex items-center justify-between gap-3 p-3.5 px-4 border-b border-line">
          <h3 v-if="title" :id="titleId" class="m-0 text-lg font-display text-ink truncate">{{ title }}</h3>
          <button type="button" class="p-drawer-x w-auto bg-transparent border-0 text-[22px] leading-none text-muted cursor-pointer p-0 hover:text-ink" :aria-label="t('common.close')" @click="open = false">×</button>
        </header>
        <div class="flex-1 overflow-y-auto p-4"><slot /></div>
        <footer v-if="$slots.footer" class="flex justify-end gap-2.5 p-3 px-4 border-t border-line"><slot name="footer" /></footer>
      </aside>
    </div>
  </Transition>
</template>

<style scoped>
/* Backdrop fade */
.p-drawer-anim-enter-active,
.p-drawer-anim-leave-active {
  transition: opacity 0.28s ease-out;
}
.p-drawer-anim-leave-active {
  transition: opacity 0.2s ease-in;
}
.p-drawer-anim-enter-from,
.p-drawer-anim-leave-to {
  opacity: 0;
}
.p-drawer-anim-enter-to,
.p-drawer-anim-leave-from {
  opacity: 1;
}

/* Base sheet: permanent GPU compositing to eliminate subpixel raster snaps */
.p-drawer:not(.inline) {
  transform: translate3d(0, 0, 0);
  backface-visibility: hidden;
}

/* Base sheet slide transition: clean ease-out slide with zero bounce or overshoot */
.p-drawer-anim-enter-active .p-drawer {
  transition: transform 0.28s ease-out;
  will-change: transform;
}
.p-drawer-anim-leave-active .p-drawer {
  transition: transform 0.22s ease-in;
  will-change: transform;
}

/* Bottom Sheet: slides UP from translateY(100%) to translateY(0) */
.p-drawer-anim-enter-from.p-drawer-side-bottom .p-drawer,
.p-drawer-anim-leave-to.p-drawer-side-bottom .p-drawer {
  transform: translate3d(0, 100%, 0);
}
.p-drawer-anim-enter-to.p-drawer-side-bottom .p-drawer,
.p-drawer-anim-leave-from.p-drawer-side-bottom .p-drawer {
  transform: translate3d(0, 0, 0);
}

/* Right Drawer: slides from translateX(100%) */
.p-drawer-anim-enter-from.p-drawer-side-right .p-drawer,
.p-drawer-anim-leave-to.p-drawer-side-right .p-drawer {
  transform: translate3d(100%, 0, 0);
}
.p-drawer-anim-enter-to.p-drawer-side-right .p-drawer,
.p-drawer-anim-leave-from.p-drawer-side-right .p-drawer {
  transform: translate3d(0, 0, 0);
}

/* Left Drawer: slides from translateX(-100%) */
.p-drawer-anim-enter-from.p-drawer-side-left .p-drawer,
.p-drawer-anim-leave-to.p-drawer-side-left .p-drawer {
  transform: translate3d(-100%, 0, 0);
}
.p-drawer-anim-enter-to.p-drawer-side-left .p-drawer,
.p-drawer-anim-leave-from.p-drawer-side-left .p-drawer {
  transform: translate3d(0, 0, 0);
}

@media (prefers-reduced-motion: reduce) {
  .p-drawer-anim-enter-active,
  .p-drawer-anim-leave-active,
  .p-drawer-anim-enter-active .p-drawer,
  .p-drawer-anim-leave-active .p-drawer {
    transition: none !important;
  }
}
</style>
