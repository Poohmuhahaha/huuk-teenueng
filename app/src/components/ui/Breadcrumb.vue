<script setup lang="ts">
defineProps<{ items: { label: string; href?: string }[] }>()
</script>

<template>
  <nav class="flex items-center gap-2 flex-wrap text-sm" aria-label="Breadcrumb">
    <template v-for="(c, i) in items" :key="`${c.label}-${i}`">
      <a v-if="c.href" class="text-muted no-underline hover:text-ink hover:underline hover:underline-offset-[3px]" :href="c.href">{{ c.label }}</a>
      <span v-else class="text-ink font-semibold" aria-current="page">{{ c.label }}</span>
      <span v-if="i < items.length - 1" class="text-faint" aria-hidden="true">/</span>
    </template>
  </nav>
</template>
