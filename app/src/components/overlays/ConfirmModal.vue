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
  <div v-if="open" class="fixed inset-0 z-[1000] bg-black/40 flex items-start justify-center pt-[15vh] p-4" @click.self="emit('cancel')">
    <div class="w-[90%] max-w-[480px] bg-surface border border-ink shadow-[6px_6px_0_var(--color-ink)] rounded-none" role="dialog" aria-modal="true">
      <div class="flex items-center justify-between p-3.5 px-4.5 border-b border-ink">
        <h3 class="m-0 text-lg font-bold text-ink">{{ title }}</h3>
        <button type="button" class="bg-transparent border-0 text-[22px] leading-none cursor-pointer text-muted w-auto hover:text-ink" aria-label="Close" @click="emit('cancel')">×</button>
      </div>
      <div class="p-4.5">
        <p v-if="message" class="m-0 mb-2 text-sm text-ink">{{ message }}</p>
        <slot />
        <div class="flex justify-end gap-3 mt-4.5">
          <button type="button" class="w-auto px-4 py-2 border border-line bg-surface text-ink text-sm font-semibold rounded-none cursor-pointer transition-colors hover:bg-wash" @click="emit('cancel')">Cancel</button>
          <button
            type="button"
            class="w-auto px-4 py-2 border text-sm font-semibold rounded-none cursor-pointer transition-colors"
            :class="tone === 'danger' ? 'bg-ink text-surface border-ink hover:bg-neutral-800' : 'bg-ink text-surface border-ink hover:bg-neutral-800'"
            @click="emit('confirm')"
          >{{ confirmText }}</button>
        </div>
      </div>
    </div>
  </div>
</template>
