<script setup lang="ts">
// CodeBlock — monospace code surface with an optional language label.
defineProps<{ code?: string; lang?: string }>()
</script>

<template>
  <div class="p-code">
    <span v-if="lang" class="p-code-lang">{{ lang }}</span>
    <pre><code><slot>{{ code }}</slot></code></pre>
  </div>
</template>

<style scoped>
.p-code {
  position: relative;
  width: 100%;
  background: var(--wash, #f6f6f6);
  border: 1px solid var(--line, #e4e4e4);
  padding: 12px;
  overflow-x: auto;
}
.p-code pre { margin: 0; }
.p-code code { font-family: var(--mono, ui-monospace, Menlo, monospace); font-size: 12px; color: var(--ink, #000); }
.p-code-lang {
  position: absolute;
  top: 8px;
  right: 10px;
  font-size: 10px;
  letter-spacing: 0.12em;
  text-transform: uppercase;
  color: var(--faint, #a3a3a3);
}
</style>
