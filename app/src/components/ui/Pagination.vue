<script setup lang="ts">
// Pagination — page info + Prev/Next (ported from Pugu Pagination).
import { computed } from 'vue'

const props = defineProps<{ currentPage: number; totalPages: number }>()
const emit = defineEmits<{ 'update:currentPage': [number] }>()
const visible = computed(() => props.totalPages > 1)
function go(page: number): void {
  if (page < 1 || page > props.totalPages || page === props.currentPage) return
  emit('update:currentPage', page)
}
</script>

<template>
  <div v-if="visible" class="flex items-center justify-between gap-4 mt-5 py-3.5 border-t border-line w-full">
    <span class="text-[13px] text-muted">Page <strong class="text-ink">{{ currentPage }}</strong> of <strong class="text-ink">{{ totalPages }}</strong></span>
    <div class="flex gap-2.5">
      <button
        type="button"
        class="w-auto px-3.5 py-2 border border-line bg-surface text-ink text-sm font-semibold rounded-none cursor-pointer transition-colors hover:bg-wash disabled:opacity-50 disabled:cursor-not-allowed"
        :disabled="currentPage <= 1"
        @click="go(currentPage - 1)"
      >‹ Prev</button>
      <button
        type="button"
        class="w-auto px-3.5 py-2 border border-line bg-surface text-ink text-sm font-semibold rounded-none cursor-pointer transition-colors hover:bg-wash disabled:opacity-50 disabled:cursor-not-allowed"
        :disabled="currentPage >= totalPages"
        @click="go(currentPage + 1)"
      >Next ›</button>
    </div>
  </div>
</template>
