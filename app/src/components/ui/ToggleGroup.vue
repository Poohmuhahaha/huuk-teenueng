<script setup lang="ts">
const model = defineModel<string>()
defineProps<{ options: { value: string; label: string; disabled?: boolean }[] }>()
</script>

<template>
  <div class="p-togglegroup" role="group">
    <button
      v-for="o in options"
      :key="o.value"
      type="button"
      class="p-toggle"
      :class="{ on: model === o.value }"
      :aria-pressed="model === o.value"
      :disabled="o.disabled"
      @click="model = o.value"
    >
      {{ o.label }}
    </button>
  </div>
</template>

<style scoped>
.p-togglegroup { display: inline-flex; width: auto; }
.p-togglegroup .p-toggle {
  width: auto;
  display: inline-flex;
  align-items: center;
  background: transparent;
  border: 1px solid var(--line, #d9d9d9);
  color: var(--muted, #6b6b6b);
  font: inherit;
  font-size: 13px;
  font-weight: 600;
  padding: 6px 12px;
  cursor: pointer;
}
.p-togglegroup .p-toggle + .p-toggle { margin-left: -1px; }
.p-togglegroup .p-toggle:hover { color: var(--ink, #000); }
.p-togglegroup .p-toggle.on { background: var(--ink, #000); color: var(--paper, #fff); border-color: var(--ink, #000); z-index: 1; }
.p-togglegroup .p-toggle:disabled { opacity: 0.4; cursor: not-allowed; }
</style>
