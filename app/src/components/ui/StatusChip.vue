<script setup lang="ts">
import { computed } from 'vue'
import type { Status } from '@/mock/db'
import Chip from './Chip.vue'

const props = defineProps<{ status: Status }>()
// 3-tone mapping: Done stays dark; Design/Dev get a bordered wash mid-tone via
// the wrapper span (child selector beats Chip's own tone classes); Start and
// unknown keep the default light tone.
const wip = computed(() => props.status === 'Design' || props.status === 'Dev')
</script>

<template>
  <span :class="wip ? '[&>span]:border-accent-line [&>span]:bg-accent-wash/20 [&>span]:text-accent' : ''"><Chip :label="status" :dark="status === 'Done'" /></span>
</template>
