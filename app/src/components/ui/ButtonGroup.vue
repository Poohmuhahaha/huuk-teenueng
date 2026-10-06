<script setup lang="ts">
withDefaults(defineProps<{ orientation?: 'horizontal' | 'vertical'; attached?: boolean }>(), {
  orientation: 'horizontal',
  attached: true,
})
</script>

<template>
  <div class="p-btngroup" :class="[orientation, { attached }]">
    <slot />
  </div>
</template>

<style scoped>
.p-btngroup { display: inline-flex; width: auto; }
.p-btngroup.horizontal { flex-direction: row; }
.p-btngroup.vertical { flex-direction: column; align-items: stretch; }
.p-btngroup :deep(> *) { width: auto; }
.p-btngroup.horizontal.attached :deep(> * + *) { margin-left: -1px; }
.p-btngroup.vertical.attached :deep(> * + *) { margin-top: -1px; }
</style>
