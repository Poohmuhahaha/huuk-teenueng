<script setup lang="ts">
// Shared media card base: cover block + title + body slot + meta line.
// Used by the responsive card grids (feed cells keep their own square style).
defineProps<{ title: string; meta?: string; coverLabel?: string }>()
</script>

<template>
  <div class="border border-line rounded-none overflow-hidden cursor-pointer bg-surface shadow-sm transition-all duration-150 hover:shadow-md hover:-translate-y-px hover:border-ink">
    <div v-if="coverLabel !== undefined" class="h-[90px] bg-surface-2 border-b border-line flex items-center justify-center text-2xl font-bold text-muted">{{ coverLabel }}</div>
    <div class="p-3 pb-3.5 text-sm">
      <strong class="text-ink font-semibold block">{{ title }}</strong>
      <div class="text-muted text-[13px]"><slot /></div>
      <div v-if="meta" class="text-muted text-[13px] mt-0.5">{{ meta }}</div>
    </div>
  </div>
</template>
