<script setup lang="ts">
defineProps<{
  items: {
    id: string
    label: string
    href?: string
    children?: { id: string; label: string; href?: string; description?: string }[]
  }[]
  active?: string
}>()
const emit = defineEmits<{ select: [string] }>()
</script>

<template>
  <nav class="p-navmenu">
    <div v-for="it in items" :key="it.id" class="p-navmenu-item">
      <a
        v-if="it.href && !it.children"
        class="p-navmenu-link"
        :class="{ active: active === it.id }"
        :href="it.href"
        @click="emit('select', it.id)"
      >
        {{ it.label }}
      </a>
      <button v-else type="button" class="p-navmenu-link" :class="{ active: active === it.id }">
        {{ it.label }} <span aria-hidden="true">▾</span>
      </button>
      <div v-if="it.children" class="p-navmenu-panel">
        <a
          v-for="child in it.children"
          :key="child.id"
          class="p-navmenu-child"
          :href="child.href"
          @click="emit('select', child.id)"
        >
          <strong>{{ child.label }}</strong>
          <span v-if="child.description" class="p-navmenu-desc">{{ child.description }}</span>
        </a>
      </div>
    </div>
  </nav>
</template>

<style scoped>
.p-navmenu { display: flex; align-items: center; gap: 4px; }
.p-navmenu-item { position: relative; }
.p-navmenu-link {
  display: inline-flex;
  align-items: center;
  gap: 5px;
  width: auto;
  background: transparent;
  border: 0;
  color: var(--muted, #6b6b6b);
  font: inherit;
  font-size: 13px;
  font-weight: 600;
  padding: 8px 12px;
  cursor: pointer;
  text-decoration: none;
}
.p-navmenu-link:hover { color: var(--ink, #000); }
.p-navmenu-link.active { color: var(--ink, #000); text-decoration: underline; text-underline-offset: 4px; }
.p-navmenu-panel {
  position: absolute;
  z-index: 60;
  top: 100%;
  left: 0;
  min-width: 240px;
  background: var(--paper, #fff);
  border: 1px solid var(--ink, #000);
  padding: 6px;
  box-shadow: 4px 4px 0 var(--ink, #000);
  display: none;
  flex-direction: column;
}
.p-navmenu-item:hover .p-navmenu-panel,
.p-navmenu-item:focus-within .p-navmenu-panel { display: flex; }
.p-navmenu-child {
  display: flex;
  flex-direction: column;
  gap: 2px;
  padding: 8px 10px;
  color: var(--ink, #000);
  font-size: 13px;
  text-decoration: none;
}
.p-navmenu-child:hover { background: var(--wash, #f6f6f6); }
.p-navmenu-desc { font-size: 11px; color: var(--muted, #6b6b6b); }
</style>
