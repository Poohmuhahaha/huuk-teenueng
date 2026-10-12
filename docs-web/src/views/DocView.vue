<script setup>
import { computed, onMounted, ref, watch } from 'vue'
import { marked } from 'marked'
import { getPage, listPages } from '../api.js'

const props = defineProps({ slug: { type: String, required: true } })

const page = ref(null)
const pages = ref([])
const state = ref('loading') // loading | ready | missing | offline

marked.setOptions({ breaks: true })
const html = computed(() => (page.value ? marked.parse(page.value.body || '') : ''))

function neighbors() {
  const i = pages.value.findIndex((p) => p.slug === props.slug)
  return {
    prev: i > 0 ? pages.value[i - 1] : null,
    next: i >= 0 && i < pages.value.length - 1 ? pages.value[i + 1] : null,
  }
}

async function load() {
  state.value = 'loading'
  try {
    const [all, one] = await Promise.all([listPages(), getPage(props.slug)])
    pages.value = all
    page.value = one
    state.value = one ? 'ready' : 'missing'
  } catch {
    state.value = 'offline'
  }
}

onMounted(load)
watch(() => props.slug, load)
</script>

<template>
  <div v-if="state === 'loading'" class="state">กำลังโหลดจาก Strapi…</div>

  <div v-else-if="state === 'offline'" class="state">
    <h1>ต่อ Strapi ไม่ได้</h1>
    <p>เว็บนี้อ่าน content สดจาก HeadlessCMS — รัน Strapi ก่อน:</p>
    <pre>cd cms &amp;&amp; docker compose up --build   # :1337</pre>
    <p class="muted">ตรวจ API: <code>curl http://localhost:1337/api/doc-pages</code></p>
  </div>

  <div v-else-if="state === 'missing'" class="state">
    <h1>ไม่พบหน้านี้ ({{ $route.params.slug }})</h1>
    <RouterLink to="/index">← กลับหน้าแรก</RouterLink>
  </div>

  <article v-else class="doc">
    <h1>{{ page.title }}</h1>
    <!-- content มาจาก CMS ของทีมเอง (trusted) จึง render ตรง -->
    <div class="markdown" v-html="html" />
    <nav class="pager">
      <RouterLink v-if="neighbors().prev" :to="`/${neighbors().prev.slug}`">
        ← {{ neighbors().prev.title }}
      </RouterLink>
      <span v-else />
      <RouterLink v-if="neighbors().next" :to="`/${neighbors().next.slug}`">
        {{ neighbors().next.title }} →
      </RouterLink>
    </nav>
  </article>
</template>
