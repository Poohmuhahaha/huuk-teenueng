<script setup lang="ts">
const model = defineModel<string[]>({ default: () => [] })
const props = withDefaults(defineProps<{ items: { id: string; title: string }[]; multiple?: boolean }>(), {
  multiple: false,
})

function isOpen(id: string): boolean {
  return model.value.includes(id)
}

function toggle(id: string): void {
  if (props.multiple) {
    model.value = isOpen(id) ? model.value.filter((x) => x !== id) : [...model.value, id]
  } else {
    model.value = isOpen(id) ? [] : [id]
  }
}
</script>

<template>
  <div class="p-acc">
    <div v-for="it in items" :key="it.id" class="p-acc-item">
      <button
        type="button"
        class="p-acc-head"
        :aria-expanded="isOpen(it.id)"
        @click="toggle(it.id)"
      >
        <span>{{ it.title }}</span>
        <span class="p-acc-caret" :class="{ open: isOpen(it.id) }" aria-hidden="true">＋</span>
      </button>
      <div v-show="isOpen(it.id)" class="p-acc-body">
        <slot :item="it" />
      </div>
    </div>
  </div>
</template>

<style scoped>
.p-acc { border: 1px solid var(--line, #d9d9d9); width: 100%; }
.p-acc-item + .p-acc-item { border-top: 1px solid var(--line, #d9d9d9); }
.p-acc-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  width: 100%;
  background: var(--paper, #fff);
  border: 0;
  color: var(--ink, #000);
  font: inherit;
  font-size: 14px;
  font-weight: 600;
  padding: 12px 14px;
  cursor: pointer;
  text-align: left;
}
.p-acc-head:hover { background: var(--wash, #f6f6f6); }
.p-acc-caret { transition: transform 0.15s; color: var(--muted, #6b6b6b); font-size: 13px; }
.p-acc-caret.open { transform: rotate(45deg); }
.p-acc-body { padding: 0 14px 14px; font-size: 13px; color: var(--muted, #6b6b6b); }
</style>
