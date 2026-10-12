<script setup lang="ts">
const open = defineModel<boolean>({ default: false })
defineProps<{ title?: string }>()
</script>

<template>
  <div class="w-full">
    <button
      type="button"
      class="flex items-center gap-2 w-auto bg-transparent border-0 text-ink font-sans text-sm font-semibold py-1 px-0 cursor-pointer text-left rounded-none"
      :aria-expanded="open"
      @click="open = !open"
    >
      <slot name="trigger">{{ title }}</slot>
      <span
        class="text-muted transition-transform duration-150 inline-block"
        :class="{ 'rotate-90': open }"
        aria-hidden="true"
      >▸</span>
    </button>
    <div v-show="open" class="pt-2 pl-5 text-[13px] text-muted">
      <slot />
    </div>
  </div>
</template>
