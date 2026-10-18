<script setup lang="ts">
// Card footer shared by every hub card: the page drops its primary
// controls in the slot and they close the card in normal flow (see the
// footer mechanics in style.css) — never hovering over content.
defineProps<{ label: string }>()
</script>

<template>
  <aside class="dock">
    <div class="dock-bar" role="group" :aria-label="label"><slot /></div>
  </aside>
</template>
