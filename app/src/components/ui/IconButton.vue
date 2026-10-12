<script setup lang="ts">
// Compact circular icon button for toolbars and dialog footers.
// The label doubles as tooltip and accessible name — the slot holds the icon.
// Sizing exception: fixed w-9 h-9 is intentional (circular icon target),
// overriding the every-<button>-gets-w-full/w-auto rule (flex-none beats base).
withDefaults(
  defineProps<{ label: string; tone?: 'default' | 'primary' | 'danger'; disabled?: boolean }>(),
  { tone: 'default' },
)
const emit = defineEmits<{ click: [MouseEvent] }>()
</script>

<template>
  <button
    type="button"
    class="w-9 h-9 flex-none inline-flex items-center justify-center p-0 rounded-full border bg-transparent cursor-pointer transition-colors disabled:opacity-40 disabled:cursor-default hover:bg-wash hover:text-ink hover:border-ink focus-visible:ring-2 focus-visible:border-accent"
    :class="tone === 'primary' ? 'bg-ink border-ink text-white hover:bg-accent-hover hover:border-accent-hover hover:text-white' : tone === 'danger' ? 'text-danger border-danger hover:bg-danger-wash hover:text-danger hover:border-danger' : 'border-line text-muted'"
    :title="label"
    :aria-label="label"
    :disabled="disabled"
    @click="emit('click', $event)"
  >
    <slot />
  </button>
</template>
