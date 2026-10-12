<script setup lang="ts">
import Label from './Label.vue'

defineProps<{ label?: string; hint?: string; error?: string; required?: boolean; htmlFor?: string }>()
</script>

<template>
  <div class="flex flex-col gap-1.5 w-full">
    <Label v-if="label" :html-for="htmlFor" :required="required">{{ label }}</Label>
    <slot />
    <p v-if="error" class="m-0 text-xs text-ink font-semibold" role="alert">{{ error }}</p>
    <p v-else-if="hint" class="m-0 text-xs text-muted">{{ hint }}</p>
  </div>
</template>
