<script setup lang="ts">
defineProps<{ label: string; dark?: boolean }>()
</script>

<template>
  <span class="chip" :class="{ dark: !!dark }">{{ label }}</span>
</template>
