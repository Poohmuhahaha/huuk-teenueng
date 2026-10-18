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
  <div class="max-w-article mx-auto px-4 pt-5 pb-15">
    <header class="flex justify-between items-center gap-3 mb-6">
      <RouterLink class="font-bold no-underline text-inherit" to="/">Huuk <span class="text-muted">· Studio</span></RouterLink>
      <nav class="flex gap-1.5">
        <RouterLink class="btn" to="/read">{{ t('read.all') }}</RouterLink>
        <RouterLink class="btn" to="/">{{ t('read.back') }}</RouterLink>
      </nav>
    </header>

    <!-- single article -->
    <template v-if="slug">
      <div v-if="itemQ.isPending.value" class="text-muted">{{ t('common.loading') }}</div>
      <div v-else-if="notFound" class="card">
        <h1>{{ t('read.notFound') }}</h1>
        <p class="text-muted">{{ t('read.notFoundHint') }}</p>
        <RouterLink class="btn" to="/read">{{ t('read.all') }}</RouterLink>
      </div>
      <article v-else-if="itemQ.data.value">
        <p class="text-xs-plus text-muted">
          {{ itemQ.data.value.author }}
          <template v-if="itemQ.data.value.publishedAt"> · {{ formatStamp(itemQ.data.value.publishedAt) }}</template>
        </p>
        <h1 class="text-2xl font-bold my-2">{{ itemQ.data.value.title }}</h1>
        <p v-if="itemQ.data.value.excerpt" class="text-lg text-muted my-2">{{ itemQ.data.value.excerpt }}</p>
        <img v-if="itemQ.data.value.heroImageUrl" :src="itemQ.data.value.heroImageUrl"
          :alt="itemQ.data.value.title" class="max-w-full rounded-media my-3" />
        <!-- sanitized by renderMarkdown -->
        <div class="markdown" v-html="bodyHtml" />
        <p v-if="itemQ.data.value.tags.length" class="flex gap-1.5 flex-wrap mt-5">
          <span v-for="tag in itemQ.data.value.tags" :key="tag" class="chip">#{{ tag }}</span>
        </p>
      </article>
    </template>

    <!-- index -->
    <template v-else>
      <h1 class="text-2xl font-bold mb-4">{{ t('read.published') }}</h1>
      <div v-if="listQ.isPending.value" class="text-muted">{{ t('common.loading') }}</div>
      <p v-else-if="!listQ.data.value?.length" class="text-muted">{{ t('read.empty') }}</p>
      <ul v-else class="list-none p-0 grid gap-2.5">
        <li v-for="item in listQ.data.value" :key="item.id">
          <RouterLink :to="`/read/${item.slug}`" class="grid gap-tight py-3 px-3.5 no-underline text-inherit border border-faint rounded-card bg-surface shadow-xs transition hover:bg-surface-2 hover:shadow-sm">
            <strong>{{ item.title }}</strong>
            <span class="text-muted text-sm">{{ item.excerpt }}</span>
            <span class="text-xs-plus text-muted">{{ item.author }} · {{ formatStamp(item.publishedAt) }}</span>
          </RouterLink>
        </li>
      </ul>
    </template>
  </div>
</template>
