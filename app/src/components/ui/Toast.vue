<script setup lang="ts">
import { watch } from 'vue'

const open = defineModel<boolean>({ default: false })
const props = withDefaults(
  defineProps<{ title?: string; message?: string; tone?: 'info' | 'success' | 'warning' | 'danger'; duration?: number; fixed?: boolean }>(),
  { tone: 'info', duration: 4000 },
)
let timer: number | undefined

watch(open, (value) => {
  if (!value) return
  if (timer) window.clearTimeout(timer)
  if (props.duration > 0) timer = window.setTimeout(() => (open.value = false), props.duration)
})
</script>

<template>
  <div class="p-toast" :class="[`t-${tone}`, { fixed }]" role="status">
    <div class="p-toast-main">
      <strong v-if="title" class="p-toast-title">{{ title }}</strong>
      <span v-if="message" class="p-toast-msg">{{ message }}</span>
      <slot />
    </div>
    <button type="button" class="p-toast-x" aria-label="Dismiss" @click="open = false">×</button>
  </div>
</template>

<style scoped>
.p-toast {
  display: flex;
  align-items: flex-start;
  gap: 12px;
  width: 100%;
  background: var(--paper, #fff);
  border: 1px solid var(--ink, #000);
  border-left-width: 4px;
  padding: 12px 14px;
}
.p-toast.fixed {
  position: fixed;
  right: 20px;
  bottom: 20px;
  z-index: 1000;
  width: auto;
  min-width: 280px;
  max-width: 380px;
  box-shadow: 4px 4px 0 var(--ink, #000);
}
.t-success { border-left-style: double; }
.t-warning { border-left-style: dashed; }
.t-danger { background: var(--wash, #f6f6f6); }
.p-toast-main { display: flex; flex-direction: column; gap: 2px; flex: 1; }
.p-toast-title { font-size: 13px; }
.p-toast-msg { font-size: 12px; color: var(--muted, #6b6b6b); }
.p-toast-x {
  width: auto;
  background: transparent;
  border: 0;
  font-size: 18px;
  line-height: 1;
  color: var(--muted, #6b6b6b);
  cursor: pointer;
  padding: 0;
}
.p-toast-x:hover { color: var(--ink, #000); }
</style>
