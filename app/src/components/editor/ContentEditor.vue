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
import Drawer from '@/components/overlays/Drawer.vue'
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
    <button class="btn w-auto" @click="() => itemQ.refetch()">{{ t('common.tryAgain') }}</button>
  </div>
  <div v-else-if="content && draft" class="grid gap-3 in-[.deck.full]:gap-4">
    <!-- Header: Title + Status + Saved indicator -->
    <header class="flex items-center justify-between gap-3 flex-wrap">
      <div class="flex-1 min-w-[240px]">
        <input
          v-model="draft.title"
          class="field text-xl md:text-2xl font-bold w-full bg-transparent border-0 border-b border-line/60 rounded-none px-1 py-1.5 focus:border-accent focus:ring-0 placeholder:text-muted/60"
          :disabled="!canEdit"
          :aria-label="t('studio.field.title')"
          :placeholder="t('studio.untitled')"
        />
      </div>
      <div class="flex items-center gap-2 whitespace-nowrap">
        <span class="statuschip" :class="`st-${status}`">{{ statusLabel(status) }}</span>
        <span class="muted text-xs">
          <template v-if="saving">{{ t('studio.saving') }}</template>
          <template v-else-if="conflict">{{ t('studio.conflictShort') }}</template>
          <template v-else-if="dirty">{{ t('studio.unsaved') }}</template>
          <template v-else>{{ t('studio.saved') }}</template>
        </span>
      </div>
    </header>

    <!-- Streamlined Sticky Action Bar -->
    <div class="sticky top-[-16px] z-10 -mx-4 px-4 py-2 bg-surface/95 backdrop-blur-sm border-b border-line flex flex-wrap items-center justify-between gap-2">
      <!-- Left: Primary Publish & Save -->
      <div class="flex items-center gap-2">
        <button
          class="btn btn-primary w-auto text-xs py-1.5 px-3 font-semibold"
          :disabled="!canEdit || saving || !dirty || conflict"
          @click="save()"
        >
          {{ t('studio.saveDraft') }}
        </button>

        <template v-if="canPublish">
          <div class="relative publish-wrap">
            <button
              class="btn w-auto text-xs py-1.5 px-3"
              :disabled="!canEdit"
              :aria-expanded="publishOpen"
              @click="publishOpen = !publishOpen"
            >
              {{ status === 'published' ? '✓ ' : '' }}{{ t('studio.publish') }} ▾
            </button>
            <div
              v-if="publishOpen"
              class="absolute z-30 top-[calc(100%+6px)] left-0 bg-surface border border-faint rounded-xl p-3 min-w-[260px] grid gap-2 shadow-xl [&_.menubtn]:text-left"
              role="dialog"
              :aria-label="t('studio.publish')"
            >
              <button class="menubtn" :disabled="status === 'published'" @click="publishNow">
                {{ t('studio.publishNow') }}
              </button>
              <div class="border-t border-faint pt-2 grid gap-1">
                <label class="lbl text-2xs" for="studio-schedule">{{ t('studio.scheduleFor') }}</label>
                <input id="studio-schedule" class="field text-xs" type="datetime-local" v-model="scheduleLocal" />
                <button class="menubtn mt-1" :disabled="!scheduleLocal" @click="schedulePublish">
                  {{ t('studio.schedule') }}
                </button>
              </div>
              <button
                v-if="status === 'published' || status === 'scheduled'"
                class="menubtn text-danger border-t border-faint pt-2"
                @click="unpublishNow"
              >
                {{ t('studio.unpublish') }}
              </button>
            </div>
          </div>
        </template>
        <template v-else-if="canWrite">
          <button v-if="status === 'draft'" class="btn w-auto text-xs py-1.5 px-3" :disabled="!canEdit" @click="setStatus('review')">
            {{ t('studio.submitReview') }}
          </button>
          <button v-else-if="status === 'review'" class="btn w-auto text-xs py-1.5 px-3" :disabled="!canEdit" @click="setStatus('draft')">
            {{ t('studio.backToDraft') }}
          </button>
        </template>
      </div>

      <!-- Center: View Mode Segmented Control -->
      <div class="inline-flex items-center gap-0.5 bg-wash p-1 rounded-lg border border-line" role="tablist" :aria-label="t('studio.view')">
        <button
          class="tab py-1 px-3 rounded-md text-xs font-medium cursor-pointer transition-colors"
          :class="{ active: mode === 'edit', 'bg-surface shadow-xs font-semibold text-ink': mode === 'edit', 'text-muted hover:text-ink': mode !== 'edit' }"
          role="tab"
          :aria-selected="mode === 'edit'"
          @click="mode = 'edit'"
        >
          {{ t('studio.view.edit') }}
        </button>
        <button
          class="tab py-1 px-3 rounded-md text-xs font-medium cursor-pointer transition-colors"
          :class="{ active: mode === 'split', 'bg-surface shadow-xs font-semibold text-ink': mode === 'split', 'text-muted hover:text-ink': mode !== 'split' }"
          role="tab"
          :aria-selected="mode === 'split'"
          @click="mode = 'split'"
        >
          {{ t('studio.view.split') }}
        </button>
        <button
          class="tab py-1 px-3 rounded-md text-xs font-medium cursor-pointer transition-colors"
          :class="{ active: mode === 'preview', 'bg-surface shadow-xs font-semibold text-ink': mode === 'preview', 'text-muted hover:text-ink': mode !== 'preview' }"
          role="tab"
          :aria-selected="mode === 'preview'"
          @click="mode = 'preview'"
        >
          {{ t('studio.view.preview') }}
        </button>
      </div>

      <!-- Right: Revisions & Post Actions -->
      <div class="flex items-center gap-1.5">
        <button
          class="btn w-auto text-xs py-1.5 px-2.5 inline-flex items-center gap-1.5"
          :title="t('studio.revisions')"
          @click="revisionsOpen = !revisionsOpen"
        >
          <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
            <circle cx="12" cy="12" r="10"></circle>
            <polyline points="12 6 12 12 16 14"></polyline>
          </svg>
          <span>{{ t('studio.revisions') }}</span>
          <span class="inline-flex items-center justify-center px-1.5 py-0.2 rounded-full text-2xs bg-wash font-bold">{{ content.revisions.length }}</span>
        </button>

        <button
          class="btn w-auto text-xs py-1.5 px-2.5"
          :disabled="!canWrite"
          :title="canWrite ? '' : t('auth.noPerm')"
          @click="duplicateContent"
        >
          {{ t('studio.duplicate') }}
        </button>

        <button
          v-if="status === 'archived' && canWrite"
          class="btn w-auto text-xs py-1.5 px-2.5"
          @click="setStatus('draft')"
        >
          {{ t('studio.restoreDraft') }}
        </button>
        <button
          v-else-if="canWrite"
          class="btn w-auto text-xs py-1.5 px-2.5"
          @click="setStatus('archived')"
        >
          {{ t('studio.archive') }}
        </button>

        <button
          v-if="canDelete"
          class="btn w-auto text-xs py-1.5 px-2.5 text-danger hover:bg-danger-wash hover:border-danger/30"
          @click="removeContent"
        >
          {{ t('studio.delete') }}
        </button>
      </div>
    </div>

    <!-- Alert / Notice Banners -->
    <div v-if="conflict" class="flex flex-wrap gap-2 items-center border border-warning bg-warning-wash rounded-lg py-2 px-3 text-xs" role="alert">
      <strong>{{ t('studio.conflict') }}</strong>
      <span class="muted flex-1">{{ t('studio.conflictHint') }}</span>
      <button class="btn w-auto text-xs" @click="reloadLatest">{{ t('studio.reloadLatest') }}</button>
      <button class="btn btn-primary w-auto text-xs" @click="keepMine">{{ t('studio.keepMine') }}</button>
    </div>
    <p v-if="saveError" class="autherr text-xs" role="alert">{{ saveError }}</p>
    <p v-if="notice" class="muted text-xs" role="status">{{ notice }}</p>

    <div v-if="status === 'published' && publishedUrl" class="flex gap-2 items-center flex-wrap bg-wash/60 border border-faint py-2 px-3 rounded-lg text-xs">
      <strong class="text-ink">{{ t('studio.live') }}</strong>
      <code class="text-2xs px-2 py-0.5 rounded bg-surface border border-line [overflow-wrap:anywhere] flex-1">{{ publishedUrl }}</code>
      <button class="btn w-auto text-xs py-1 px-2" @click="copyLink">{{ copied ? t('studio.copied') : t('studio.copyLink') }}</button>
      <RouterLink class="btn w-auto text-xs py-1 px-2" :to="`/read/${content.slug}`" target="_blank">{{ t('studio.openLive') }}</RouterLink>
    </div>

    <!-- Writing Workspace: Split / Edit / Preview -->
    <div
      class="grid gap-4 items-start"
      :class="mode === 'split' ? 'grid-cols-1 md:grid-cols-2' : 'grid-cols-1'"
    >
      <!-- Left / Editor Column -->
      <section v-show="mode !== 'preview'" class="min-w-0 flex flex-col gap-2">
        <!-- Formatting Toolbar -->
        <div
          class="flex flex-wrap gap-1 items-center justify-between p-1.5 bg-wash/50 border border-faint rounded-lg"
          role="toolbar"
          :aria-label="t('studio.field.body')"
        >
          <div class="flex flex-wrap gap-1 items-center">
            <button
              type="button"
              class="toolbtn inline-flex items-center justify-center min-w-[30px] h-[30px] py-[3px] px-[7px] border border-faint rounded-md bg-surface text-ink cursor-pointer hover:bg-wash hover:border-line disabled:opacity-50 disabled:cursor-not-allowed transition text-xs font-bold"
              :disabled="!canEdit"
              :title="t('studio.tool.bold')"
              @click="wrapSelection('**', '**', t('studio.tool.text'))"
            >
              <strong>B</strong>
            </button>
            <button
              type="button"
              class="toolbtn inline-flex items-center justify-center min-w-[30px] h-[30px] py-[3px] px-[7px] border border-faint rounded-md bg-surface text-ink cursor-pointer hover:bg-wash hover:border-line disabled:opacity-50 disabled:cursor-not-allowed transition text-xs italic font-serif"
              :disabled="!canEdit"
              :title="t('studio.tool.italic')"
              @click="wrapSelection('*', '*', t('studio.tool.text'))"
            >
              <em>I</em>
            </button>
            <button
              type="button"
              class="toolbtn inline-flex items-center justify-center min-w-[30px] h-[30px] py-[3px] px-[7px] border border-faint rounded-md bg-surface text-ink cursor-pointer hover:bg-wash hover:border-line disabled:opacity-50 disabled:cursor-not-allowed transition text-xs font-semibold"
              :disabled="!canEdit"
              :title="t('studio.tool.h2')"
              @click="prefixLines('## ')"
            >
              H2
            </button>
            <button
              type="button"
              class="toolbtn inline-flex items-center justify-center min-w-[30px] h-[30px] py-[3px] px-[7px] border border-faint rounded-md bg-surface text-ink cursor-pointer hover:bg-wash hover:border-line disabled:opacity-50 disabled:cursor-not-allowed transition text-xs"
              :disabled="!canEdit"
              :title="t('studio.tool.list')"
              @click="prefixLines('- ')"
            >
              •
            </button>
            <button
              type="button"
              class="toolbtn inline-flex items-center justify-center min-w-[30px] h-[30px] py-[3px] px-[7px] border border-faint rounded-md bg-surface text-ink cursor-pointer hover:bg-wash hover:border-line disabled:opacity-50 disabled:cursor-not-allowed transition text-xs"
              :disabled="!canEdit"
              :title="t('studio.tool.ordered')"
              @click="prefixLines('1. ')"
            >
              1.
            </button>
            <button
              type="button"
              class="toolbtn inline-flex items-center justify-center min-w-[30px] h-[30px] py-[3px] px-[7px] border border-faint rounded-md bg-surface text-ink cursor-pointer hover:bg-wash hover:border-line disabled:opacity-50 disabled:cursor-not-allowed transition text-xs"
              :disabled="!canEdit"
              :title="t('studio.tool.quote')"
              @click="prefixLines('> ')"
            >
              ”
            </button>
            <button
              type="button"
              class="toolbtn inline-flex items-center justify-center min-w-[30px] h-[30px] py-[3px] px-[7px] border border-faint rounded-md bg-surface text-ink cursor-pointer hover:bg-wash hover:border-line disabled:opacity-50 disabled:cursor-not-allowed transition text-xs font-mono"
              :disabled="!canEdit"
              :title="t('studio.tool.code')"
              @click="wrapSelection('`', '`', 'code')"
            >
              &lt;/&gt;
            </button>
            <button
              type="button"
              class="toolbtn inline-flex items-center justify-center min-w-[30px] h-[30px] py-[3px] px-[7px] border border-faint rounded-md bg-surface text-ink cursor-pointer hover:bg-wash hover:border-line disabled:opacity-50 disabled:cursor-not-allowed transition text-xs"
              :disabled="!canEdit"
              :title="t('studio.tool.link')"
              @click="openLink"
            >
              <svg viewBox="0 0 16 16" width="13" height="13" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" aria-hidden="true"><path d="M6.5 9.5a3 3 0 0 0 4.2 0l2-2a3 3 0 0 0-4.2-4.2l-1 1" /><path d="M9.5 6.5a3 3 0 0 0-4.2 0l-2 2a3 3 0 0 0 4.2 4.2l1-1" /></svg>
            </button>
          </div>
          <span class="muted text-xs pr-1">
            {{ wordCount }} {{ t('studio.words') }} · {{ readingMinutes }} {{ t('studio.minRead') }}
          </span>
        </div>

        <!-- Link input popover bar -->
        <div v-if="linkOpen" class="flex gap-1.5 items-center my-0 mb-1.5 [&_.field]:flex-1 [&_.field]:min-w-0 bg-wash/60 p-2 rounded-lg border border-line">
          <input v-model="linkDraft" class="field text-xs" :aria-label="t('studio.linkPrompt')" placeholder="https://" @keydown.enter.prevent="confirmLink" @keydown.esc.prevent="cancelLink" />
          <button type="button" class="btn w-auto text-xs py-1 px-3" :disabled="!linkDraft.trim()" @click="confirmLink">{{ t('common.add') }}</button>
          <button type="button" class="btn w-auto text-xs py-1 px-3" @click="cancelLink">{{ t('common.cancel') }}</button>
        </div>

        <!-- Body Textarea -->
        <textarea
          id="studio-body"
          ref="bodyEl"
          v-model="draft.body"
          class="field font-mono text-sm leading-relaxed min-h-[300px] md:min-h-[380px] p-3.5 resize-y"
          :disabled="!canEdit"
          rows="14"
          :aria-label="t('studio.field.body')"
          placeholder="Write in Markdown here..."
        />
      </section>

      <!-- Right / Preview Column -->
      <section v-show="mode !== 'edit'" class="min-w-0 flex flex-col gap-2">
        <div class="flex justify-between gap-2 items-baseline p-1.5 bg-wash/50 border border-faint rounded-lg">
          <strong class="text-xs text-ink px-1">{{ t('studio.preview') }}</strong>
          <span class="muted text-xs pr-1">{{ t('studio.previewHint') }}</span>
        </div>
        <article class="border border-faint rounded-xl p-4 bg-surface min-h-[300px] md:min-h-[380px] overflow-y-auto">
          <h1 v-if="draft.title" class="text-2xl font-bold my-2">{{ draft.title }}</h1>
          <p v-if="draft.excerpt" class="muted text-[15px] my-2">{{ draft.excerpt }}</p>
          <img v-if="draft.heroImageUrl" :src="draft.heroImageUrl" :alt="draft.title" class="max-w-full rounded-lg my-3 border border-line" @error="(e) => ((e.target as HTMLImageElement).style.display = 'none')" />
          <!-- renderMarkdown escapes all text before inserting its own tags -->
          <div class="markdown" v-html="previewHtml" />
        </article>
      </section>
    </div>

    <!-- Structured Post Details & Metadata Card -->
    <section class="border border-line rounded-xl bg-surface p-4 grid gap-4 mt-2">
      <div class="flex items-center justify-between border-b border-faint pb-3">
        <div class="flex items-center gap-2">
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="text-muted"><path d="M12.22 2h-.44a2 2 0 0 0-2 2v.18a2 2 0 0 1-1 1.73l-.43.25a2 2 0 0 1-2 0l-.15-.08a2 2 0 0 0-2.73.73l-.22.38a2 2 0 0 0 .73 2.73l.15.1a2 2 0 0 1 1 1.72v.51a2 2 0 0 1-1 1.74l-.15.09a2 2 0 0 0-.73 2.73l.22.38a2 2 0 0 0 2.73.73l.15-.08a2 2 0 0 1 2 0l.43.25a2 2 0 0 1 1 1.73V20a2 2 0 0 0 2 2h.44a2 2 0 0 0 2-2v-.18a2 2 0 0 1 1-1.73l.43-.25a2 2 0 0 1 2 0l.15.08a2 2 0 0 0 2.73-.73l.22-.39a2 2 0 0 0-.73-2.73l-.15-.08a2 2 0 0 1-1-1.74v-.5a2 2 0 0 1 1-1.74l.15-.1a2 2 0 0 0 .73-2.73l-.22-.38a2 2 0 0 0-2.73-.73l-.15.08a2 2 0 0 1-2 0l-.43-.25a2 2 0 0 1-1-1.73V4a2 2 0 0 0-2-2z"></path><circle cx="12" cy="12" r="3"></circle></svg>
          <h3 class="text-sm font-semibold text-ink m-0">Post Details & Settings</h3>
        </div>
        <span class="text-2xs text-muted">Slug · Category · Summary · Media · SEO</span>
      </div>

      <!-- Group 1: Slug, Type, Summary -->
      <div class="grid grid-cols-1 md:grid-cols-3 gap-3">
        <div>
          <label class="lbl text-xs font-medium" for="studio-slug">{{ t('studio.field.slug') }}</label>
          <input id="studio-slug" v-model="draft.slug" class="field text-sm" :disabled="!canEdit" />
        </div>
        <div>
          <label class="lbl text-xs font-medium" for="studio-kind">{{ t('studio.field.kind') }}</label>
          <select id="studio-kind" v-model="draft.kind" class="field text-sm" :disabled="!canEdit">
            <option value="article">{{ t('studio.kind.article') }}</option>
            <option value="page">{{ t('studio.kind.page') }}</option>
            <option value="note">{{ t('studio.kind.note') }}</option>
          </select>
        </div>
        <div>
          <label class="lbl text-xs font-medium" for="studio-excerpt">{{ t('studio.field.excerpt') }}</label>
          <input id="studio-excerpt" v-model="draft.excerpt" class="field text-sm" :disabled="!canEdit" maxlength="500" />
        </div>
      </div>

      <!-- Group 2: Tags & Hero Image -->
      <div class="grid grid-cols-1 md:grid-cols-2 gap-3">
        <div>
          <label class="lbl text-xs font-medium" for="studio-tags">{{ t('studio.field.tags') }}</label>
          <input
            id="studio-tags"
            v-model="draft.tagsText"
            class="field text-sm"
            :disabled="!canEdit"
            :placeholder="t('studio.field.tagsHint')"
          />
        </div>
        <div>
          <label class="lbl text-xs font-medium" for="studio-hero">{{ t('studio.field.hero') }}</label>
          <div class="flex gap-2 items-center">
            <input id="studio-hero" v-model="draft.heroImageUrl" class="field text-sm flex-1" :disabled="!canEdit" placeholder="https://" />
            <div v-if="draft.heroImageUrl" class="w-9 h-9 rounded-lg border border-line overflow-hidden flex-none bg-wash flex items-center justify-center">
              <img :src="draft.heroImageUrl" alt="" class="w-full h-full object-cover" @error="(e) => ((e.target as HTMLImageElement).style.display = 'none')" />
            </div>
          </div>
        </div>
      </div>

      <!-- Group 3: SEO Listing -->
      <details class="border border-line rounded-lg p-3 bg-wash/30">
        <summary class="font-medium text-xs text-ink cursor-pointer select-none">{{ t('studio.seo') }}</summary>
        <div class="grid grid-cols-1 md:grid-cols-2 gap-3 mt-3">
          <div>
            <label class="lbl text-xs" for="studio-seo-title">{{ t('studio.field.seoTitle') }}</label>
            <input id="studio-seo-title" v-model="draft.seoTitle" class="field text-sm" :disabled="!canEdit" maxlength="200" />
            <span class="muted text-[11px]">{{ draft.seoTitle.length }}/200 (≤ 60)</span>
          </div>
          <div>
            <label class="lbl text-xs" for="studio-seo-desc">{{ t('studio.field.seoDescription') }}</label>
            <input id="studio-seo-desc" v-model="draft.seoDescription" class="field text-sm" :disabled="!canEdit" maxlength="500" />
            <span class="muted text-[11px]">{{ draft.seoDescription.length }}/500 (≤ 160)</span>
          </div>
        </div>
      </details>
    </section>

    <!-- Revisions Bottom Sheet Drawer -->
    <Drawer v-model="revisionsOpen" :title="t('studio.revisions')" side="bottom">
      <div class="grid gap-3 max-w-2xl mx-auto">
        <div v-if="revisionsQ.isPending.value" class="muted">{{ t('common.loading') }}</div>
        <p v-else-if="!revisions.length" class="muted">{{ t('studio.noRevisions') }}</p>
        <ul v-else class="list-none m-0 p-0 grid gap-2">
          <li v-for="revision in revisions" :key="revision.revision">
            <button
              class="w-full text-left p-3 rounded-xl border border-line bg-surface hover:bg-wash transition flex flex-col gap-1 cursor-pointer"
              :class="{ 'border-accent bg-wash': revisionPreview?.revision === revision.revision }"
              @click="revisionPreview = revision"
            >
              <div class="flex items-center justify-between">
                <strong class="text-sm text-ink">#{{ revision.revision }} · {{ revision.note || t('studio.edit') }}</strong>
                <span class="text-xs text-muted">{{ formatStamp(revision.savedAt) }}</span>
              </div>
              <span class="text-2xs text-muted">{{ revision.author }}</span>
            </button>
          </li>
        </ul>
        <div v-if="revisionPreview" class="border-t border-dashed border-line mt-2 pt-3">
          <h4 class="m-0 mb-2 font-bold">{{ revisionPreview.title }}</h4>
          <div class="markdown p-3 rounded-lg border border-line bg-wash/30 text-sm max-h-[300px] overflow-y-auto" v-html="renderMarkdown(revisionPreview.body)" />
          <button class="btn btn-primary mt-3 w-auto" :disabled="!canWrite" @click="restoreRevision(revisionPreview)">
            {{ t('studio.restoreThis') }}
          </button>
        </div>
      </div>
    </Drawer>
  </div>
  <div v-else class="py-2 grid gap-2 justify-items-center in-[.deck.full]:justify-items-start">
    <p class="muted">Content not found</p>
    <RouterLink class="btn w-auto" to="/studio">{{ t('studio.title') }}</RouterLink>
  </div>
</template>
