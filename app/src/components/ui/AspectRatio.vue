<script setup lang="ts">
withDefaults(defineProps<{ ratio?: number }>(), { ratio: 16 / 9 })
</script>

<template>
  <div
    class="w-full overflow-hidden bg-wash [&_img]:w-full [&_img]:h-full [&_img]:object-cover [&_img]:block [&_video]:w-full [&_video]:h-full [&_video]:object-cover [&_video]:block [&_canvas]:w-full [&_canvas]:h-full [&_canvas]:object-cover [&_canvas]:block"
    :style="{ aspectRatio: String(ratio) }"
  >
    <slot />
  </div>
</template>
