<script setup lang="ts">
// Shared table chrome: bordered scrolling frame + table. Header/cell look
// comes from the .tbl rules (single source), so plain <thead>/<tbody>
// markup in the slot renders exactly like the TanStack tables.
</script>

<template>
  <div class="tblwrap">
    <table class="tbl"><slot /></table>
  </div>
</template>
