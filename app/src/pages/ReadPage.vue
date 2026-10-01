<script setup lang="ts">
// Public delivery view: renders published CMS content at /#/read/{slug}.
// Without a slug it lists everything that is live. Safe HTML comes from
// `renderMarkdown` (escapes all text before inserting tags).
import { computed } from 'vue'
import { useRoute } from 'vue-router'
import { usePublicContent, usePublicContentList } from '@/core/queries'
import { renderMarkdown } from '@/core/markdown'
import { t } from '@/core/i18n'

const route = useRoute()
const slug = computed(() => (typeof route.params.slug === 'string' ? route.params.slug : ''))
const listQ = usePublicContentList()
const itemQ = usePublicContent(slug)

const bodyHtml = computed(() => renderMarkdown(itemQ.data.value?.body ?? ''))
const notFound = computed(() => itemQ.isError.value)

function formatStamp(stamp: string | null | undefined): string {
  if (!stamp) return ''
  const date = new Date(stamp)
  return Number.isNaN(date.getTime())
    ? stamp
    : date.toLocaleDateString(undefined, { year: 'numeric', month: 'long', day: 'numeric' })
}
</script>

<template>
  <div class="read">
    <header class="read-head">
      <RouterLink class="read-brand" to="/">Huuk <span class="muted">· Studio</span></RouterLink>
      <nav>
        <RouterLink class="btn" to="/read">{{ t('read.all') }}</RouterLink>
        <RouterLink class="btn" to="/">{{ t('read.back') }}</RouterLink>
      </nav>
    </header>

    <!-- single article -->
    <template v-if="slug">
      <div v-if="itemQ.isPending.value" class="muted">{{ t('common.loading') }}</div>
      <div v-else-if="notFound" class="card">
        <h1>{{ t('read.notFound') }}</h1>
        <p class="muted">{{ t('read.notFoundHint') }}</p>
        <RouterLink class="btn" to="/read">{{ t('read.all') }}</RouterLink>
      </div>
      <article v-else-if="itemQ.data.value" class="read-article">
        <p class="muted read-meta">
          {{ itemQ.data.value.author }}
          <template v-if="itemQ.data.value.publishedAt"> · {{ formatStamp(itemQ.data.value.publishedAt) }}</template>
        </p>
        <h1>{{ itemQ.data.value.title }}</h1>
        <p v-if="itemQ.data.value.excerpt" class="read-excerpt">{{ itemQ.data.value.excerpt }}</p>
        <img v-if="itemQ.data.value.heroImageUrl" :src="itemQ.data.value.heroImageUrl"
          :alt="itemQ.data.value.title" class="read-hero" />
        <!-- sanitized by renderMarkdown -->
        <div class="markdown" v-html="bodyHtml" />
        <p v-if="itemQ.data.value.tags.length" class="read-tags">
          <span v-for="tag in itemQ.data.value.tags" :key="tag" class="chip">#{{ tag }}</span>
        </p>
      </article>
    </template>

    <!-- index -->
    <template v-else>
      <h1>{{ t('read.published') }}</h1>
      <div v-if="listQ.isPending.value" class="muted">{{ t('common.loading') }}</div>
      <p v-else-if="!listQ.data.value?.length" class="muted">{{ t('read.empty') }}</p>
      <ul v-else class="read-list">
        <li v-for="item in listQ.data.value" :key="item.id">
          <RouterLink :to="`/read/${item.slug}`">
            <strong>{{ item.title }}</strong>
            <span class="muted">{{ item.excerpt }}</span>
            <span class="muted read-meta">{{ item.author }} · {{ formatStamp(item.publishedAt) }}</span>
          </RouterLink>
        </li>
      </ul>
    </template>
  </div>
</template>

<style scoped>
.read { max-width: 760px; margin: 0 auto; padding: 20px 16px 60px; }
.read-head { display: flex; justify-content: space-between; align-items: center; gap: 12px; margin-bottom: 24px; }
.read-brand { font-weight: 700; text-decoration: none; color: inherit; }
.read-head nav { display: flex; gap: 6px; }
.read-meta { font-size: 13px; }
.read-excerpt { font-size: 18px; color: var(--muted); }
.read-hero { max-width: 100%; border-radius: 10px; margin: 12px 0; }
.read-tags { display: flex; gap: 6px; flex-wrap: wrap; margin-top: 20px; }
.read-list { list-style: none; padding: 0; display: grid; gap: 10px; }
.read-list a {
  display: grid; gap: 3px; padding: 12px 14px; text-decoration: none; color: inherit;
  border: 1px solid var(--faint); border-radius: var(--radius); background: var(--surface);
  box-shadow: var(--shadow-xs); transition: background 0.15s, box-shadow 0.15s;
}
.read-list a:hover { background: var(--surface-2); box-shadow: var(--shadow-sm); }
</style>
