<script setup lang="ts">
// Alert — inline message box. Ported from Pugu (ui/Alert) to Huuk design
// (monochrome, hairline, square).
withDefaults(defineProps<{ title?: string; tone?: 'info' | 'success' | 'warning' | 'danger' }>(), {
  tone: 'info',
})
</script>

<template>
  <div class="p-alert" :class="`t-${tone}`" role="alert">
    <strong v-if="title" class="p-alert-title">{{ title }}</strong>
    <slot />
  </div>
</template>

<style scoped>
.p-alert {
  border: 1px solid var(--line, #e4e4e4);
  border-left-width: 4px;
  padding: 12px 14px;
  font-size: 13px;
  color: var(--ink, #000);
  width: 100%;
}
.p-alert-title { display: block; margin-bottom: 2px; }
.t-info { border-left-color: var(--ink, #000); }
.t-success { border-left-style: double; border-left-color: var(--ink, #000); }
.t-warning { border-left-style: dashed; border-left-color: var(--ink, #000); }
.t-danger { border-left-color: var(--ink, #000); background: var(--wash, #f6f6f6); }
</style>
