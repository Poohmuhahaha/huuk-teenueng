<script setup lang="ts">
defineProps<{ prefix?: string; suffix?: string; disabled?: boolean }>()
</script>

<template>
  <div
    class="flex items-stretch border border-line bg-surface w-full rounded-none transition-colors focus-within:border-ink [&_input]:border-0! [&_input]:bg-transparent! [&_input]:focus:outline-none [&_textarea]:border-0! [&_textarea]:bg-transparent! [&_select]:border-0! [&_select]:bg-transparent!"
    :class="{ 'opacity-50 cursor-not-allowed': disabled }"
  >
    <span v-if="prefix || $slots.prefix" class="inline-flex items-center px-3 bg-wash text-muted text-[13px] whitespace-nowrap border-r border-line"><slot name="prefix">{{ prefix }}</slot></span>
    <div class="flex-1 min-w-0 flex"><slot /></div>
    <span v-if="suffix || $slots.suffix" class="inline-flex items-center px-3 bg-wash text-muted text-[13px] whitespace-nowrap border-l border-line"><slot name="suffix">{{ suffix }}</slot></span>
  </div>
</template>
