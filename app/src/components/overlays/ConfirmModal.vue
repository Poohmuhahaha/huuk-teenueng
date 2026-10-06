<script setup lang="ts">
// ConfirmModal — destructive / primary confirmation dialog.
// Ported from Pugu ConfirmModal to Huuk chrome (square, ink, hard-step shadow).
withDefaults(
  defineProps<{
    open: boolean
    title: string
    message?: string
    confirmText?: string
    tone?: 'danger' | 'primary'
  }>(),
  { message: '', confirmText: 'Confirm', tone: 'danger' },
)
const emit = defineEmits<{ cancel: []; confirm: [] }>()
</script>

<template>
  <div v-if="open" class="p-backdrop" @click.self="emit('cancel')">
    <div class="p-modal" role="dialog" aria-modal="true" :class="`t-${tone}`">
      <div class="p-head">
        <h3>{{ title }}</h3>
        <button type="button" class="p-x" aria-label="Close" @click="emit('cancel')">×</button>
      </div>
      <div class="p-body">
        <p v-if="message" class="p-msg">{{ message }}</p>
        <slot />
        <div class="p-actions">
          <button type="button" class="btn" @click="emit('cancel')">Cancel</button>
          <button type="button" class="btn btn-primary" @click="emit('confirm')">{{ confirmText }}</button>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.p-backdrop {
  position: fixed;
  inset: 0;
  z-index: 1000;
  background: rgba(0, 0, 0, 0.4);
  display: flex;
  align-items: flex-start;
  justify-content: center;
  padding-top: 15vh;
}
.p-modal {
  width: 90%;
  max-width: 480px;
  background: var(--paper, #fff);
  border: 1px solid var(--ink, #000);
  box-shadow: 6px 6px 0 var(--ink, #000);
}
.p-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 14px 18px;
  border-bottom: 1px solid var(--ink, #000);
}
.p-head h3 { margin: 0; font-size: 18px; }
.p-x { background: transparent; border: 0; font-size: 22px; line-height: 1; cursor: pointer; color: var(--muted, #6b6b6b); width: auto; }
.p-x:hover { color: var(--ink, #000); }
.p-body { padding: 18px; }
.p-msg { margin: 0 0 8px; font-size: 14px; color: var(--ink, #000); }
.p-actions { display: flex; justify-content: flex-end; gap: 12px; margin-top: 18px; }
.t-danger { border-color: var(--ink, #000); }
</style>
