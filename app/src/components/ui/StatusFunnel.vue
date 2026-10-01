<script setup lang="ts">
import type { Status } from '@/mock/db'
import Chip from './Chip.vue'

defineProps<{ current: Status }>()
const emit = defineEmits<{ advance: [s: Status] }>()
const order: Status[] = ['Start', 'Design', 'Dev', 'Done']
</script>

<template>
  <span>
    <Chip
      v-for="s in order" :key="s" :label="s" :dark="s === current"
      role="button" tabindex="0" :aria-pressed="s === current" style="cursor: pointer;"
      @click="emit('advance', s)"
      @keydown.enter.prevent="emit('advance', s)" @keydown.space.prevent="emit('advance', s)"
    />
  </span>
</template>
