<script setup lang="ts">
// Shared media card base: cover block + title + body slot + meta line.
// Used by the responsive card grids (feed cells keep their own square style).
defineProps<{ title: string; meta?: string; coverLabel?: string }>()
</script>

<template>
  <div class="pcard">
    <div v-if="coverLabel !== undefined" class="cover">{{ coverLabel }}</div>
    <div class="body">
      <strong>{{ title }}</strong>
      <div class="muted"><slot /></div>
      <div v-if="meta" class="muted">{{ meta }}</div>
    </div>
  </div>
</template>
