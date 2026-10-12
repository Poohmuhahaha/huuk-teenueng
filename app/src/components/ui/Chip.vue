<script setup lang="ts">
// Presentational pill only — interactive callers own button/tabindex semantics.
// Contract: label must be non-empty (no v-if guard, API stays label+dark);
// long labels truncate with a native tooltip via :title.
defineProps<{ label: string; dark?: boolean }>()
</script>

<template>
  <span :title="label" class="inline-block rounded-full my-[2px] mr-1 py-[2px] px-2.5 text-[12.5px] border max-w-full overflow-hidden text-ellipsis whitespace-nowrap" :class="dark ? 'bg-accent-wash text-accent border-accent-line font-[650]' : 'bg-surface text-muted border-faint font-[550]'">{{ label }}</span>
</template>
