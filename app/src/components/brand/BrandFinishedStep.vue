<script setup lang="ts">
import type { Brand } from '@/mock/db'

defineProps<{
  brand: Brand
}>()

const emit = defineEmits<{
  (e: 'export-pdf'): void
  (e: 'restart-step'): void
}>()
</script>

<template>
  <section class="brand-body brand-finished-hero">
    <div class="brand-finished-card">
      <span class="brand-finished-badge">Brand Guidelines Complete</span>
      <div class="brand-finished-name">{{ brand.channel || 'Studio Channel' }}</div>
      <p class="brand-finished-desc">All 7 chapters configured and ready for presentation.</p>
      <div class="brand-finished-actions">
        <button type="button" class="btn btn-primary btn-finished-export" @click="emit('export-pdf')">
          Export PDF
        </button>
        <button type="button" class="btn btn-finished-review" @click="emit('restart-step')">
          Review Chapters
        </button>
      </div>
    </div>
  </section>
</template>
