<script setup lang="ts">
const model = defineModel<string>()
withDefaults(
  defineProps<{ options?: { value: string; label: string; disabled?: boolean }[]; name?: string; direction?: 'row' | 'column' }>(),
  { options: () => [], name: 'p-radio', direction: 'column' },
)
</script>

<template>
  <div class="p-radios" :class="direction" role="radiogroup">
    <label v-for="o in options" :key="o.value" class="p-radio" :class="{ disabled: o.disabled }">
      <input
        type="radio"
        :name="name"
        :value="o.value"
        :checked="model === o.value"
        :disabled="o.disabled"
        @change="model = o.value"
      />
      <span class="p-radio-mark" aria-hidden="true" />
      <span>{{ o.label }}</span>
    </label>
  </div>
</template>

<style scoped>
.p-radios { display: flex; gap: 10px; }
.p-radios.column { flex-direction: column; }
.p-radios.row { flex-direction: row; flex-wrap: wrap; gap: 16px; }
.p-radio { display: inline-flex; align-items: center; gap: 8px; cursor: pointer; font-size: 14px; color: var(--ink, #000); }
.p-radio.disabled { opacity: 0.5; cursor: not-allowed; }
.p-radio input { position: absolute; opacity: 0; width: 1px; height: 1px; }
.p-radio-mark {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 18px;
  height: 18px;
  border: 1px solid var(--line, #d9d9d9);
  border-radius: 999px;
  background: var(--paper, #fff);
  flex: none;
}
.p-radio-mark::after { content: ''; width: 8px; height: 8px; border-radius: 999px; background: transparent; }
.p-radio input:checked + .p-radio-mark { border-color: var(--ink, #000); }
.p-radio input:checked + .p-radio-mark::after { background: var(--ink, #000); }
.p-radio input:focus-visible + .p-radio-mark { outline: 2px solid var(--ink, #000); outline-offset: 2px; }
</style>
