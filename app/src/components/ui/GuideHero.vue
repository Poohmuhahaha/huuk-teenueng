<script setup lang="ts">
const steps = [
  { n: '1 · Read', d: 'guide + tutorial video' },
  { n: '2 · Configure', d: 'Set up sheet' },
  { n: '3 · Plan', d: 'pick a month tab' },
]
const warnings = [
  'Never insert/delete rows or columns — it breaks formulas.',
  'Never type over formula cells — choose from dropdowns only.',
  'Paste values only (Ctrl+Shift+V) into Caption/CTA/Hashtags.',
]
</script>

<template>
  <img class="heroimg" src="/system-map.png" alt="Huuk system map" />
  <div class="steps mt">
    <div v-for="s in steps" :key="s.n" class="card" style="text-align: center;">
      <strong>{{ s.n }}</strong>
      <div class="muted">{{ s.d }}</div>
    </div>
  </div>
  <div class="card warnbox">
    <strong>Warnings before you start</strong>
    <ul style="margin: 6px 0 0; padding-left: 18px;">
      <li v-for="w in warnings" :key="w">{{ w }}</li>
    </ul>
  </div>
</template>
