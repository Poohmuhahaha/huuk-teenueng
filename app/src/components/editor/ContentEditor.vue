<script setup lang="ts">
// Client-facing editor: markdown body with live preview, autosave, revisions,
// scheduled publishing and SEO fields. Optimistic concurrency via `version`.
import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { useRouter } from 'vue-router'
import {
  useBrand, useContentItem, useContentRevisions, useDeleteContent, useDuplicateContent,
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
const { data: brand } = useBrand()

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
const rootEl = ref<HTMLElement | null>(null)
const linkOpen = ref(false)
const linkDraft = ref('https://')
const revisionsCloseBtn = ref<HTMLButtonElement | null>(null)
const inspectorTab = ref<'all' | 'general' | 'seo'>('all')
const tagDraft = ref('')
const moreOpen = ref(false)
const heroBroken = ref(false)

const dirty = computed(() => Boolean(draft.value) && JSON.stringify(draft.value) !== baseline.value)
const content = computed<Content | null>(() => itemQ.data.value ?? null)
const status = computed<ContentStatus>(() => content.value?.status ?? 'draft')
const canEdit = computed(() => canWrite.value && status.value !== 'archived')
const revisions = computed<ContentRevision[]>(() => revisionsQ.data.value ?? [])
const previewHtml = computed(() => renderMarkdown(draft.value?.body ?? ''))
const wordCount = computed(() => (draft.value?.body ?? '').split(/\s+/).filter(Boolean).length)
const readingMinutes = computed(() => Math.max(1, Math.round(wordCount.value / 200)))
const characterCount = computed(() => (draft.value?.body ?? '').length)
const tags = computed(() => (draft.value ? parseTags(draft.value.tagsText) : []))
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
const previewUrl = computed(() => {
  const slug = draft.value?.slug || content.value?.slug
  if (!slug) return ''
  try {
    const href = router.resolve({ path: `/read/${slug}` }).href
    return href.startsWith('http') ? href : `${window.location.origin}${href}`
  } catch {
    return ''
  }
})
const shownUrl = computed(() =>
  status.value === 'published' && publishedUrl.value ? publishedUrl.value : previewUrl.value,
)
const seoTitleLength = computed(() => draft.value?.seoTitle.length ?? 0)
const seoDescLength = computed(() => draft.value?.seoDescription.length ?? 0)
const seoTitleTone = computed(() => (seoTitleLength.value > 60 ? 'warn' : 'ok'))
const seoDescTone = computed(() => (seoDescLength.value > 160 ? 'warn' : 'ok'))
const seoTitlePct = computed(() => Math.min(100, Math.round((seoTitleLength.value / 60) * 100)))
const seoDescPct = computed(() => Math.min(100, Math.round((seoDescLength.value / 160) * 100)))
const serpTitle = computed(() => draft.value?.seoTitle || draft.value?.title || t('studio.untitledPost'))
const serpDescription = computed(() => draft.value?.seoDescription || draft.value?.excerpt || '')
const serpHost = computed(() => (typeof window === 'undefined' ? 'localhost' : window.location.host))
const brandName = computed(() => brand.value?.channel?.trim() || 'Studio Publication')
const brandLogo = computed(() => (brand.value?.logos ?? []).find((url) => url.trim()) ?? '')
const brandInitial = computed(() => (brandName.value.charAt(0) || 'S').toUpperCase())
const inspectorTabs = computed(() => [
  { id: 'all' as const, label: t('studio.inspector.all') },
  { id: 'general' as const, label: t('studio.inspector.general') },
  { id: 'seo' as const, label: t('studio.inspector.seo') },
])

watch(
  () => draft.value?.heroImageUrl,
  () => { heroBroken.value = false },
)

function addTag(): void {
  if (!draft.value) return
  const raw = tagDraft.value.trim().replace(/^#/, '')
  if (!raw) return
  const next = parseTags(`${draft.value.tagsText}, ${raw}`)
  draft.value.tagsText = next.join(', ')
  tagDraft.value = ''
}

function removeTag(tag: string): void {
  if (!draft.value) return
  draft.value.tagsText = tags.value.filter((item) => item !== tag).join(', ')
}

function onTagKeydown(event: KeyboardEvent): void {
  if (event.key === 'Enter' || event.key === ',') {
    event.preventDefault()
    addTag()
    return
  }
  if (event.key === 'Backspace' && !tagDraft.value) {
    const last = tags.value[tags.value.length - 1]
    if (last) removeTag(last)
  }
}

function onHeroError(): void {
  heroBroken.value = true
}

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
  if (!shownUrl.value) return
  try {
    await navigator.clipboard.writeText(shownUrl.value)
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
  const target = e.target as HTMLElement | null
  if (publishOpen.value && !target?.closest?.('.publish-wrap')) publishOpen.value = false
  if (moreOpen.value && !target?.closest?.('.studio-more-wrap')) moreOpen.value = false
}

function goToPosts(): void {
  void router.push({ path: '/content', query: { tab: 'studio' } })
}

function scrollToSettings(): void {
  rootEl.value?.querySelector('#studio-settings')?.scrollIntoView({ behavior: 'smooth', block: 'start' })
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
  <div v-else-if="content && draft" ref="rootEl" class="studio-editor">
    <!-- Sticky studio command bar -->
    <header class="studio-cmdbar">
      <div class="studio-cmd-left">
        <button type="button" class="studio-crumb" @click="goToPosts">
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
            <path d="m15 18-6-6 6-6" />
          </svg>
          <span>{{ t('studio.posts') }}</span>
        </button>
        <span class="studio-crumb-sep" aria-hidden="true">/</span>
        <span class="statuschip" :class="`st-${status}`">{{ statusLabel(status) }}</span>
        <span class="studio-save-state" aria-live="polite">
          <svg v-if="!saving && !conflict && !dirty" width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
            <path d="M20 6 9 17l-5-5" />
          </svg>
          <span v-if="saving">{{ t('studio.saving') }}</span>
          <span v-else-if="conflict">{{ t('studio.conflictShort') }}</span>
          <span v-else-if="dirty">{{ t('studio.unsaved') }}</span>
          <span v-else>{{ t('studio.saved') }}</span>
        </span>
      </div>

      <div class="studio-viewswitch" role="tablist" :aria-label="t('studio.view')">
        <button type="button" class="studio-view-tab" :class="{ active: mode === 'edit' }" role="tab" :aria-selected="mode === 'edit'" @click="mode = 'edit'">
          <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="M12 20h9" /><path d="M16.5 3.5a2.12 2.12 0 0 1 3 3L7 19l-4 1 1-4Z" /></svg>
          <span>{{ t('studio.view.edit') }}</span>
        </button>
        <button type="button" class="studio-view-tab" :class="{ active: mode === 'split' }" role="tab" :aria-selected="mode === 'split'" @click="mode = 'split'">
          <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><rect width="18" height="18" x="3" y="3" rx="2" /><path d="M12 3v18" /></svg>
          <span>{{ t('studio.view.split') }}</span>
        </button>
        <button type="button" class="studio-view-tab" :class="{ active: mode === 'preview' }" role="tab" :aria-selected="mode === 'preview'" @click="mode = 'preview'">
          <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="M2 12s3-7 10-7 10 7 10 7-3 7-10 7-10-7-10-7Z" /><circle cx="12" cy="12" r="3" /></svg>
          <span>{{ t('studio.view.preview') }}</span>
        </button>
      </div>

      <div class="studio-cmd-actions">
        <button type="button" class="studio-icon-btn" :title="t('studio.revisions')" @click="revisionsOpen = !revisionsOpen">
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><circle cx="12" cy="12" r="10" /><polyline points="12 6 12 12 16 14" /></svg>
          <span class="studio-btn-label">{{ t('studio.revisions') }}</span>
          <span class="studio-count">{{ content.revisions.length }}</span>
        </button>
        <button type="button" class="studio-icon-btn" :title="t('studio.settings')" @click="scrollToSettings">
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="M12.22 2h-.44a2 2 0 0 0-2 2v.18a2 2 0 0 1-1 1.73l-.43.25a2 2 0 0 1-2 0l-.15-.08a2 2 0 0 0-2.73.73l-.22.38a2 2 0 0 0 .73 2.73l.15.1a2 2 0 0 1 1 1.72v.51a2 2 0 0 1-1 1.74l-.15.09a2 2 0 0 0-.73 2.73l.22.38a2 2 0 0 0 2.73.73l.15-.08a2 2 0 0 1 2 0l.43.25a2 2 0 0 1 1 1.73V20a2 2 0 0 0 2 2h.44a2 2 0 0 0 2-2v-.18a2 2 0 0 1 1-1.73l.43-.25a2 2 0 0 1 2 0l.15.08a2 2 0 0 0 2.73-.73l.22-.39a2 2 0 0 0-.73-2.73l-.15-.08a2 2 0 0 1-1-1.74v-.5a2 2 0 0 1 1-1.74l.15-.1a2 2 0 0 0 .73-2.73l-.22-.38a2 2 0 0 0-2.73-.73l-.15.08a2 2 0 0 1-2 0l-.43-.25a2 2 0 0 1-1-1.73V4a2 2 0 0 0-2-2z" /><circle cx="12" cy="12" r="3" /></svg>
          <span class="studio-btn-label">{{ t('studio.settings') }}</span>
        </button>

        <template v-if="!canPublish && canWrite">
          <button v-if="status === 'draft'" type="button" class="studio-text-btn" :disabled="!canEdit" @click="setStatus('review')">
            {{ t('studio.submitReview') }}
          </button>
          <button v-else-if="status === 'review'" type="button" class="studio-text-btn" :disabled="!canEdit" @click="setStatus('draft')">
            {{ t('studio.backToDraft') }}
          </button>
        </template>

        <div class="studio-save-split publish-wrap" :class="{ 'is-solo': !canPublish }">
          <button type="button" class="studio-save-main" :disabled="!canEdit || saving || !dirty || conflict" @click="save()">
            <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="M20 6 9 17l-5-5" /></svg>
            <span>{{ dirty ? t('studio.saveDraft') : t('studio.saved') }}</span>
          </button>
          <button v-if="canPublish" type="button" class="studio-save-caret" :aria-expanded="publishOpen" :aria-label="t('studio.publish')" @click="publishOpen = !publishOpen">
            <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="m6 9 6 6 6-6" /></svg>
          </button>

          <div v-if="publishOpen" class="studio-publish-menu" role="dialog" :aria-label="t('studio.publishOptions')">
            <div class="studio-menu-label">{{ t('studio.publishOptions') }}</div>
            <button type="button" class="studio-publish-option" :disabled="status === 'published'" @click="publishNow">
              <span class="studio-option-dot is-live" aria-hidden="true"></span>
              <span>
                <strong>{{ t('studio.publishNow') }}</strong>
                <small>{{ t('studio.publishLiveHint') }}</small>
              </span>
            </button>
            <div class="studio-schedule-block">
              <label class="studio-field-label" for="studio-schedule">{{ t('studio.scheduleFor') }}</label>
              <input id="studio-schedule" v-model="scheduleLocal" type="datetime-local" class="field studio-input" />
              <button type="button" class="btn w-auto studio-small-btn" :disabled="!scheduleLocal" @click="schedulePublish">
                {{ t('studio.schedule') }}
              </button>
            </div>
            <button v-if="status === 'published' || status === 'scheduled'" type="button" class="studio-publish-option is-danger" @click="unpublishNow">
              <span class="studio-option-dot is-muted" aria-hidden="true"></span>
              <span><strong>{{ t('studio.unpublish') }}</strong></span>
            </button>
          </div>
        </div>

        <div class="studio-more-wrap">
          <button type="button" class="studio-icon-btn studio-more-btn" :title="t('studio.moreActions')" :aria-expanded="moreOpen" @click="moreOpen = !moreOpen">
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><circle cx="12" cy="12" r="1" /><circle cx="19" cy="12" r="1" /><circle cx="5" cy="12" r="1" /></svg>
          </button>
          <div v-if="moreOpen" class="studio-more-menu">
            <button v-if="canWrite" type="button" class="studio-more-item" @click="moreOpen = false; duplicateContent()">
              <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><rect width="14" height="14" x="8" y="8" rx="2" ry="2" /><path d="M4 16c-1.1 0-2-.9-2-2V4c0-1.1.9-2 2-2h10c1.1 0 2 .9 2 2" /></svg>
              {{ t('studio.duplicate') }}
            </button>
            <button v-if="status === 'archived' && canWrite" type="button" class="studio-more-item" @click="moreOpen = false; setStatus('draft')">
              {{ t('studio.restoreDraft') }}
            </button>
            <button v-else-if="canWrite" type="button" class="studio-more-item" @click="moreOpen = false; setStatus('archived')">
              {{ t('studio.archive') }}
            </button>
            <div v-if="canDelete" class="studio-menu-divider"></div>
            <button v-if="canDelete" type="button" class="studio-more-item is-danger" @click="moreOpen = false; removeContent()">
              <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="M3 6h18" /><path d="M19 6v14c0 1-1 2-2 2H7c-1 0-2-1-2-2V6" /><path d="M8 6V4c0-1 1-2 2-2h4c1 0 2 1 2 2v2" /></svg>
              {{ t('studio.delete') }}
            </button>
          </div>
        </div>
      </div>
    </header>

    <div v-if="conflict" class="studio-alert is-warning" role="alert">
      <strong>{{ t('studio.conflict') }}</strong>
      <span>{{ t('studio.conflictHint') }}</span>
      <button type="button" class="btn w-auto studio-small-btn" @click="reloadLatest">{{ t('studio.reloadLatest') }}</button>
      <button type="button" class="btn btn-primary w-auto studio-small-btn" @click="keepMine">{{ t('studio.keepMine') }}</button>
    </div>
    <p v-if="saveError" class="studio-alert is-danger" role="alert">{{ saveError }}</p>
    <p v-if="notice" class="studio-notice" role="status">{{ notice }}</p>

    <!-- Document header: live URL, title and subtitle -->
    <section class="studio-doc-head">
      <div class="studio-url-row">
        <div class="studio-url-pill">
          <span class="studio-url-state">
            <span class="studio-url-dot" :class="{ live: status === 'published' }" aria-hidden="true"></span>
            <span>{{ status === 'published' ? t('studio.live') : t('studio.previewUrl') }}</span>
          </span>
          <span class="studio-url-sep" aria-hidden="true">|</span>
          <code>{{ shownUrl }}</code>
          <button type="button" class="studio-pill-btn" @click="copyLink">{{ copied ? t('studio.copied') : t('studio.copyLink') }}</button>
          <RouterLink v-if="status === 'published'" class="studio-pill-link" :to="`/read/${content.slug}`" target="_blank">
            {{ t('studio.openLive') }}
            <svg width="11" height="11" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="M7 7h10v10" /><path d="M7 17 17 7" /></svg>
          </RouterLink>
        </div>
        <div class="studio-doc-meta">
          <span>{{ t('studio.typeLabel') }}: <strong>{{ t(`studio.kind.${draft.kind}`) }}</strong></span>
          <span aria-hidden="true">·</span>
          <span>{{ t('studio.lastEdited') }} <strong>{{ formatStamp(content.updatedAt) }}</strong></span>
        </div>
      </div>

      <div class="studio-title-card">
        <input v-model="draft.title" class="studio-title-input" :disabled="!canEdit" :aria-label="t('studio.field.title')" :placeholder="t('studio.untitled')" />
        <input v-model="draft.excerpt" class="studio-subtitle-input" :disabled="!canEdit" :aria-label="t('studio.field.excerpt')" maxlength="500" placeholder="Add a subtitle or brief summary for cards and previews…" />
      </div>
    </section>

    <!-- Unified writing workspace -->
    <section class="studio-workspace">
      <div class="studio-toolbar" role="toolbar" :aria-label="t('studio.toolbar')">
        <div class="studio-tools">
          <button type="button" class="toolbtn studio-tool" :disabled="!canEdit" :title="t('studio.tool.bold')" @click="wrapSelection('**', '**', t('studio.tool.text'))"><strong>B</strong></button>
          <button type="button" class="toolbtn studio-tool is-italic" :disabled="!canEdit" :title="t('studio.tool.italic')" @click="wrapSelection('*', '*', t('studio.tool.text'))"><em>I</em></button>
          <button type="button" class="toolbtn studio-tool" :disabled="!canEdit" :title="t('studio.tool.h2')" @click="prefixLines('## ')">H2</button>
          <button type="button" class="toolbtn studio-tool" :disabled="!canEdit" :title="t('studio.tool.h3')" @click="prefixLines('### ')">H3</button>
          <span class="studio-tool-sep" aria-hidden="true"></span>
          <button type="button" class="toolbtn studio-tool" :disabled="!canEdit" :title="t('studio.tool.list')" @click="prefixLines('- ')">
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><line x1="8" x2="21" y1="6" y2="6" /><line x1="8" x2="21" y1="12" y2="12" /><line x1="8" x2="21" y1="18" y2="18" /><line x1="3" x2="3.01" y1="6" y2="6" /><line x1="3" x2="3.01" y1="12" y2="12" /><line x1="3" x2="3.01" y1="18" y2="18" /></svg>
          </button>
          <button type="button" class="toolbtn studio-tool" :disabled="!canEdit" :title="t('studio.tool.ordered')" @click="prefixLines('1. ')">
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><line x1="10" x2="21" y1="6" y2="6" /><line x1="10" x2="21" y1="12" y2="12" /><line x1="10" x2="21" y1="18" y2="18" /><path d="M4 6h1v4" /><path d="M4 10h2" /><path d="M6 18H4c0-1 2-2 2-3s-1-1.5-2-1" /></svg>
          </button>
          <button type="button" class="toolbtn studio-tool" :disabled="!canEdit" :title="t('studio.tool.quote')" @click="prefixLines('> ')">”</button>
          <span class="studio-tool-sep" aria-hidden="true"></span>
          <button type="button" class="toolbtn studio-tool is-mono" :disabled="!canEdit" :title="t('studio.tool.code')" @click="wrapSelection('`', '`', 'code')">&lt;/&gt;</button>
          <button type="button" class="toolbtn studio-tool" :disabled="!canEdit" :title="t('studio.tool.link')" @click="openLink">
            <svg width="14" height="14" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" aria-hidden="true"><path d="M6.5 9.5a3 3 0 0 0 4.2 0l2-2a3 3 0 0 0-4.2-4.2l-1 1" /><path d="M9.5 6.5a3 3 0 0 0-4.2 0l-2 2a3 3 0 0 0 4.2 4.2l1-1" /></svg>
          </button>
        </div>
        <div class="studio-metrics">
          <span>{{ wordCount }} {{ t('studio.words') }}</span>
          <span aria-hidden="true">·</span>
          <span>{{ characterCount }} {{ t('studio.characters') }}</span>
          <span aria-hidden="true">·</span>
          <span class="is-strong">{{ readingMinutes }} {{ t('studio.minRead') }}</span>
        </div>
      </div>

      <div class="studio-panes" :class="{ 'is-split': mode === 'split' }">
        <section v-show="mode !== 'preview'" class="studio-pane studio-pane-editor">
          <div class="studio-pane-head">
            <span>{{ t('studio.markdownSource') }}</span>
            <span>UTF-8</span>
          </div>
          <div v-if="linkOpen" class="studio-linkbar">
            <input v-model="linkDraft" class="field studio-input" :aria-label="t('studio.linkPrompt')" placeholder="https://" @keydown.enter.prevent="confirmLink" @keydown.esc.prevent="cancelLink" />
            <button type="button" class="btn w-auto studio-small-btn" :disabled="!linkDraft.trim()" @click="confirmLink">{{ t('common.add') }}</button>
            <button type="button" class="btn w-auto studio-small-btn" @click="cancelLink">{{ t('common.cancel') }}</button>
          </div>
          <textarea
            id="studio-body"
            ref="bodyEl"
            v-model="draft.body"
            class="studio-textarea"
            :disabled="!canEdit"
            rows="14"
            :aria-label="t('studio.field.body')"
            :placeholder="t('studio.bodyPlaceholder')"
          />
        </section>

        <section v-show="mode !== 'edit'" class="studio-pane studio-pane-preview">
          <div class="studio-pane-head">
            <span class="studio-live-label"><span class="studio-live-dot" aria-hidden="true"></span>{{ t('studio.livePreview') }}</span>
            <span>{{ t('studio.previewHint') }}</span>
          </div>
          <article class="studio-preview-article">
            <div v-if="tags.length" class="studio-preview-tags">
              <span v-for="tag in tags" :key="tag">#{{ tag }}</span>
            </div>
            <div v-if="draft.heroImageUrl && !heroBroken" class="studio-preview-hero">
              <img :src="draft.heroImageUrl" :alt="draft.title" @error="onHeroError" />
            </div>
            <h1>{{ draft.title || t('studio.untitled') }}</h1>
            <p v-if="draft.excerpt" class="studio-preview-dek">{{ draft.excerpt }}</p>
            <!-- renderMarkdown escapes all text before inserting its own tags -->
            <div class="markdown" v-html="previewHtml" />
          </article>
        </section>
      </div>
    </section>

    <!-- Settings inspector -->
    <section id="studio-settings" class="studio-inspector">
      <header class="studio-inspector-head">
        <div class="studio-inspector-title">
          <span class="studio-inspector-icon" aria-hidden="true">
            <svg width="17" height="17" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M12.22 2h-.44a2 2 0 0 0-2 2v.18a2 2 0 0 1-1 1.73l-.43.25a2 2 0 0 1-2 0l-.15-.08a2 2 0 0 0-2.73.73l-.22.38a2 2 0 0 0 .73 2.73l.15.1a2 2 0 0 1 1 1.72v.51a2 2 0 0 1-1 1.74l-.15.09a2 2 0 0 0-.73 2.73l.22.38a2 2 0 0 0 2.73.73l.15-.08a2 2 0 0 1 2 0l.43.25a2 2 0 0 1 1 1.73V20a2 2 0 0 0 2 2h.44a2 2 0 0 0 2-2v-.18a2 2 0 0 1 1-1.73l.43-.25a2 2 0 0 1 2 0l.15.08a2 2 0 0 0 2.73-.73l.22-.39a2 2 0 0 0-.73-2.73l-.15-.08a2 2 0 0 1-1-1.74v-.5a2 2 0 0 1 1-1.74l.15-.1a2 2 0 0 0 .73-2.73l-.22-.38a2 2 0 0 0-2.73-.73l-.15.08a2 2 0 0 1-2 0l-.43-.25a2 2 0 0 1-1-1.73V4a2 2 0 0 0-2-2z" /><circle cx="12" cy="12" r="3" /></svg>
          </span>
          <div>
            <h3>{{ t('studio.detailsTitle') }}</h3>
            <p>{{ t('studio.detailsHint') }}</p>
          </div>
        </div>
        <div class="studio-inspector-tabs" role="tablist" :aria-label="t('studio.detailsTitle')">
          <button
            v-for="tab in inspectorTabs"
            :key="tab.id"
            type="button"
            class="studio-inspector-tab"
            :class="{ active: inspectorTab === tab.id }"
            role="tab"
            :aria-selected="inspectorTab === tab.id"
            @click="inspectorTab = tab.id"
          >
            {{ tab.label }}
          </button>
        </div>
      </header>

      <div class="studio-inspector-body">
        <div v-show="inspectorTab !== 'seo'" class="studio-general">
          <div class="studio-settings-row is-three">
            <div class="studio-field">
              <label class="studio-field-label" for="studio-slug">{{ t('studio.field.slug') }}</label>
              <div class="studio-prefix-input">
                <span>{{ t('studio.slugPrefix') }}</span>
                <input id="studio-slug" v-model="draft.slug" :disabled="!canEdit" />
              </div>
            </div>
            <div class="studio-field">
              <label class="studio-field-label" for="studio-kind">{{ t('studio.postFormat') }}</label>
              <select id="studio-kind" v-model="draft.kind" class="field studio-input" :disabled="!canEdit">
                <option value="article">{{ t('studio.kind.article') }}</option>
                <option value="page">{{ t('studio.kind.page') }}</option>
                <option value="note">{{ t('studio.kind.note') }}</option>
              </select>
            </div>
            <div class="studio-field">
              <div class="studio-label-row">
                <label class="studio-field-label" for="studio-excerpt">{{ t('studio.cardSummary') }}</label>
                <span class="studio-counter">{{ draft.excerpt.length }} / 500</span>
              </div>
              <textarea id="studio-excerpt" v-model="draft.excerpt" class="field studio-input studio-textarea-sm" :disabled="!canEdit" maxlength="500" rows="2"></textarea>
            </div>
          </div>

          <div class="studio-settings-row is-tags-hero">
            <div class="studio-field">
              <div class="studio-label-row">
                <label class="studio-field-label" for="studio-tags">{{ t('studio.tagsTopics') }}</label>
                <span class="studio-counter">{{ t('studio.tagHint') }}</span>
              </div>
              <div class="studio-tagbox">
                <span v-for="tag in tags" :key="tag" class="studio-tag">
                  #{{ tag }}
                  <button type="button" :aria-label="`Remove ${tag}`" @click="removeTag(tag)">×</button>
                </span>
                <input id="studio-tags" v-model="tagDraft" class="studio-tag-input" :disabled="!canEdit" :placeholder="t('studio.addTag')" @keydown="onTagKeydown" @blur="addTag" />
              </div>
            </div>
            <div class="studio-field">
              <label class="studio-field-label" for="studio-hero">{{ t('studio.heroCover') }}</label>
              <div class="studio-hero-input-row">
                <input id="studio-hero" v-model="draft.heroImageUrl" class="field studio-input" :disabled="!canEdit" placeholder="https://" />
                <button v-if="draft.heroImageUrl" type="button" class="btn w-auto studio-small-btn" @click="draft.heroImageUrl = ''">{{ t('studio.clear') }}</button>
              </div>
              <div class="studio-hero-preview" :class="{ 'is-empty': !draft.heroImageUrl || heroBroken }">
                <img v-if="draft.heroImageUrl && !heroBroken" :src="draft.heroImageUrl" alt="" @error="onHeroError" />
                <div v-else class="studio-hero-empty">
                  <svg width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><rect width="18" height="18" x="3" y="3" rx="2" ry="2" /><circle cx="9" cy="9" r="2" /><path d="m21 15-3.086-3.086a2 2 0 0 0-2.828 0L6 21" /></svg>
                  <strong>{{ t('studio.heroEmpty') }}</strong>
                  <span>{{ t('studio.heroHint') }}</span>
                </div>
              </div>
            </div>
          </div>
        </div>

        <div v-show="inspectorTab !== 'general'" class="studio-seo">
          <div class="studio-seo-intro">
            <h3>{{ t('studio.searchOptimization') }}</h3>
            <p>{{ t('studio.seoHint') }}</p>
          </div>
          <div class="studio-seo-grid">
            <div class="studio-seo-fields">
              <div class="studio-field">
                <div class="studio-label-row">
                  <label class="studio-field-label" for="studio-seo-title">{{ t('studio.field.seoTitle') }}</label>
                  <span class="studio-counter" :class="`is-${seoTitleTone}`">{{ t('studio.optimal', { n: seoTitleLength, max: 60 }) }}</span>
                </div>
                <input id="studio-seo-title" v-model="draft.seoTitle" class="field studio-input" :disabled="!canEdit" maxlength="200" />
                <div class="studio-meter"><span :class="`is-${seoTitleTone}`" :style="{ width: `${seoTitlePct}%` }"></span></div>
              </div>
              <div class="studio-field">
                <div class="studio-label-row">
                  <label class="studio-field-label" for="studio-seo-desc">{{ t('studio.field.seoDescription') }}</label>
                  <span class="studio-counter" :class="`is-${seoDescTone}`">{{ t('studio.optimal', { n: seoDescLength, max: 160 }) }}</span>
                </div>
                <textarea id="studio-seo-desc" v-model="draft.seoDescription" class="field studio-input studio-textarea-sm" :disabled="!canEdit" maxlength="500" rows="3"></textarea>
                <div class="studio-meter"><span :class="`is-${seoDescTone}`" :style="{ width: `${seoDescPct}%` }"></span></div>
              </div>
            </div>

            <div class="studio-serp">
              <div class="studio-serp-head">
                <span>
                  <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><circle cx="11" cy="11" r="8" /><path d="m21 21-4.3-4.3" /></svg>
                  {{ t('studio.googlePreview') }}
                </span>
                <span>{{ t('studio.desktopSerp') }}</span>
              </div>
              <div class="studio-serp-card">
                <div class="studio-serp-brand">
                  <span class="studio-serp-avatar">
                    <img v-if="brandLogo" :src="brandLogo" alt="" />
                    <template v-else>{{ brandInitial }}</template>
                  </span>
                  <span>
                    <strong>{{ brandName }}</strong>
                    <small>{{ serpHost }} › read › {{ draft.slug || 'untitled' }}</small>
                  </span>
                </div>
                <h4>{{ serpTitle }}</h4>
                <p><span>{{ formatStamp(content.updatedAt) }} — </span>{{ serpDescription }}</p>
              </div>
            </div>
          </div>
        </div>
      </div>
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

<style scoped>
.studio-editor {
  container-type: inline-size;
  --studio-line: color-mix(in srgb, var(--ink) 12%, transparent);
  --studio-faint: color-mix(in srgb, var(--ink) 7%, transparent);
  --studio-wash: var(--wash, #f7f7f8);
  --studio-ok: var(--accent, #0a0a0a);
  --studio-warn: var(--warning, #b45309);
  display: flex;
  flex-direction: column;
  min-width: 0;
  color: var(--ink);
}

.studio-cmdbar {
  position: sticky;
  top: 0;
  z-index: 25;
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
  margin: -16px -16px 18px;
  padding: 9px 16px;
  border-bottom: 1px solid var(--studio-line);
  background: color-mix(in srgb, var(--surface) 98%, transparent);
  backdrop-filter: blur(14px);
  -webkit-backdrop-filter: blur(14px);
}

@media (min-width: 768px) {
  .studio-cmdbar {
    top: -24px;
    margin: -24px -24px 20px;
    padding: 9px 24px;
  }
}

@media (max-width: 767px) {
  .studio-cmdbar {
    top: -16px;
  }
}

.studio-cmd-left,
.studio-cmd-actions {
  display: flex;
  align-items: center;
  flex-wrap: wrap;
  gap: 7px;
  min-width: 0;
}

.studio-cmd-actions {
  justify-content: flex-end;
  margin-left: auto;
}

.studio-crumb,
.studio-icon-btn,
.studio-text-btn,
.studio-view-tab,
.studio-pill-btn,
.studio-pill-link,
.studio-save-main,
.studio-save-caret,
.studio-inspector-tab,
.studio-tag button,
.studio-more-item,
.studio-publish-option {
  width: auto;
}

.studio-crumb {
  display: inline-flex;
  align-items: center;
  gap: 5px;
  padding: 4px 7px;
  margin-left: -7px;
  border: 0;
  border-radius: 6px;
  background: transparent;
  color: var(--muted);
  font: inherit;
  font-size: 12px;
  font-weight: 600;
  cursor: pointer;
}

.studio-crumb:hover {
  background: var(--studio-wash);
  color: var(--ink);
}

.studio-crumb-sep {
  color: var(--studio-line);
  font-size: 13px;
}

.studio-editor .statuschip {
  display: inline-flex;
  align-items: center;
  min-height: 22px;
  padding: 2px 9px;
  border: 1px solid transparent;
  border-radius: 9999px;
  background: var(--studio-wash);
  color: var(--muted);
  font-family: var(--font);
  font-size: 11px;
  font-weight: 650;
  line-height: 1.2;
  white-space: nowrap;
}

.studio-editor .statuschip.st-published {
  background: var(--studio-accent-wash);
  border-color: var(--studio-accent-line);
  color: var(--studio-accent);
}

.studio-editor .statuschip.st-draft {
  background: var(--warning-wash);
  border-color: color-mix(in srgb, var(--warning) 22%, transparent);
  color: var(--warning);
}

.studio-editor .statuschip.st-review {
  background: var(--studio-accent-wash);
  border-color: var(--studio-accent-line);
  color: var(--studio-accent);
}

.studio-editor .statuschip.st-scheduled {
  background: var(--studio-accent-wash);
  border-color: var(--studio-accent-line);
  color: var(--studio-accent);
}

.studio-editor .statuschip.st-archived {
  background: var(--studio-wash);
  border-color: var(--studio-line);
  color: var(--muted);
}

.studio-save-state {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  color: var(--muted);
  font-size: 11.5px;
  font-weight: 500;
  white-space: nowrap;
}

.studio-save-state svg {
  color: var(--studio-ok);
}

.studio-viewswitch {
  display: inline-flex;
  align-items: center;
  gap: 1px;
  padding: 2px;
  border: 1px solid var(--studio-line);
  border-radius: 9px;
  background: var(--studio-wash);
}

.studio-view-tab {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 5px;
  height: 27px;
  padding: 0 9px;
  border: 0;
  border-radius: 6px;
  background: transparent;
  color: var(--muted);
  font: inherit;
  font-size: 11.5px;
  font-weight: 550;
  cursor: pointer;
  transition: background 0.15s ease, color 0.15s ease, box-shadow 0.15s ease;
}

.studio-view-tab:hover {
  color: var(--ink);
}

.studio-view-tab.active {
  background: var(--surface);
  color: var(--ink);
  font-weight: 700;
  box-shadow: var(--shadow-sm);
}

.studio-icon-btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 5px;
  height: 31px;
  padding: 0 8px;
  border: 1px solid var(--studio-line);
  border-radius: 8px;
  background: var(--surface);
  color: var(--muted);
  font: inherit;
  font-size: 11.5px;
  font-weight: 600;
  cursor: pointer;
  box-shadow: var(--shadow-xs);
}

.studio-icon-btn:hover {
  background: var(--studio-wash);
  color: var(--ink);
}

.studio-btn-label {
  display: inline;
}

.studio-count {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  min-width: 18px;
  height: 17px;
  padding: 0 5px;
  border: 1px solid var(--studio-line);
  border-radius: 9999px;
  background: var(--studio-wash);
  color: var(--muted);
  font-family: var(--mono);
  font-size: 9.5px;
  font-weight: 700;
}

.studio-text-btn {
  height: 31px;
  padding: 0 9px;
  border: 1px solid var(--studio-line);
  border-radius: 8px;
  background: var(--surface);
  color: var(--ink);
  font: inherit;
  font-size: 11.5px;
  font-weight: 600;
  cursor: pointer;
}

.studio-more-btn {
  width: 31px;
  padding: 0;
}

.studio-save-split {
  position: relative;
  display: inline-flex;
  align-items: stretch;
}

.studio-save-main,
.studio-save-caret {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 5px;
  height: 31px;
  border: 1px solid var(--ink);
  background: var(--ink);
  color: var(--studio-accent-ink);
  font: inherit;
  font-size: 11.5px;
  font-weight: 650;
  cursor: pointer;
}

.studio-save-main {
  padding: 0 11px;
  border-radius: 8px 0 0 8px;
}

.studio-save-caret {
  width: 28px;
  padding: 0;
  border-left-color: color-mix(in srgb, var(--studio-accent-ink) 22%, transparent);
  border-radius: 0 8px 8px 0;
}

.studio-save-split.is-solo .studio-save-main {
  border-radius: 8px;
}

.studio-save-main:disabled,
.studio-save-caret:disabled {
  opacity: 0.38;
  cursor: not-allowed;
}

.studio-publish-menu,
.studio-more-menu {
  position: absolute;
  top: calc(100% + 6px);
  right: 0;
  z-index: 60;
  padding: 6px;
  border: 1px solid var(--studio-line);
  border-radius: 12px;
  background: var(--surface);
  box-shadow: var(--shadow-lg);
}

.studio-publish-menu {
  width: 280px;
}

.studio-more-menu {
  width: 194px;
}

.studio-menu-label {
  padding: 5px 7px 6px;
  color: var(--muted);
  font-size: 9.5px;
  font-weight: 750;
  letter-spacing: 0.09em;
  text-transform: uppercase;
}

.studio-publish-option {
  display: flex;
  align-items: flex-start;
  gap: 8px;
  width: 100%;
  padding: 7px;
  border: 0;
  border-radius: 8px;
  background: transparent;
  color: var(--ink);
  font: inherit;
  text-align: left;
  cursor: pointer;
}

.studio-publish-option:hover {
  background: var(--studio-wash);
}

.studio-publish-option:disabled {
  opacity: 0.45;
  cursor: not-allowed;
}

.studio-publish-option > span:last-child {
  display: grid;
  gap: 1px;
}

.studio-publish-option strong {
  font-size: 11.5px;
  font-weight: 650;
}

.studio-publish-option small {
  color: var(--muted);
  font-size: 10px;
  line-height: 1.35;
}

.studio-option-dot {
  width: 7px;
  height: 7px;
  margin-top: 4px;
  border-radius: 50%;
  background: var(--muted);
  flex: none;
}

.studio-option-dot.is-live {
  background: var(--studio-accent);
}

.studio-publish-option.is-danger,
.studio-more-item.is-danger {
  color: var(--danger);
}

.studio-schedule-block {
  display: grid;
  gap: 6px;
  margin-top: 4px;
  padding: 8px 7px 5px;
  border-top: 1px solid var(--studio-faint);
}

.studio-schedule-block .studio-field-label {
  margin: 0;
}

.studio-schedule-block .studio-input {
  min-height: 30px;
  font-size: 11px;
}

.studio-schedule-block .studio-small-btn {
  justify-self: start;
}

.studio-more-item {
  display: flex;
  align-items: center;
  gap: 7px;
  width: 100%;
  padding: 7px 8px;
  border: 0;
  border-radius: 7px;
  background: transparent;
  color: var(--ink);
  font: inherit;
  font-size: 11.5px;
  font-weight: 550;
  text-align: left;
  cursor: pointer;
}

.studio-more-item:hover {
  background: var(--studio-wash);
}

.studio-menu-divider {
  height: 1px;
  margin: 4px;
  background: var(--studio-faint);
}

.studio-small-btn {
  width: auto;
  min-height: 28px;
  padding: 0 9px;
  font-size: 11px;
}

.studio-alert {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 8px;
  margin: 0 0 10px;
  padding: 8px 11px;
  border: 1px solid var(--studio-line);
  border-radius: 8px;
  font-size: 11.5px;
}

.studio-alert.is-warning {
  border-color: color-mix(in srgb, var(--warning) 22%, transparent);
  background: var(--warning-wash);
  color: var(--warning);
}

.studio-alert.is-danger {
  border-color: color-mix(in srgb, var(--danger) 22%, transparent);
  background: var(--danger-wash);
  color: var(--danger);
}

.studio-alert span {
  flex: 1;
  min-width: 160px;
}

.studio-notice {
  margin: 0 0 10px;
  color: var(--muted);
  font-size: 11.5px;
}

.studio-doc-head {
  display: flex;
  flex-direction: column;
  gap: 11px;
  margin-bottom: 18px;
}

.studio-url-row {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
}

.studio-url-pill {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  min-width: 0;
  max-width: 100%;
  padding: 4px 6px 4px 10px;
  border: 1px solid var(--studio-line);
  border-radius: 9999px;
  background: var(--surface);
  box-shadow: var(--shadow-xs);
}

.studio-url-state {
  display: inline-flex;
  align-items: center;
  gap: 5px;
  color: var(--muted);
  font-size: 9.5px;
  font-weight: 750;
  letter-spacing: 0.08em;
  text-transform: uppercase;
  white-space: nowrap;
}

.studio-url-dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background: var(--muted);
}

.studio-url-dot.live {
  background: var(--studio-accent);
}

.studio-url-sep {
  color: var(--studio-line);
}

.studio-url-pill code {
  max-width: 360px;
  overflow: hidden;
  color: var(--muted);
  font-family: var(--mono);
  font-size: 10.5px;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.studio-pill-btn,
.studio-pill-link {
  display: inline-flex;
  align-items: center;
  gap: 3px;
  padding: 2px 8px;
  border: 0;
  border-radius: 9999px;
  background: var(--studio-wash);
  color: var(--ink);
  font: inherit;
  font-size: 10.5px;
  font-weight: 600;
  text-decoration: none;
  cursor: pointer;
  white-space: nowrap;
}

.studio-pill-btn:hover,
.studio-pill-link:hover {
  background: var(--studio-wash);
}

.studio-doc-meta {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 4px;
  color: var(--muted);
  font-size: 11.5px;
}

.studio-doc-meta strong {
  color: var(--ink);
  font-weight: 600;
}

.studio-title-card {
  display: flex;
  flex-direction: column;
  gap: 4px;
  padding: 13px 17px;
  border: 1px solid var(--studio-line);
  border-radius: 12px;
  background: var(--surface);
  box-shadow: var(--shadow-xs);
}

.studio-title-card:focus-within {
  border-color: var(--studio-accent-line);
}

.studio-title-input,
.studio-subtitle-input {
  width: 100%;
  padding: 0;
  border: 0;
  background: transparent;
  outline: none;
  font-family: var(--font);
  box-shadow: none;
}

.studio-title-input {
  color: var(--ink) !important;
  font-family: var(--font) !important;
  font-size: 27px !important;
  font-weight: 720 !important;
  line-height: 1.2 !important;
  letter-spacing: -0.035em;
  background: transparent !important;
  border: 0 !important;
  border-radius: 0 !important;
  padding: 0 !important;
  box-shadow: none !important;
}

.studio-subtitle-input {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  color: var(--muted) !important;
  font-family: var(--font) !important;
  font-size: 13.5px !important;
  font-weight: 450 !important;
  line-height: 1.45 !important;
  background: transparent !important;
  border: 0 !important;
  border-radius: 0 !important;
  padding: 0 !important;
  box-shadow: none !important;
}

.studio-title-input::placeholder,
.studio-subtitle-input::placeholder {
  color: var(--muted);
}

.studio-title-input:disabled,
.studio-subtitle-input:disabled {
  opacity: 0.6;
}

.studio-workspace {
  overflow: hidden;
  border: 1px solid var(--studio-line);
  border-radius: 13px;
  background: var(--surface);
  box-shadow: var(--shadow-sm);
}

.studio-toolbar {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
  padding: 6px 10px;
  border-bottom: 1px solid var(--studio-line);
  background: var(--studio-wash);
}

.studio-tools {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 2px;
}

.studio-editor .studio-tool {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: auto;
  min-width: 28px;
  height: 28px;
  padding: 0 6px;
  border: 1px solid transparent;
  border-radius: 6px;
  background: transparent;
  color: var(--ink);
  font-size: 11.5px;
  cursor: pointer;
}

.studio-editor .studio-tool:hover {
  border-color: var(--studio-line);
  background: var(--surface);
  box-shadow: var(--shadow-xs);
}

.studio-editor .studio-tool:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}

.studio-tool.is-italic {
  font-family: Georgia, serif;
  font-style: italic;
}

.studio-tool.is-mono {
  font-family: var(--mono);
  font-size: 10.5px;
}

.studio-tool-sep {
  width: 1px;
  height: 15px;
  margin: 0 3px;
  background: var(--studio-line);
}

.studio-metrics {
  display: inline-flex;
  align-items: center;
  gap: 5px;
  padding: 3px 8px;
  border: 1px solid var(--studio-line);
  border-radius: 6px;
  background: var(--surface);
  color: var(--muted);
  font-family: var(--mono);
  font-size: 9.5px;
  white-space: nowrap;
}

.studio-metrics .is-strong {
  color: var(--ink);
  font-weight: 650;
}

.studio-panes {
  display: grid;
  grid-template-columns: minmax(0, 1fr);
  min-height: 440px;
}

.studio-panes.is-split {
  grid-template-columns: repeat(2, minmax(0, 1fr));
}

.studio-panes.is-split .studio-pane-preview {
  border-left: 1px solid var(--studio-line);
}

.studio-pane {
  display: flex;
  flex-direction: column;
  min-width: 0;
  min-height: 0;
  background: var(--surface);
}

.studio-pane-preview {
  background: var(--surface-2, #fbfbfc);
}

.studio-pane-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
  min-height: 32px;
  padding: 6px 14px;
  border-bottom: 1px solid var(--studio-faint);
  color: var(--muted);
  font-size: 9.5px;
  font-weight: 650;
  letter-spacing: 0.08em;
  text-transform: uppercase;
}

.studio-live-label {
  display: inline-flex;
  align-items: center;
  gap: 6px;
}

.studio-live-dot {
  width: 5px;
  height: 5px;
  border-radius: 50%;
  background: var(--studio-accent);
}

.studio-textarea {
  width: 100%;
  min-height: 380px;
  flex: 1;
  padding: 20px 22px;
  border: 0;
  border-radius: 0;
  background: transparent;
  color: var(--ink);
  font-family: var(--mono);
  font-size: 12.5px;
  line-height: 1.75;
  resize: vertical;
  outline: none;
  box-shadow: none;
}

.studio-textarea::placeholder {
  color: var(--muted);
}

.studio-linkbar {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 7px 10px;
  border-bottom: 1px solid var(--studio-faint);
  background: var(--studio-wash);
}

.studio-linkbar .studio-input {
  flex: 1;
  min-width: 0;
}

.studio-preview-article {
  width: 100%;
  max-width: 700px;
  margin: 0 auto;
  padding: 24px 28px;
  overflow-y: auto;
}

.studio-preview-tags {
  display: flex;
  flex-wrap: wrap;
  gap: 5px;
  margin-bottom: 10px;
}

.studio-preview-tags span {
  padding: 2px 7px;
  border: 1px solid var(--studio-line);
  border-radius: 5px;
  background: var(--studio-wash);
  color: var(--muted);
  font-family: var(--mono);
  font-size: 9.5px;
  font-weight: 600;
  letter-spacing: 0.05em;
  text-transform: uppercase;
}

.studio-preview-hero {
  overflow: hidden;
  margin-bottom: 18px;
  border: 1px solid var(--studio-line);
  border-radius: 10px;
  background: var(--studio-wash);
  aspect-ratio: 16 / 9;
}

.studio-preview-hero img {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.studio-preview-article h1 {
  display: block !important;
  margin: 0 0 6px;
  color: var(--ink);
  font-family: var(--font);
  font-size: 26px;
  font-weight: 750;
  line-height: 1.15;
  letter-spacing: -0.03em;
}

.studio-preview-dek {
  margin: 0 0 18px;
  padding-bottom: 14px;
  border-bottom: 1px solid var(--studio-line);
  color: var(--muted);
  font-family: var(--font-display);
  font-size: 16px;
  font-style: italic;
  line-height: 1.55;
}

.studio-preview-article :deep(.markdown) {
  color: var(--muted);
  font-size: 13.5px;
  line-height: 1.75;
}

.studio-preview-article :deep(.markdown h1),
.studio-preview-article :deep(.markdown h2),
.studio-preview-article :deep(.markdown h3) {
  color: var(--ink);
  font-family: var(--font);
  text-transform: none;
  letter-spacing: -0.02em;
  margin: 20px 0 8px;
}

.studio-preview-article :deep(.markdown h1) { font-size: 22px; }
.studio-preview-article :deep(.markdown h2) { font-size: 18px; }
.studio-preview-article :deep(.markdown h3) { font-size: 15px; }
.studio-preview-article :deep(.markdown p) { margin: 0 0 12px; }
.studio-preview-article :deep(.markdown a) { color: var(--studio-accent); text-decoration: underline; text-underline-offset: 3px; }
.studio-preview-article :deep(.markdown code) {
  padding: 1px 4px;
  border: 1px solid var(--studio-line);
  border-radius: 4px;
  background: var(--studio-wash);
  font-family: var(--mono);
  font-size: 0.9em;
}
.studio-preview-article :deep(.markdown pre) {
  overflow-x: auto;
  padding: 12px;
  border-radius: 7px;
  background: var(--ink);
  color: var(--surface);
}
.studio-preview-article :deep(.markdown pre code) {
  border: 0;
  background: transparent;
  color: inherit;
  padding: 0;
}
.studio-preview-article :deep(.markdown blockquote) {
  margin: 14px 0;
  padding: 10px 12px;
  border-left: 3px solid var(--ink);
  border-radius: 0 7px 7px 0;
  background: var(--studio-wash);
}

.studio-inspector {
  margin-top: 18px;
  scroll-margin-top: 56px;
  overflow: hidden;
  border: 1px solid var(--studio-line);
  border-radius: 13px;
  background: var(--surface);
  box-shadow: var(--shadow-sm);
}

.studio-inspector-head {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  justify-content: space-between;
  gap: 10px;
  padding: 12px 16px;
  border-bottom: 1px solid var(--studio-line);
  background: var(--studio-wash);
}

.studio-inspector-title {
  display: flex;
  align-items: center;
  gap: 9px;
  min-width: 0;
}

.studio-inspector-icon {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 29px;
  height: 29px;
  flex: none;
  border-radius: 8px;
  background: var(--accent-wash);
  color: var(--accent);
}

.studio-inspector-title h3 {
  margin: 0;
  color: var(--ink);
  font-family: var(--font);
  font-size: 13px;
  font-weight: 700;
  letter-spacing: -0.01em;
}

.studio-inspector-title p {
  margin: 1px 0 0;
  color: var(--muted);
  font-size: 10.5px;
  line-height: 1.35;
}

.studio-inspector-tabs {
  display: inline-flex;
  align-items: center;
  gap: 1px;
  padding: 2px;
  border: 1px solid var(--studio-line);
  border-radius: 8px;
  background: var(--studio-wash);
}

.studio-inspector-tab {
  height: 26px;
  padding: 0 8px;
  border: 0;
  border-radius: 5px;
  background: transparent;
  color: var(--muted);
  font: inherit;
  font-size: 10.5px;
  font-weight: 550;
  cursor: pointer;
}

.studio-inspector-tab.active {
  background: var(--surface);
  color: var(--ink);
  font-weight: 650;
  box-shadow: var(--shadow-xs);
}

.studio-inspector-body {
  display: grid;
  gap: 18px;
  padding: 16px;
}

.studio-general,
.studio-seo {
  display: grid;
  gap: 16px;
}

.studio-settings-row {
  display: grid;
  gap: 12px;
}

.studio-settings-row.is-three {
  grid-template-columns: minmax(0, 1.25fr) minmax(0, 0.8fr) minmax(0, 1.4fr);
}

.studio-settings-row.is-tags-hero {
  grid-template-columns: minmax(0, 1.1fr) minmax(0, 1.3fr);
  align-items: start;
}

.studio-field {
  display: flex;
  flex-direction: column;
  gap: 5px;
  min-width: 0;
}

.studio-field-label {
  color: var(--ink);
  font-size: 10.5px;
  font-weight: 600;
}

.studio-label-row {
  display: flex;
  align-items: baseline;
  justify-content: space-between;
  gap: 8px;
}

.studio-counter {
  color: var(--muted);
  font-family: var(--mono);
  font-size: 9.5px;
  font-weight: 500;
}

.studio-counter.is-ok {
  color: var(--studio-ok);
}

.studio-counter.is-warn {
  color: var(--studio-warn);
}

.studio-input {
  width: 100%;
  min-height: 32px;
  padding: 5px 8px !important;
  border: 1px solid var(--studio-line) !important;
  border-radius: 7px !important;
  background: var(--surface) !important;
  color: var(--ink) !important;
  font-family: var(--font) !important;
  font-size: 11.5px !important;
  line-height: 1.4 !important;
  outline: none;
  box-shadow: none !important;
}

.studio-input:focus {
  border-color: var(--studio-accent-line);
  box-shadow: 0 0 0 2px var(--studio-accent-wash);
}

.studio-textarea-sm {
  min-height: 52px;
  resize: vertical;
}

.studio-prefix-input {
  display: flex;
  align-items: stretch;
  overflow: hidden;
  border: 1px solid var(--studio-line);
  border-radius: 7px;
  background: var(--surface);
}

.studio-prefix-input:focus-within {
  border-color: var(--studio-accent-line);
  box-shadow: 0 0 0 2px var(--studio-accent-wash);
}

.studio-prefix-input > span {
  display: inline-flex;
  align-items: center;
  padding: 0 7px;
  border-right: 1px solid var(--studio-line);
  background: var(--studio-wash);
  color: var(--muted);
  font-family: var(--mono);
  font-size: 10px;
  white-space: nowrap;
}

.studio-prefix-input input {
  min-width: 0;
  flex: 1;
  padding: 5px 8px !important;
  border: 0 !important;
  border-radius: 0 !important;
  background: transparent !important;
  color: var(--ink) !important;
  font-family: var(--mono) !important;
  font-size: 11px !important;
  outline: none;
  box-shadow: none !important;
}

.studio-tagbox {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 5px;
  min-height: 36px;
  padding: 4px 5px;
  border: 1px solid var(--studio-line);
  border-radius: 7px;
  background: var(--surface);
}

.studio-tagbox:focus-within {
  border-color: var(--studio-accent-line);
  box-shadow: 0 0 0 2px var(--studio-accent-wash);
}

.studio-tag {
  display: inline-flex;
  align-items: center;
  gap: 3px;
  padding: 2px 6px;
  border: 1px solid var(--studio-accent-line);
  border-radius: 5px;
  background: var(--studio-accent-wash);
  color: var(--studio-accent);
  font-family: var(--mono);
  font-size: 9.5px;
  font-weight: 600;
}

.studio-tag button {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 13px;
  height: 13px;
  padding: 0;
  border: 0;
  background: transparent;
  color: currentColor;
  font-size: 13px;
  line-height: 1;
  cursor: pointer;
}

.studio-tag-input {
  min-width: 90px;
  flex: 1;
  padding: 2px 4px !important;
  border: 0 !important;
  border-radius: 0 !important;
  background: transparent !important;
  color: var(--ink) !important;
  font-family: var(--font) !important;
  font-size: 11px !important;
  outline: none;
  box-shadow: none !important;
}

.studio-hero-input-row {
  display: flex;
  align-items: center;
  gap: 6px;
}

.studio-hero-input-row .studio-input {
  flex: 1;
}

.studio-hero-preview {
  display: flex;
  align-items: center;
  justify-content: center;
  height: 128px;
  overflow: hidden;
  border: 1px dashed var(--studio-line);
  border-radius: 8px;
  background: var(--studio-wash);
}

.studio-hero-preview img {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.studio-hero-empty {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 3px;
  padding: 12px;
  color: var(--muted);
  text-align: center;
}

.studio-hero-empty strong {
  color: var(--muted);
  font-size: 10.5px;
  font-weight: 600;
}

.studio-hero-empty span {
  font-size: 9.5px;
}

.studio-seo-intro h3 {
  margin: 0;
  color: var(--ink);
  font-family: var(--font);
  font-size: 10.5px;
  font-weight: 750;
  letter-spacing: 0.06em;
  text-transform: uppercase;
}

.studio-seo-intro p {
  margin: 2px 0 0;
  color: var(--muted);
  font-size: 10.5px;
}

.studio-seo-grid {
  display: grid;
  grid-template-columns: minmax(0, 1fr) minmax(0, 1fr);
  gap: 18px;
  align-items: start;
}

.studio-seo-fields {
  display: grid;
  gap: 12px;
}

.studio-meter {
  height: 3px;
  overflow: hidden;
  border-radius: 9999px;
  background: var(--studio-wash);
}

.studio-meter > span {
  display: block;
  height: 100%;
  border-radius: inherit;
  transition: width 0.18s ease;
}

.studio-meter > span.is-ok {
  background: var(--studio-accent);
}

.studio-meter > span.is-warn {
  background: var(--studio-warn);
}

.studio-serp {
  display: grid;
  min-width: 0;
  gap: 9px;
  padding: 13px;
  border: 1px solid var(--studio-line);
  border-radius: 10px;
  background: var(--studio-wash);
}

.studio-serp-head {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  justify-content: space-between;
  gap: 4px 8px;
  min-width: 0;
  color: var(--muted);
  font-size: 9.5px;
  font-weight: 650;
  letter-spacing: 0.06em;
  text-transform: uppercase;
}

.studio-serp-head > span:first-child {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  color: var(--studio-accent);
}

.studio-serp-card {
  display: flex;
  flex-direction: column;
  gap: 3px;
  min-width: 0;
  padding: 12px;
  border: 1px solid var(--studio-line);
  border-radius: 8px;
  background: var(--surface);
  box-shadow: var(--shadow-xs);
}

.studio-serp-brand {
  display: flex;
  align-items: center;
  gap: 7px;
}

.studio-serp-avatar {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 21px;
  height: 21px;
  flex: none;
  border-radius: 50%;
  background: var(--studio-accent);
  color: var(--studio-accent-ink);
  font-size: 9px;
  font-weight: 700;
}

.studio-serp-brand > span:last-child {
  display: flex;
  flex-direction: column;
  min-width: 0;
  line-height: 1.25;
}

.studio-serp-brand strong {
  overflow: hidden;
  color: var(--ink);
  font-size: 10.5px;
  font-weight: 600;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.studio-serp-brand small {
  overflow: hidden;
  color: var(--muted);
  font-size: 9.5px;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.studio-serp-card h4 {
  margin: 4px 0 0;
  color: var(--studio-accent);
  font-family: var(--font);
  font-size: 15px;
  font-weight: 500;
  line-height: 1.3;
}

.studio-serp-card p {
  margin: 0;
  display: -webkit-box;
  overflow: hidden;
  color: var(--muted);
  font-size: 11px;
  line-height: 1.5;
  -webkit-box-orient: vertical;
  -webkit-line-clamp: 2;
}

.studio-serp-card p span {
  color: var(--muted);
}

@media (max-width: 1220px) {
  .studio-btn-label {
    display: none;
  }
}

@container (max-width: 680px) {
  .studio-panes.is-split {
    grid-template-columns: minmax(0, 1fr);
  }

  .studio-panes.is-split .studio-pane-preview {
    border-top: 1px solid var(--studio-line);
    border-left: 0;
  }
}

@media (max-width: 900px) {
  .studio-panes.is-split {
    grid-template-columns: minmax(0, 1fr);
  }

  .studio-panes.is-split .studio-pane-preview {
    border-top: 1px solid var(--studio-line);
    border-left: 0;
  }

  .studio-settings-row.is-three,
  .studio-settings-row.is-tags-hero,
  .studio-seo-grid {
    grid-template-columns: minmax(0, 1fr);
  }
}

@media (min-width: 681px) and (max-width: 1220px) {
  .studio-inspector {
    scroll-margin-top: 100px;
  }
}

@media (max-width: 680px) {
  .studio-cmdbar {
    align-items: flex-start;
  }

  .studio-inspector {
    scroll-margin-top: 128px;
  }

  .studio-viewswitch {
    order: 3;
    width: 100%;
  }

  .studio-view-tab {
    flex: 1;
  }

  .studio-cmd-actions {
    gap: 5px;
  }

  .studio-editor .studio-icon-btn {
    height: 29px;
    padding: 0 6px;
  }

  .studio-url-pill code {
    max-width: 160px;
  }

  .studio-doc-meta {
    width: 100%;
  }

  .studio-title-input {
    font-size: 20px !important;
  }

  .studio-subtitle-input {
    font-size: 12.5px !important;
  }

  .studio-preview-article {
    padding: 20px 18px;
  }

  .studio-inspector-head,
  .studio-inspector-body {
    padding-left: 12px;
    padding-right: 12px;
  }

  .studio-inspector-tabs {
    width: 100%;
  }

  .studio-inspector-tab {
    flex: 1;
    padding: 0 5px;
    font-size: 9.5px;
  }

  .studio-seo,
  .studio-seo-grid,
  .studio-serp {
    min-width: 0;
  }

  .studio-serp-head > span:last-child {
    display: none;
  }
}
</style>

<style scoped>
/* Brand CI bridge: the Studio follows the workspace palette, typeface and
   corner-radius tokens instead of the old hardcoded black/white chrome. */
.studio-editor {
  --studio-accent: var(--accent, #0a0a0a);
  --studio-accent-ink: var(--accent-ink, #ffffff);
  --studio-accent-wash: var(--accent-wash, #f4f5f6);
  --studio-accent-line: color-mix(in srgb, var(--studio-accent) 26%, transparent);
  --studio-radius: var(--radius-sm, 8px);
  --studio-radius-lg: var(--radius, 12px);
}

.studio-save-main,
.studio-save-caret {
  background: var(--studio-accent);
  border-color: var(--studio-accent);
  color: var(--studio-accent-ink);
}

.studio-save-main {
  border-radius: var(--studio-radius) 0 0 var(--studio-radius);
}

.studio-save-caret {
  border-radius: 0 var(--studio-radius) var(--studio-radius) 0;
}

.studio-save-split.is-solo .studio-save-main {
  border-radius: var(--studio-radius);
}

.studio-live-dot,
.studio-serp-avatar {
  background: var(--studio-accent);
  color: var(--studio-accent-ink);
}

.studio-title-card,
.studio-workspace,
.studio-inspector,
.studio-publish-menu,
.studio-more-menu {
  border-radius: var(--studio-radius-lg);
}

.studio-input,
.studio-prefix-input,
.studio-tagbox,
.studio-small-btn,
.studio-pill-btn,
.studio-pill-link,
.studio-viewswitch,
.studio-view-tab,
.studio-icon-btn,
.studio-text-btn,
.studio-inspector-tabs,
.studio-inspector-tab,
.studio-publish-option,
.studio-more-item,
.studio-tag,
.studio-preview-tags span,
.studio-metrics,
.studio-hero-preview,
.studio-alert,
.studio-serp,
.studio-serp-card,
.studio-preview-hero,
.studio-tool {
  border-radius: var(--studio-radius);
}

.studio-input {
  border-radius: var(--studio-radius) !important;
}

.studio-prefix-input > span {
  border-radius: var(--studio-radius) 0 0 var(--studio-radius);
}

.studio-tag {
  border-color: var(--studio-accent-line);
  background: var(--studio-accent-wash);
  color: var(--studio-accent);
}

.studio-preview-tags span {
  border-color: var(--studio-accent-line);
  background: var(--studio-accent-wash);
  color: var(--studio-accent);
}

.studio-title-card:focus-within {
  border-color: var(--studio-accent-line);
}

.studio-input:focus,
.studio-prefix-input:focus-within,
.studio-tagbox:focus-within {
  border-color: var(--studio-accent-line) !important;
  box-shadow: 0 0 0 2px var(--studio-accent-wash) !important;
}

.studio-serp-avatar {
  /* Google SERP avatars look best as a solid brand mark. */
  border-radius: 50%;
}

.studio-serp-avatar img {
  width: 100%;
  height: 100%;
  padding: 2px;
  border-radius: inherit;
  background: var(--surface);
  object-fit: contain;
}

.studio-preview-dek {
  font-family: var(--font);
  font-style: italic;
}

.studio-preview-article :deep(.markdown a) {
  color: var(--studio-accent);
}

.studio-preview-article :deep(.markdown blockquote) {
  border-left-color: var(--studio-accent);
}
</style>
