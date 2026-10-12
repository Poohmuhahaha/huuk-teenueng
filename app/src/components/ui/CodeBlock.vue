<script setup lang="ts">
// CodeBlock — monospace code surface with an optional language label.
defineProps<{ code?: string; lang?: string }>()
</script>

<template>
  <div class="relative w-full bg-wash border border-line p-3 overflow-x-auto rounded-none">
    <span v-if="lang" class="absolute top-2 right-2.5 text-[10px] tracking-[0.12em] uppercase text-faint">{{ lang }}</span>
    <pre class="m-0"><code class="font-mono text-xs text-ink"><slot>{{ code }}</slot></code></pre>
  </div>
</template>
