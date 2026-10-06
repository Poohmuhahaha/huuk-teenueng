<script setup lang="ts">
// Compact circular icon button for toolbars and dialog footers.
// The label doubles as tooltip and accessible name — the slot holds the icon.
withDefaults(
  defineProps<{ label: string; tone?: 'default' | 'primary' | 'danger'; disabled?: boolean }>(),
  { tone: 'default' },
)
const emit = defineEmits<{ click: [MouseEvent] }>()
</script>

<template>
  <button
    type="button"
    class="iconbtn"
    :class="tone"
    :title="label"
    :aria-label="label"
    :disabled="disabled"
    @click="emit('click', $event)"
  >
    <slot />
  </button>
</template>

<style scoped>
.iconbtn {
  width: 36px;
  height: 36px;
  flex: 0 0 auto;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  padding: 0;
  border: 1px solid var(--line, #e4e4e4);
  border-radius: 50%;
  background: transparent;
  color: var(--muted, #6b6b6b);
  cursor: pointer;
  transition: background 0.15s, color 0.15s, border-color 0.15s;
}
.iconbtn:hover:not(:disabled) {
  background: var(--wash, #f6f6f6);
  color: var(--ink, #000);
  border-color: var(--ink, #000);
}
.iconbtn.primary {
  background: var(--ink, #000);
  border-color: var(--ink, #000);
  color: var(--paper, #fff);
}
.iconbtn.primary:hover:not(:disabled) {
  background: #222;
  border-color: #222;
}
.iconbtn.danger {
  color: var(--danger, #b91c1c);
  border-color: var(--danger, #b91c1c);
}
.iconbtn.danger:hover:not(:disabled) {
  background: var(--danger-wash, #fee2e2);
  color: var(--danger, #b91c1c);
  border-color: var(--danger, #b91c1c);
}
.iconbtn:disabled {
  opacity: 0.4;
  cursor: default;
}
</style>
