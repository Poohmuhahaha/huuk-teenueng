<script setup lang="ts">
import Label from './Label.vue'

defineProps<{ label?: string; hint?: string; error?: string; required?: boolean; htmlFor?: string }>()
</script>

<template>
  <div class="p-field" :class="{ invalid: !!error }">
    <Label v-if="label" :html-for="htmlFor" :required="required">{{ label }}</Label>
    <slot />
    <p v-if="error" class="p-field-msg">{{ error }}</p>
    <p v-else-if="hint" class="p-field-hint">{{ hint }}</p>
  </div>
</template>

<style scoped>
.p-field { display: flex; flex-direction: column; gap: 6px; width: 100%; }
.p-field-msg { margin: 0; font-size: 12px; color: var(--ink, #000); font-weight: 600; }
.p-field-hint { margin: 0; font-size: 12px; color: var(--muted, #6b6b6b); }
</style>
