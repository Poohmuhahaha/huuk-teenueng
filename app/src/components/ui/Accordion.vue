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
  <div class="border border-line w-full rounded-none">
    <div v-for="it in items" :key="it.id" class="border-t border-line first:border-t-0">
      <button
        type="button"
        class="flex items-center justify-between w-full bg-surface border-0 text-ink font-sans text-sm font-semibold p-3 px-3.5 cursor-pointer text-left transition-colors hover:bg-wash rounded-none"
        :aria-expanded="isOpen(it.id)"
        @click="toggle(it.id)"
      >
        <span>{{ it.title }}</span>
        <span
          class="transition-transform duration-150 text-muted text-[13px] inline-block"
          :class="{ 'rotate-45': isOpen(it.id) }"
          aria-hidden="true"
        >＋</span>
      </button>
      <div v-show="isOpen(it.id)" class="px-3.5 pb-3.5 text-[13px] text-muted">
        <slot :item="it" />
      </div>
    </div>
  </div>
</template>
