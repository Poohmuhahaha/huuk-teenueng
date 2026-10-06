<script setup lang="ts">
const open = defineModel<boolean>({ default: false })
withDefaults(
  defineProps<{
    side?: 'right' | 'left' | 'bottom'
    title?: string
    width?: string
    /** Anchor to the nearest positioned ancestor (the card) instead of the viewport. */
    contained?: boolean
    /** Render as a plain in-flow panel (wide master–detail layouts). */
    inline?: boolean
  }>(),
  { side: 'right', width: '360px' },
)
</script>

<template>
  <aside v-if="inline && open" class="p-drawer inline" :class="side" role="region">
    <header class="p-drawer-head">
      <h3 v-if="title" class="p-drawer-title">{{ title }}</h3>
      <button type="button" class="p-drawer-x" aria-label="Close" @click="open = false">×</button>
    </header>
    <div class="p-drawer-body"><slot /></div>
    <footer v-if="$slots.footer" class="p-drawer-foot"><slot name="footer" /></footer>
  </aside>

  <div v-else-if="open" class="p-drawer-backdrop" :class="{ contained }" @click.self="open = false">
    <aside
      class="p-drawer"
      :class="side"
      :style="side === 'bottom' ? undefined : { width }"
      role="dialog"
      aria-modal="true"
    >
      <header class="p-drawer-head">
        <h3 v-if="title" class="p-drawer-title">{{ title }}</h3>
        <button type="button" class="p-drawer-x" aria-label="Close" @click="open = false">×</button>
      </header>
      <div class="p-drawer-body"><slot /></div>
      <footer v-if="$slots.footer" class="p-drawer-foot"><slot name="footer" /></footer>
    </aside>
  </div>
</template>

<style scoped>
.p-drawer-backdrop {
  position: fixed;
  inset: 0;
  z-index: 1000;
  background: rgba(0, 0, 0, 0.4);
  display: flex;
}
/* Contained: cover just the card it lives in (nearest positioned ancestor). */
.p-drawer-backdrop.contained {
  position: absolute;
}
/* Inline: a plain panel that flows in its parent (no overlay, no chrome). */
.p-drawer.inline {
  position: static;
  width: 100%;
  max-width: none;
  min-height: 0;
  background: transparent;
  border: 0;
  box-shadow: none;
  animation: none;
  overflow: hidden;
}
.p-drawer {
  display: flex;
  flex-direction: column;
  background: var(--paper, #fff);
  border: 1px solid var(--line, #e4e4e4);
  box-shadow: 0 12px 40px rgba(0, 0, 0, 0.12);
  max-width: 94vw;
}
.p-drawer.right {
  margin-left: auto;
  border-radius: 12px 0 0 12px;
  animation: sheet-in-right 0.18s ease-out;
}
.p-drawer.left {
  margin-right: auto;
  border-radius: 0 12px 12px 0;
  animation: sheet-in-left 0.18s ease-out;
}
.p-drawer.bottom {
  margin-top: auto;
  width: 100%;
  max-width: none;
  max-height: 80vh;
  border-radius: 12px 12px 0 0;
  animation: sheet-in-up 0.2s ease-out;
}
/* Contained bottom sheet fills the card completely. */
.p-drawer-backdrop.contained .p-drawer.bottom {
  max-height: none;
  height: 100%;
  border-radius: 0;
}
@keyframes sheet-in-right {
  from { transform: translateX(100%); }
  to { transform: none; }
}
@keyframes sheet-in-left {
  from { transform: translateX(-100%); }
  to { transform: none; }
}
@keyframes sheet-in-up {
  from { transform: translateY(100%); }
  to { transform: none; }
}
.p-drawer-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  padding: 14px 16px;
  border-bottom: 1px solid var(--line, #e4e4e4);
}
.p-drawer-title { margin: 0; font-size: 18px; font-family: var(--font-display, Georgia, serif); }
.p-drawer-x {
  width: auto;
  background: transparent;
  border: 0;
  font-size: 22px;
  line-height: 1;
  color: var(--muted, #6b6b6b);
  cursor: pointer;
  padding: 0;
}
.p-drawer-x:hover { color: var(--ink, #000); }
.p-drawer-body { flex: 1; overflow-y: auto; padding: 16px; }
.p-drawer-foot {
  display: flex;
  justify-content: flex-end;
  gap: 10px;
  padding: 12px 16px;
  border-top: 1px solid var(--line, #e4e4e4);
}
</style>
