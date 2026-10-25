<script setup lang="ts">
import { useCountUp } from '@/composables/useCountUp'

const props = withDefaults(
  defineProps<{
    value: string | number
    duration?: number
    delay?: number
  }>(),
  {
    duration: 950,
    delay: 0,
  },
)

const display = useCountUp(() => props.value, {
  duration: props.duration,
  delay: props.delay,
})
</script>

<template>
  <span class="countup-number tabular tabular-nums">{{ display }}</span>
</template>
