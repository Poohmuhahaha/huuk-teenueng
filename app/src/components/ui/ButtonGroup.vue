<script setup lang="ts">
withDefaults(defineProps<{ orientation?: 'horizontal' | 'vertical'; attached?: boolean }>(), {
  orientation: 'horizontal',
  attached: true,
})
</script>

<template>
  <div
    class="inline-flex w-auto [&>*]:w-auto"
    :class="[
      orientation === 'vertical' ? 'flex-col items-stretch' : 'flex-row',
      attached && orientation === 'vertical' ? '[&>*+*]:-mt-px' : '',
      attached && orientation !== 'vertical' ? '[&>*+*]:-ml-px' : ''
    ]"
  >
    <slot />
  </div>
</template>
