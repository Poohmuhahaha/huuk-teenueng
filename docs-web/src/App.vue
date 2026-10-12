<script setup>
import { computed, onMounted, ref } from 'vue'
import { useRoute } from 'vue-router'
import { STRAPI_URL, listPages } from './api.js'

const route = useRoute()
const pages = ref([])
const query = ref('')
const failed = ref(false)

const groups = computed(() => {
  const q = query.value.trim().toLowerCase()
  const rows = q
    ? pages.value.filter(
        (p) =>
          p.title.toLowerCase().includes(q) ||
          (p.summary || '').toLowerCase().includes(q),
      )
    : pages.value
  const map = new Map()
  for (const p of rows) {
    const key = p.sectionLabel || p.section
    if (!map.has(key)) map.set(key, [])
    map.get(key).push(p)
  }
  return [...map.entries()]
})

onMounted(async () => {
  try {
    pages.value = await listPages()
  } catch {
    failed.value = true
  }
})
</script>

<template>
  <div class="shell">
    <aside class="side">
      <RouterLink to="/index" class="brand">
        <strong>Huuk Docs</strong>
        <span class="live" :title="STRAPI_URL">● live</span>
      </RouterLink>
      <input v-model="query" class="search" type="search" placeholder="ค้นหา…" aria-label="ค้นหา docs" />
      <p v-if="failed" class="warn">
        ต่อ Strapi ไม่ได้ — รัน <code>cd cms &amp;&amp; docker compose up --build</code>
      </p>
      <nav v-for="[label, rows] in groups" :key="label" class="group">
        <p class="group-label">{{ label }}</p>
        <RouterLink
          v-for="p in rows"
          :key="p.slug"
          :to="`/${p.slug}`"
          class="link"
          :class="{ active: route.params.slug === p.slug }"
          :title="p.summary"
        >
          {{ p.title }}
        </RouterLink>
      </nav>
    </aside>
    <main class="main">
      <RouterView :key="route.params.slug" />
    </main>
  </div>
</template>
