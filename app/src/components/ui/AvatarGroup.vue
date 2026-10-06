<script setup lang="ts">
// AvatarGroup — overlapping avatars with a +N overflow chip.
import { computed } from 'vue'
import Avatar from './Avatar.vue'

const props = withDefaults(defineProps<{ items?: string[]; max?: number }>(), { items: () => [], max: 3 })
const shown = computed(() => props.items.slice(0, props.max))
const extra = computed(() => Math.max(0, props.items.length - shown.value.length))
</script>

<template>
  <div class="p-avatars">
    <Avatar v-for="label in shown" :key="label" :label="label" />
    <Avatar v-if="extra > 0" :label="`+${extra}`" more />
  </div>
</template>

<style scoped>
.p-avatars { display: flex; }
.p-avatars :deep(.p-avatar + .p-avatar) { margin-left: -8px; }
</style>
