<script setup lang="ts">
import type { Status } from '@/mock/db'
import Chip from './Chip.vue'

defineProps<{ status: Status }>()
</script>

<template>
  <span><Chip :label="status" dark /></span>
</template>
