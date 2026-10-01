<script setup lang="ts">
import type { Status } from '@/mock/db'
import Chip from './Chip.vue'

defineProps<{ status: Status }>()
</script>

<template>
  <Chip :label="status" :dark="status === 'Done'" />
</template>
