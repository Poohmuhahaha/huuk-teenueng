<script setup lang="ts">
defineProps<{ size?: 'sm' | 'md' }>()
</script>

<template>
  <kbd
    class="inline-flex items-center justify-center min-w-5 border border-b-2 border-line bg-wash text-ink font-mono text-[11px] px-1.5 py-0.5 rounded-none"
    :class="{ 'text-[10px] px-1 py-[1px]': size === 'sm' }"
  >
    <slot />
  </kbd>
</template>
