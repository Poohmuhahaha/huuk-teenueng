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
  <div v-if="visible" class="p-pagination">
    <span class="p-pageinfo">Page <strong>{{ currentPage }}</strong> of <strong>{{ totalPages }}</strong></span>
    <div class="p-pagebtns">
      <button type="button" class="btn" :disabled="currentPage <= 1" @click="go(currentPage - 1)">‹ Prev</button>
      <button type="button" class="btn" :disabled="currentPage >= totalPages" @click="go(currentPage + 1)">Next ›</button>
    </div>
  </div>
</template>

<style scoped>
.p-pagination {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 16px;
  margin-top: 20px;
  padding: 14px 0;
  border-top: 1px solid var(--line, #e4e4e4);
}
.p-pageinfo { font-size: 13px; color: var(--muted, #6b6b6b); }
.p-pageinfo strong { color: var(--ink, #000); }
.p-pagebtns { display: flex; gap: 10px; }
.p-pagebtns .btn { width: auto; }
</style>
