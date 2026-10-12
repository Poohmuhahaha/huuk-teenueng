<script setup lang="ts">
// Client-facing editor: markdown body with live preview, autosave, revisions,
// scheduled publishing and SEO fields. Optimistic concurrency via `version`.
import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { useRouter } from 'vue-router'
import {
  useContentItem, useContentRevisions, useDeleteContent, useDuplicateContent,
  usePermission, usePublishContent, useRestoreContentRevision, useScheduleContent,
  useUnpublishContent, useUpdateContent,
} from '@/core/queries'
import { renderMarkdown } from '@/core/markdown'
import { t } from '@/core/i18n'
import type { Content, ContentKind, ContentRevision, ContentStatus } from '@/mock/db'

const props = defineProps<{ id: string }>()
const router = useRouter()
const { can } = usePermission()

const canWrite = computed(() => can('content.write'))
const canPublish = computed(() => can('content.publish'))
const canDelete = computed(() => can('content.delete'))

const itemQ = useContentItem(computed(() => props.id))
const revisionsQ = useContentRevisions(computed(() => props.id))
const update = useUpdateContent()
const publish = usePublishContent()
const unpublish = useUnpublishContent()
const schedule = useScheduleContent()
const remove = useDeleteContent()
const duplicate = useDuplicateContent()
const restore = useRestoreContentRevision()

interface Editable {
  title: string
  slug: string
  kind: ContentKind
  body: string
  excerpt: string
  heroImageUrl: string
  tagsText: string
  seoTitle: string
  seoDescription: string
}

function fromContent(c: Content): Editable {
  return {
    title: c.title,
    slug: c.slug,
    kind: c.kind,
    body: c.body,
    excerpt: c.excerpt,
    heroImageUrl: c.heroImageUrl,
    tagsText: c.tags.join(', '),
    seoTitle: c.seoTitle,
    seoDescription: c.seoDescription,
  }
}

function parseTags(text: string): string[] {
  return [...new Set(text.split(',').map((tag) => tag.trim()).filter(Boolean))].slice(0, 20)
}

const draft = ref<Editable | null>(null)
const baseline = ref('')
const loadedVersion = ref(0)
const conflict = ref(false)
const serverVersion = ref(0)
const saving = ref(false)
const saveError = ref('')
const notice = ref('')
const mode = ref<'edit' | 'split' | 'preview'>('split')
const revisionsOpen = ref(false)
const revisionPreview = ref<ContentRevision | null>(null)
const publishOpen = ref(false)
const scheduleLocal = ref('')
const copied = ref(false)
const bodyEl = ref<HTMLTextAreaElement | null>(null)
const linkOpen = ref(false)
const linkDraft = ref('https://')
const revisionsCloseBtn = ref<HTMLButtonElement | null>(null)

const dirty = computed(() => Boolean(draft.value) && JSON.stringify(draft.value) !== baseline.value)
const content = computed<Content | null>(() => itemQ.data.value ?? null)
const status = computed<ContentStatus>(() => content.value?.status ?? 'draft')
const canEdit = computed(() => canWrite.value && status.value !== 'archived')
const revisions = computed<ContentRevision[]>(() => revisionsQ.data.value ?? [])
const previewHtml = computed(() => renderMarkdown(draft.value?.body ?? ''))
const wordCount = computed(() => (draft.value?.body ?? '').split(/\s+/).filter(Boolean).length)
const readingMinutes = computed(() => Math.max(1, Math.round(wordCount.value / 200)))
const publishedUrl = computed(() => {
  const slug = content.value?.slug
  if (!slug) return ''
  // History-mode aware: derive the public URL from the router, not from the
  // current location (which breaks under history deploy).
  try {
    const href = router.resolve({ path: `/read/${slug}` }).href
    if (href) return href.startsWith('http') ? href : `${window.location.origin}${href}`
  } catch {
    // Fall through to the legacy derivation below.
  }
  return `${window.location.origin}${window.location.pathname}#/read/${slug}`
})

// Adopt server state when there are no local edits; flag a conflict when the
// server moved while local edits are pending.
watch(
  () => itemQ.data.value,
  (next) => {
    if (!next) return
    const nextDraft = fromContent(next)
    const nextSnapshot = JSON.stringify(nextDraft)
    if (!dirty.value) {
      draft.value = nextDraft
      baseline.value = nextSnapshot
      loadedVersion.value = next.version
      conflict.value = false
      saveError.value = ''
      return
    }
    if (next.version !== loadedVersion.value) {
      conflict.value = true
      serverVersion.value = next.version
    }
  },
  { immediate: true },
)

// Autosave: 1.5s after the last keystroke, paused while conflicted.
let autosaveTimer: number | undefined
watch(
  draft,
  () => {
    window.clearTimeout(autosaveTimer)
    if (!dirty.value || !canEdit.value || conflict.value || saving.value || !canWrite.value) return
    autosaveTimer = window.setTimeout(() => {
      void save(undefined, true)
    }, 1500)
  },
  { deep: true },
)
onBeforeUnmount(() => {
  window.clearTimeout(autosaveTimer)
  document.removeEventListener('click', onPublishDocClick)
})

async function save(note?: string, silent = false): Promise<boolean> {
  const current = content.value
  const value = draft.value
  if (!current || !value || !canWrite.value || saving.value) return false
  if (!dirty.value || conflict.value) return !dirty.value && !conflict.value
  saving.value = true
  saveError.value = ''
  notice.value = ''
  try {
    const result = await update.mutateAsync({
      id: props.id,
      patch: {
        version: loadedVersion.value,
        title: value.title,
        slug: value.slug,
        kind: value.kind,
        body: value.body,
        excerpt: value.excerpt,
        heroImageUrl: value.heroImageUrl,
        tags: parseTags(value.tagsText),
        seoTitle: value.seoTitle,
        seoDescription: value.seoDescription,
        note,
      },
    })
    draft.value = fromContent(result)
    baseline.value = JSON.stringify(draft.value)
    loadedVersion.value = result.version
    conflict.value = false
    if (!silent) {
      notice.value = t('studio.saved')
      window.setTimeout(() => { if (notice.value === t('studio.saved')) notice.value = '' }, 2000)
    }
    return true
  } catch (e) {
    const message = e instanceof Error ? e.message : String(e)
    if (/content changed since version/.test(message)) {
      conflict.value = true
      serverVersion.value = itemQ.data.value?.version ?? loadedVersion.value
    } else {
      saveError.value = message
    }
    return false
  } finally {
    saving.value = false
  }
}

async function ensureSaved(): Promise<boolean> {
  if (conflict.value) return false
  if (dirty.value) return save(undefined, true)
  return true
}

function reloadLatest(): void {
  conflict.value = false
  void itemQ.refetch().then(() => {
    const next = itemQ.data.value
    if (next) {
      draft.value = fromContent(next)
      baseline.value = JSON.stringify(draft.value)
      loadedVersion.value = next.version
    }
  })
}

function keepMine(): void {
  // Re-read the server version at click time: the stored serverVersion may be
  // stale if the item refetched since the conflict was flagged.
  loadedVersion.value = itemQ.data.value?.version ?? serverVersion.value
  conflict.value = false
  void save(t('studio.overwriteNote'))
}

async function publishNow(): Promise<void> {
  publishOpen.value = false
  if (!canPublish.value || !(await ensureSaved())) return
  try {
    await publish.mutateAsync({ id: props.id, version: loadedVersion.value })
    notice.value = t('studio.published')
  } catch (e) {
    saveError.value = e instanceof Error ? e.message : String(e)
  }
}

async function schedulePublish(): Promise<void> {
  if (!canPublish.value || !scheduleLocal.value) return
  const when = new Date(scheduleLocal.value)
  if (Number.isNaN(when.getTime()) || when.getTime() <= Date.now()) {
    saveError.value = t('studio.scheduleFuture')
    return
  }
  if (!(await ensureSaved())) return
  try {
    await schedule.mutateAsync({ id: props.id, version: loadedVersion.value, scheduledFor: when.toISOString() })
    publishOpen.value = false
    notice.value = t('studio.scheduled')
  } catch (e) {
    saveError.value = e instanceof Error ? e.message : String(e)
  }
}

async function unpublishNow(): Promise<void> {
  if (!canPublish.value) return
  try {
    await unpublish.mutateAsync({ id: props.id, version: loadedVersion.value })
    notice.value = t('studio.unpublished')
  } catch (e) {
    saveError.value = e instanceof Error ? e.message : String(e)
  }
}

async function setStatus(next: 'draft' | 'review' | 'archived'): Promise<void> {
  if (!canWrite.value) return
  if (!(await ensureSaved())) return
  try {
    const result = await update.mutateAsync({
      id: props.id,
      patch: { version: loadedVersion.value, status: next, note: `Status → ${next}` },
    })
    draft.value = fromContent(result)
    baseline.value = JSON.stringify(draft.value)
    loadedVersion.value = result.version
  } catch (e) {
    saveError.value = e instanceof Error ? e.message : String(e)
  }
}

async function removeContent(): Promise<void> {
  if (!canDelete.value) return
  if (!window.confirm(t('studio.confirmDelete').replace('{title}', content.value?.title ?? ''))) return
  try {
    await remove.mutateAsync(props.id)
    void router.push('/studio')
  } catch (e) {
    saveError.value = e instanceof Error ? e.message : String(e)
  }
}

async function duplicateContent(): Promise<void> {
  if (!canWrite.value) return
  try {
    const copy = await duplicate.mutateAsync(props.id)
    void router.push(`/studio/${copy.id}`)
  } catch (e) {
    saveError.value = e instanceof Error ? e.message : String(e)
  }
}

async function restoreRevision(revision: ContentRevision): Promise<void> {
  if (!canWrite.value) return
  if (!window.confirm(t('studio.confirmRestore').replace('{n}', String(revision.revision)))) return
  try {
    await restore.mutateAsync({ id: props.id, revision: revision.revision, version: loadedVersion.value })
    revisionPreview.value = null
    notice.value = t('studio.restored')
  } catch (e) {
    saveError.value = e instanceof Error ? e.message : String(e)
  }
}

async function copyLink(): Promise<void> {
  if (!publishedUrl.value) return
  try {
    await navigator.clipboard.writeText(publishedUrl.value)
    copied.value = true
    window.setTimeout(() => { copied.value = false }, 1500)
  } catch {
    saveError.value = t('studio.copyFailed')
  }
}

// ---- markdown toolbar ----

function wrapSelection(before: string, after = before, placeholder = 'text'): void {
  const el = bodyEl.value
  const value = draft.value
  if (!el || !value) return
  const start = el.selectionStart
  const end = el.selectionEnd
  const selected = el.value.slice(start, end) || placeholder
  value.body = el.value.slice(0, start) + before + selected + after + el.value.slice(end)
  void nextTick(() => {
    el.focus()
    el.setSelectionRange(start + before.length, start + before.length + selected.length)
  })
}

function prefixLines(prefix: string): void {
  const el = bodyEl.value
  const value = draft.value
  if (!el || !value) return
  const start = el.selectionStart
  const end = el.selectionEnd
  const lineStart = el.value.lastIndexOf('\n', start - 1) + 1
  const lineEnd = el.value.indexOf('\n', end) === -1 ? el.value.length : el.value.indexOf('\n', end)
  const block = el.value.slice(lineStart, lineEnd)
  const next = block
    .split('\n')
    .map((line) => (line.startsWith(prefix) ? line : `${prefix}${line}`))
    .join('\n')
  value.body = el.value.slice(0, lineStart) + next + el.value.slice(lineEnd)
  void nextTick(() => {
    el.focus()
    el.setSelectionRange(lineStart, lineStart + next.length)
  })
}

function openLink(): void {
  linkDraft.value = 'https://'
  linkOpen.value = true
}

function confirmLink(): void {
  const url = linkDraft.value.trim()
  linkOpen.value = false
  if (!url) return
  wrapSelection('[', `](${url})`, t('studio.linkText'))
}

function cancelLink(): void {
  linkOpen.value = false
}

// The publish popover closes on outside click (trigger toggles, clicks inside
// .publish-wrap are ignored).
function onPublishDocClick(e: MouseEvent): void {
  if (!publishOpen.value) return
  const target = e.target as HTMLElement | null
  if (!target?.closest?.('.publish-wrap')) publishOpen.value = false
}

onMounted(() => document.addEventListener('click', onPublishDocClick))

// Move focus to the drawer close button when it opens.
watch(revisionsOpen, (open) => {
  if (open) {
    void nextTick(() => revisionsCloseBtn.value?.focus())
  }
})

function statusLabel(s: ContentStatus): string {
  return t(`studio.status.${s}`)
}

function formatStamp(stamp: string): string {
  const date = new Date(stamp)
  return Number.isNaN(date.getTime())
    ? stamp
    : date.toLocaleString(undefined, { month: 'short', day: 'numeric', hour: '2-digit', minute: '2-digit' })
}
</script>

<template>
  <div v-if="itemQ.isPending.value" class="muted">{{ t('studio.loading') }}</div>
  <div v-else-if="itemQ.isError.value" class="py-2 grid gap-2 justify-items-center in-[.deck.full]:justify-items-start">
    <p class="muted">{{ itemQ.error.value?.message }}</p>
    <button class="btn" @click="() => itemQ.refetch()">{{ t('common.tryAgain') }}</button>
  </div>
  <div v-else-if="content && draft" class="grid gap-1.5 in-[.deck.full]:gap-4">
    <header class="flex gap-2.5 items-center in-[.deck.full]:flex-wrap">
      <input v-model="draft.title" class="field text-lg font-bold flex-1" :disabled="!canEdit"
        :aria-label="t('studio.field.title')" :placeholder="t('studio.untitled')" />
      <div class="flex gap-2 items-center whitespace-nowrap">
        <span class="statuschip" :class="`st-${status}`">{{ statusLabel(status) }}</span>
        <span class="muted text-xs">
          <template v-if="saving">{{ t('studio.saving') }}</template>
          <template v-else-if="conflict">{{ t('studio.conflictShort') }}</template>
          <template v-else-if="dirty">{{ t('studio.unsaved') }}</template>
          <template v-else>{{ t('studio.saved') }}</template>
        </span>
      </div>
    </header>

    <div class="flex flex-wrap gap-1.5 items-center justify-center in-[.deck.full]:justify-start [&_.btn]:py-2 [&_.btn]:px-3.5 [&_.btn]:text-xs [&_.btn]:w-auto">
      <button class="btn btn-primary" :disabled="!canEdit || saving || !dirty || conflict" @click="save()">
        {{ t('studio.saveDraft') }}
      </button>
      <template v-if="canPublish">
        <div class="relative">
          <button class="btn" :disabled="!canEdit" :aria-expanded="publishOpen" @click="publishOpen = !publishOpen">
            {{ status === 'published' ? '✓ ' : '' }}{{ t('studio.publish') }} ▾
          </button>
          <div v-if="publishOpen" class="absolute z-30 top-[calc(100%+6px)] left-0 bg-surface border border-faint rounded-[var(--radius,10px)] p-2 min-w-[240px] grid gap-1 shadow-lg [&_.menubtn]:text-left" role="dialog" :aria-label="t('studio.publish')">
            <button class="menubtn" :disabled="status === 'published'" @click="publishNow">{{ t('studio.publishNow') }}</button>
            <label class="lbl" for="studio-schedule" style="margin: 6px 0 0;">{{ t('studio.scheduleFor') }}</label>
            <input id="studio-schedule" class="field" type="datetime-local" v-model="scheduleLocal" />
            <button class="menubtn" :disabled="!scheduleLocal" @click="schedulePublish">{{ t('studio.schedule') }}</button>
            <button v-if="status === 'published' || status === 'scheduled'" class="menubtn" @click="unpublishNow">
              {{ t('studio.unpublish') }}
            </button>
          </div>
        </div>
      </template>
      <template v-else-if="canWrite">
        <button v-if="status === 'draft'" class="btn" :disabled="!canEdit" @click="setStatus('review')">
          {{ t('studio.submitReview') }}
        </button>
        <button v-else-if="status === 'review'" class="btn" :disabled="!canEdit" @click="setStatus('draft')">
          {{ t('studio.backToDraft') }}
        </button>
      </template>
      <button class="btn" :disabled="!canWrite" :title="canWrite ? '' : t('auth.noPerm')" @click="duplicateContent">
        {{ t('studio.duplicate') }}
      </button>
      <button class="btn" @click="revisionsOpen = !revisionsOpen">
        {{ t('studio.revisions') }} ({{ content.revisions.length }})
      </button>
      <button v-if="status === 'archived' && canWrite" class="btn" @click="setStatus('draft')">
        {{ t('studio.restoreDraft') }}
      </button>
      <button v-else-if="canWrite" class="btn" @click="setStatus('archived')">{{ t('studio.archive') }}</button>
      <button v-if="canDelete" class="btn" @click="removeContent">{{ t('studio.delete') }}</button>
      <span class="flex-1" />
      <div class="flex gap-1" role="tablist" :aria-label="t('studio.view')">
        <button class="tab" :class="{ active: mode === 'edit' }" role="tab" :aria-selected="mode === 'edit'"
          @click="mode = 'edit'">{{ t('studio.view.edit') }}</button>
        <button class="tab" :class="{ active: mode === 'split' }" role="tab" :aria-selected="mode === 'split'"
          @click="mode = 'split'">{{ t('studio.view.split') }}</button>
        <button class="tab" :class="{ active: mode === 'preview' }" role="tab" :aria-selected="mode === 'preview'"
          @click="mode = 'preview'">{{ t('studio.view.preview') }}</button>
      </div>
    </div>

    <div v-if="conflict" class="flex flex-wrap gap-2 items-center border border-warning bg-warning-wash rounded-md py-2 px-2.5" role="alert">
      <strong>{{ t('studio.conflict') }}</strong>
      <span class="muted">{{ t('studio.conflictHint') }}</span>
      <button class="btn" @click="reloadLatest">{{ t('studio.reloadLatest') }}</button>
      <button class="btn btn-primary" @click="keepMine">{{ t('studio.keepMine') }}</button>
    </div>
    <p v-if="saveError" class="autherr" role="alert">{{ saveError }}</p>
    <p v-if="notice" class="muted" role="status">{{ notice }}</p>

    <div v-if="status === 'published' && publishedUrl" class="flex gap-2 items-center flex-wrap bg-wash py-2.5 px-3 rounded-md">
      <strong>{{ t('studio.live') }}</strong>
      <code class="text-xs [overflow-wrap:anywhere]">{{ publishedUrl }}</code>
      <button class="btn" @click="copyLink">{{ copied ? t('studio.copied') : t('studio.copyLink') }}</button>
      <RouterLink class="btn" :to="`/read/${content.slug}`" target="_blank">{{ t('studio.openLive') }}</RouterLink>
    </div>

    <div class="grid gap-2 items-start in-[.deck.full]:gap-4" :class="mode === 'split' ? 'grid-cols-1 md:grid-cols-2' : 'grid-cols-1'">
      <section v-show="mode !== 'preview'" class="min-w-0">
        <div class="grid2">
          <div>
            <label class="lbl" for="studio-slug">{{ t('studio.field.slug') }}</label>
            <input id="studio-slug" v-model="draft.slug" class="field" :disabled="!canEdit" />
          </div>
          <div>
            <label class="lbl" for="studio-kind">{{ t('studio.field.kind') }}</label>
            <select id="studio-kind" v-model="draft.kind" class="field" :disabled="!canEdit">
              <option value="article">{{ t('studio.kind.article') }}</option>
              <option value="page">{{ t('studio.kind.page') }}</option>
              <option value="note">{{ t('studio.kind.note') }}</option>
            </select>
          </div>
        </div>

        <label class="lbl" for="studio-excerpt">{{ t('studio.field.excerpt') }}</label>
        <input id="studio-excerpt" v-model="draft.excerpt" class="field" :disabled="!canEdit" maxlength="500" />

        <div class="flex gap-1 items-center justify-center in-[.deck.full]:justify-start my-2 mb-1.5" role="toolbar" :aria-label="t('studio.field.body')">
          <button type="button" class="toolbtn inline-flex items-center justify-center min-w-[32px] py-[3px] px-[7px] border border-faint rounded-md bg-surface text-ink cursor-pointer hover:bg-wash hover:border-line disabled:opacity-50 disabled:cursor-not-allowed transition" :disabled="!canEdit" :title="t('studio.tool.bold')"
            @click="wrapSelection('**', '**', t('studio.tool.text'))"><strong>B</strong></button>
          <button type="button" class="toolbtn inline-flex items-center justify-center min-w-[32px] py-[3px] px-[7px] border border-faint rounded-md bg-surface text-ink cursor-pointer hover:bg-wash hover:border-line disabled:opacity-50 disabled:cursor-not-allowed transition" :disabled="!canEdit" :title="t('studio.tool.italic')"
            @click="wrapSelection('*', '*', t('studio.tool.text'))"><em>I</em></button>
          <button type="button" class="toolbtn inline-flex items-center justify-center min-w-[32px] py-[3px] px-[7px] border border-faint rounded-md bg-surface text-ink cursor-pointer hover:bg-wash hover:border-line disabled:opacity-50 disabled:cursor-not-allowed transition" :disabled="!canEdit" :title="t('studio.tool.h2')"
            @click="prefixLines('## ')">H2</button>
          <button type="button" class="toolbtn inline-flex items-center justify-center min-w-[32px] py-[3px] px-[7px] border border-faint rounded-md bg-surface text-ink cursor-pointer hover:bg-wash hover:border-line disabled:opacity-50 disabled:cursor-not-allowed transition" :disabled="!canEdit" :title="t('studio.tool.list')"
            @click="prefixLines('- ')">•</button>
          <button type="button" class="toolbtn inline-flex items-center justify-center min-w-[32px] py-[3px] px-[7px] border border-faint rounded-md bg-surface text-ink cursor-pointer hover:bg-wash hover:border-line disabled:opacity-50 disabled:cursor-not-allowed transition" :disabled="!canEdit" :title="t('studio.tool.ordered')"
            @click="prefixLines('1. ')">1.</button>
          <button type="button" class="toolbtn inline-flex items-center justify-center min-w-[32px] py-[3px] px-[7px] border border-faint rounded-md bg-surface text-ink cursor-pointer hover:bg-wash hover:border-line disabled:opacity-50 disabled:cursor-not-allowed transition" :disabled="!canEdit" :title="t('studio.tool.quote')"
            @click="prefixLines('> ')">”</button>
          <button type="button" class="toolbtn inline-flex items-center justify-center min-w-[32px] py-[3px] px-[7px] border border-faint rounded-md bg-surface text-ink cursor-pointer hover:bg-wash hover:border-line disabled:opacity-50 disabled:cursor-not-allowed transition" :disabled="!canEdit" :title="t('studio.tool.code')"
            @click="wrapSelection('`', '`', 'code')">&lt;/&gt;</button>
          <button type="button" class="toolbtn inline-flex items-center justify-center min-w-[32px] py-[3px] px-[7px] border border-faint rounded-md bg-surface text-ink cursor-pointer hover:bg-wash hover:border-line disabled:opacity-50 disabled:cursor-not-allowed transition" :disabled="!canEdit" :title="t('studio.tool.link')"
            @click="openLink">🔗</button>
          <span class="muted text-xs">
            {{ wordCount }} {{ t('studio.words') }} · {{ readingMinutes }} {{ t('studio.minRead') }}
          </span>
        </div>
        <div v-if="linkOpen" class="flex gap-1.5 items-center my-0 mb-1.5 [&_.field]:flex-1 [&_.field]:min-w-0">
          <input v-model="linkDraft" class="field" :aria-label="t('studio.linkPrompt')" placeholder="https://" @keydown.enter.prevent="confirmLink" @keydown.esc.prevent="cancelLink" />
          <button type="button" class="btn" :disabled="!linkDraft.trim()" @click="confirmLink">{{ t('common.add') }}</button>
          <button type="button" class="btn" @click="cancelLink">{{ t('common.cancel') }}</button>
        </div>
        <textarea id="studio-body" ref="bodyEl" v-model="draft.body" class="field font-mono text-sm leading-relaxed"
          :disabled="!canEdit" rows="7" :aria-label="t('studio.field.body')" />

        <div class="grid2">
          <div>
            <label class="lbl" for="studio-tags">{{ t('studio.field.tags') }}</label>
            <input id="studio-tags" v-model="draft.tagsText" class="field" :disabled="!canEdit"
              :placeholder="t('studio.field.tagsHint')" />
          </div>
          <div>
            <label class="lbl" for="studio-hero">{{ t('studio.field.hero') }}</label>
            <input id="studio-hero" v-model="draft.heroImageUrl" class="field" :disabled="!canEdit" placeholder="https://" />
          </div>
        </div>

        <details class="mt-2.5">
          <summary>{{ t('studio.seo') }}</summary>
          <div class="grid2 mt">
            <div>
              <label class="lbl" for="studio-seo-title">{{ t('studio.field.seoTitle') }}</label>
              <input id="studio-seo-title" v-model="draft.seoTitle" class="field" :disabled="!canEdit" maxlength="200" />
              <span class="muted text-[11px]">{{ draft.seoTitle.length }}/200 (≤ 60)</span>
            </div>
            <div>
              <label class="lbl" for="studio-seo-desc">{{ t('studio.field.seoDescription') }}</label>
              <input id="studio-seo-desc" v-model="draft.seoDescription" class="field" :disabled="!canEdit" maxlength="500" />
              <span class="muted text-[11px]">{{ draft.seoDescription.length }}/500 (≤ 160)</span>
            </div>
          </div>
        </details>
      </section>

      <section v-show="mode !== 'edit'" class="min-w-0">
        <div class="flex justify-between gap-2 items-baseline mb-2">
          <strong>{{ t('studio.preview') }}</strong>
          <span class="muted">{{ t('studio.previewHint') }}</span>
        </div>
        <article class="border border-faint rounded-[var(--radius,10px)] py-2.5 px-3 bg-surface min-h-[140px]">
          <h1 v-if="draft.title" class="text-2xl font-bold my-2">{{ draft.title }}</h1>
          <p v-if="draft.excerpt" class="muted text-[15px] my-2">{{ draft.excerpt }}</p>
          <img v-if="draft.heroImageUrl" :src="draft.heroImageUrl" :alt="draft.title" class="max-w-full rounded-none my-3" @error="(e) => ((e.target as HTMLImageElement).style.display = 'none')" />
          <!-- renderMarkdown escapes all text before inserting its own tags -->
          <div class="markdown" v-html="previewHtml" />
        </article>
      </section>
    </div>

    <!-- revisions drawer -->
    <aside v-if="revisionsOpen" class="fixed top-0 right-0 bottom-0 w-[min(420px,92vw)] z-60 bg-surface border-l border-faint p-3.5 overflow-y-auto shadow-lg" role="dialog" :aria-label="t('studio.revisions')" @keydown.esc="revisionsOpen = false; revisionPreview = null">
      <div class="flex justify-between items-center mb-2">
        <strong>{{ t('studio.revisions') }}</strong>
        <button ref="revisionsCloseBtn" class="btn" @click="revisionsOpen = false; revisionPreview = null">{{ t('common.close') }}</button>
      </div>
      <div v-if="revisionsQ.isPending.value" class="muted">{{ t('common.loading') }}</div>
      <p v-else-if="!revisions.length" class="muted">{{ t('studio.noRevisions') }}</p>
      <ul v-else class="list-none m-0 p-0 grid gap-1.5">
        <li v-for="revision in revisions" :key="revision.revision">
          <button class="studio-item" :class="{ on: revisionPreview?.revision === revision.revision }"
            @click="revisionPreview = revision">
            <strong>#{{ revision.revision }} · {{ revision.note || t('studio.edit') }}</strong>
            <span class="muted">{{ revision.author }} · {{ formatStamp(revision.savedAt) }}</span>
          </button>
        </li>
      </ul>
      <div v-if="revisionPreview" class="border-t border-dashed border-line mt-2.5 pt-2">
        <h4 style="margin: 8px 0 4px;">{{ revisionPreview.title }}</h4>
        <div class="markdown" v-html="renderMarkdown(revisionPreview.body)" />
        <button class="btn btn-primary mt" :disabled="!canWrite" @click="restoreRevision(revisionPreview)">
          {{ t('studio.restoreThis') }}
        </button>
      </div>
    </aside>
  </div>
  <div v-else class="py-2 grid gap-2 justify-items-center in-[.deck.full]:justify-items-start">
    <p class="muted">Content not found</p>
    <RouterLink class="btn" to="/studio">{{ t('studio.title') }}</RouterLink>
  </div>
</template>
