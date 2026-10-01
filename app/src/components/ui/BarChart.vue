<script setup lang="ts">
import { computed } from 'vue'

const props = defineProps<{ items: { label: string; value: number }[] }>()

const max = computed(() => Math.max(1, ...props.items.map((x) => x.value)))

function height(value: number): string {
  return `${Math.max(4, Math.round((value / max.value) * 100))}%`
}
</script>

<template>
  <div class="card">
    <div v-if="!items.length" class="muted">No data yet.</div>
    <template v-else>
      <div class="bars">
        <div v-for="(it, i) in items" :key="`${it.label}-${i}`" class="bar" :style="{ height: height(it.value) }"
          :title="`${it.label}: ${it.value}`" />
      </div>
      <div class="barlabels">
        <span v-for="(it, i) in items" :key="`${it.label}-${i}`">{{ it.label }} ({{ it.value }})</span>
      </div>
    </template>
  </div>
</template>
