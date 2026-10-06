<script setup lang="ts">
defineProps<{ label: string; value: string | number; hint?: string; to?: string }>()
</script>

<template>
  <RouterLink v-if="to" class="card kpi kpi-link" :to="to">
    <span class="kpi-label muted">{{ label }}</span>
    <span class="kpi-value">{{ value }}</span>
    <span v-if="hint" class="kpi-hint muted">{{ hint }}</span>
    <svg class="kpi-go" viewBox="0 0 16 16" width="14" height="14" fill="none" stroke="currentColor"
      stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
      <path d="M4 12 12 4M6 4h6v6" />
    </svg>
  </RouterLink>
  <div v-else class="card kpi">
    <span class="kpi-label muted">{{ label }}</span>
    <span class="kpi-value">{{ value }}</span>
    <span v-if="hint" class="kpi-hint muted">{{ hint }}</span>
  </div>
</template>

<style scoped>
.kpi {
  position: relative;
  display: flex;
  flex-direction: column;
  gap: 1px;
  padding: 10px 12px;
  text-align: left;
}
.kpi-label { font-size: 12px; line-height: 1.3; }
.kpi-value { font-size: 21px; font-weight: 800; letter-spacing: -0.01em; line-height: 1.2; }
.kpi-hint { font-size: 11.5px; line-height: 1.3; }
.kpi-link { text-decoration: none; color: inherit; transition: border-color 0.15s; }
.kpi-link:hover { border-color: var(--ink); }
.kpi-go {
  position: absolute;
  top: 10px;
  right: 10px;
  opacity: 0;
  transition: opacity 0.15s;
}
.kpi-link:hover .kpi-go,
.kpi-link:focus-visible .kpi-go { opacity: 0.5; }
</style>
