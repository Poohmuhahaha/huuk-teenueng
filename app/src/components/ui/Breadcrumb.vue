<script setup lang="ts">
defineProps<{ items: { label: string; href?: string }[] }>()
</script>

<template>
  <nav class="p-crumbs" aria-label="Breadcrumb">
    <template v-for="(c, i) in items" :key="`${c.label}-${i}`">
      <a v-if="c.href" class="p-crumb-link" :href="c.href">{{ c.label }}</a>
      <span v-else class="p-crumb-current" aria-current="page">{{ c.label }}</span>
      <span v-if="i < items.length - 1" class="p-crumb-sep" aria-hidden="true">/</span>
    </template>
  </nav>
</template>

<style scoped>
.p-crumbs { display: flex; align-items: center; gap: 8px; flex-wrap: wrap; font-size: 13px; }
.p-crumb-link { color: var(--muted, #6b6b6b); text-decoration: none; }
.p-crumb-link:hover { color: var(--ink, #000); text-decoration: underline; text-underline-offset: 3px; }
.p-crumb-current { color: var(--ink, #000); font-weight: 600; }
.p-crumb-sep { color: var(--faint, #a3a3a3); }
</style>
