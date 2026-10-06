<script setup lang="ts">
const model = defineModel<boolean>({ default: false })
defineProps<{ label?: string; disabled?: boolean }>()
</script>

<template>
  <label class="p-switchrow" :class="{ disabled }">
    <button
      type="button"
      class="p-switch"
      :class="{ on: model }"
      role="switch"
      :aria-checked="model"
      :disabled="disabled"
      @click="model = !model"
    >
      <span class="p-switch-knob" />
    </button>
    <span v-if="label || $slots.default" class="p-switch-label"><slot>{{ label }}</slot></span>
  </label>
</template>

<style scoped>
.p-switchrow { display: inline-flex; align-items: center; gap: 10px; cursor: pointer; font-size: 14px; color: var(--ink, #000); }
.p-switchrow.disabled { opacity: 0.5; cursor: not-allowed; }
.p-switch {
  width: 38px;
  height: 22px;
  flex: none;
  border: 1px solid var(--ink, #000);
  background: var(--paper, #fff);
  padding: 2px;
  display: inline-flex;
  align-items: center;
  transition: background 0.15s;
}
.p-switch.on { background: var(--ink, #000); }
.p-switch-knob { width: 14px; height: 14px; background: var(--ink, #000); transition: transform 0.15s, background 0.15s; }
.p-switch.on .p-switch-knob { background: var(--paper, #fff); transform: translateX(16px); }
.p-switch:focus-visible { outline: 2px solid var(--ink, #000); outline-offset: 2px; }
</style>
