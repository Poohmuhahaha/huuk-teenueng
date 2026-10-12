<script setup lang="ts">
// Avatar — square initials chip (ported from Pugu ui/Avatar).
defineProps<{ label?: string; more?: boolean }>()
</script>

<template>
  <span
    class="inline-flex items-center justify-center w-[30px] h-[30px] border border-line text-xs font-semibold shrink-0 rounded-none"
    :class="more ? 'bg-ink text-surface border-ink' : 'bg-surface text-ink'"
  ><slot>{{ label }}</slot></span>
</template>
