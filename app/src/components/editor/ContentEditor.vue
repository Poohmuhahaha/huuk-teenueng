<script setup lang="ts">
// Client-facing editor: markdown body with live preview, autosave, revisions,
// scheduled publishing and SEO fields. Optimistic concurrency via `version`.
import { computed, nextTick, onBeforeUnmount, ref, watch } from 'vue'
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
onBeforeUnmount(() => window.clearTimeout(autosaveTimer))

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
  loadedVersion.value = serverVersion.value
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

function insertLink(): void {
  const url = window.prompt(t('studio.linkPrompt'), 'https://')
  if (!url) return
  wrapSelection('[', `](${url})`, t('studio.linkText'))
}

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
  <div v-else-if="itemQ.isError.value" class="card">
    <p class="muted">{{ itemQ.error.value?.message }}</p>
    <button class="btn" @click="() => itemQ.refetch()">{{ t('common.tryAgain') }}</button>
  </div>
  <div v-else-if="content && draft" class="editor">
    <header class="editor-head">
      <input v-model="draft.title" class="field editor-title" :disabled="!canEdit"
        :aria-label="t('studio.field.title')" :placeholder="t('studio.untitled')" />
      <div class="editor-head-right">
        <span class="statuschip" :class="`st-${status}`">{{ statusLabel(status) }}</span>
        <span class="muted editor-savestate">
          <template v-if="saving">{{ t('studio.saving') }}</template>
          <template v-else-if="conflict">{{ t('studio.conflictShort') }}</template>
          <template v-else-if="dirty">{{ t('studio.unsaved') }}</template>
          <template v-else>{{ t('studio.saved') }}</template>
        </span>
      </div>
    </header>

    <div class="editor-actions">
      <button class="btn btn-primary" :disabled="!canEdit || saving || !dirty || conflict" @click="save()">
        {{ t('studio.saveDraft') }}
      </button>
      <template v-if="canPublish">
        <div class="publish-wrap">
          <button class="btn" :disabled="!canEdit" @click="publishOpen = !publishOpen">
            {{ status === 'published' ? '✓ ' : '' }}{{ t('studio.publish') }} ▾
          </button>
          <div v-if="publishOpen" class="publish-menu" role="dialog" :aria-label="t('studio.publish')">
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
      <span class="spacer" />
      <div class="viewtoggle" role="tablist" :aria-label="t('studio.view')">
        <button class="tab" :class="{ active: mode === 'edit' }" role="tab" :aria-selected="mode === 'edit'"
          @click="mode = 'edit'">{{ t('studio.view.edit') }}</button>
        <button class="tab" :class="{ active: mode === 'split' }" role="tab" :aria-selected="mode === 'split'"
          @click="mode = 'split'">{{ t('studio.view.split') }}</button>
        <button class="tab" :class="{ active: mode === 'preview' }" role="tab" :aria-selected="mode === 'preview'"
          @click="mode = 'preview'">{{ t('studio.view.preview') }}</button>
      </div>
    </div>

    <div v-if="conflict" class="conflict" role="alert">
      <strong>{{ t('studio.conflict') }}</strong>
      <span class="muted">{{ t('studio.conflictHint') }}</span>
      <button class="btn" @click="reloadLatest">{{ t('studio.reloadLatest') }}</button>
      <button class="btn btn-primary" @click="keepMine">{{ t('studio.keepMine') }}</button>
    </div>
    <p v-if="saveError" class="autherr" role="alert">{{ saveError }}</p>
    <p v-if="notice" class="muted" role="status">{{ notice }}</p>

    <div v-if="status === 'published' && publishedUrl" class="card share">
      <strong>{{ t('studio.live') }}</strong>
      <code class="share-url">{{ publishedUrl }}</code>
      <button class="btn" @click="copyLink">{{ copied ? t('studio.copied') : t('studio.copyLink') }}</button>
      <RouterLink class="btn" :to="`/read/${content.slug}`" target="_blank">{{ t('studio.openLive') }}</RouterLink>
    </div>

    <div class="editor-grid" :class="`mode-${mode}`">
      <section v-show="mode !== 'preview'" class="editor-fields">
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

        <div class="body-toolbar" role="toolbar" :aria-label="t('studio.field.body')">
          <button type="button" class="toolbtn" :disabled="!canEdit" :title="t('studio.tool.bold')"
            @click="wrapSelection('**', '**', t('studio.tool.text'))"><strong>B</strong></button>
          <button type="button" class="toolbtn" :disabled="!canEdit" :title="t('studio.tool.italic')"
            @click="wrapSelection('*', '*', t('studio.tool.text'))"><em>I</em></button>
          <button type="button" class="toolbtn" :disabled="!canEdit" :title="t('studio.tool.h2')"
            @click="prefixLines('## ')">H2</button>
          <button type="button" class="toolbtn" :disabled="!canEdit" :title="t('studio.tool.list')"
            @click="prefixLines('- ')">•</button>
          <button type="button" class="toolbtn" :disabled="!canEdit" :title="t('studio.tool.ordered')"
            @click="prefixLines('1. ')">1.</button>
          <button type="button" class="toolbtn" :disabled="!canEdit" :title="t('studio.tool.quote')"
            @click="prefixLines('> ')">”</button>
          <button type="button" class="toolbtn" :disabled="!canEdit" :title="t('studio.tool.code')"
            @click="wrapSelection('`', '`', 'code')">&lt;/&gt;</button>
          <button type="button" class="toolbtn" :disabled="!canEdit" :title="t('studio.tool.link')"
            @click="insertLink">🔗</button>
          <span class="muted" style="margin-left: auto; font-size: 12px;">
            {{ wordCount }} {{ t('studio.words') }} · {{ readingMinutes }} {{ t('studio.minRead') }}
          </span>
        </div>
        <textarea id="studio-body" ref="bodyEl" v-model="draft.body" class="field editor-body"
          :disabled="!canEdit" rows="18" :aria-label="t('studio.field.body')" />

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

        <details class="seo">
          <summary>{{ t('studio.seo') }}</summary>
          <div class="grid2 mt">
            <div>
              <label class="lbl" for="studio-seo-title">{{ t('studio.field.seoTitle') }}</label>
              <input id="studio-seo-title" v-model="draft.seoTitle" class="field" :disabled="!canEdit" maxlength="200" />
              <span class="muted" style="font-size: 11px;">{{ draft.seoTitle.length }}/60</span>
            </div>
            <div>
              <label class="lbl" for="studio-seo-desc">{{ t('studio.field.seoDescription') }}</label>
              <input id="studio-seo-desc" v-model="draft.seoDescription" class="field" :disabled="!canEdit" maxlength="500" />
              <span class="muted" style="font-size: 11px;">{{ draft.seoDescription.length }}/160</span>
            </div>
          </div>
        </details>
      </section>

      <section v-show="mode !== 'edit'" class="editor-preview">
        <div class="preview-head">
          <strong>{{ t('studio.preview') }}</strong>
          <span class="muted">{{ t('studio.previewHint') }}</span>
        </div>
        <article class="preview-body">
          <h1 v-if="draft.title">{{ draft.title }}</h1>
          <p v-if="draft.excerpt" class="muted preview-excerpt">{{ draft.excerpt }}</p>
          <img v-if="draft.heroImageUrl" :src="draft.heroImageUrl" :alt="draft.title" class="preview-hero" />
          <!-- renderMarkdown escapes all text before inserting its own tags -->
          <div class="markdown" v-html="previewHtml" />
        </article>
      </section>
    </div>

    <!-- revisions drawer -->
    <aside v-if="revisionsOpen" class="revisions" role="dialog" :aria-label="t('studio.revisions')">
      <div class="revisions-head">
        <strong>{{ t('studio.revisions') }}</strong>
        <button class="btn" @click="revisionsOpen = false; revisionPreview = null">{{ t('common.close') }}</button>
      </div>
      <div v-if="revisionsQ.isPending.value" class="muted">{{ t('common.loading') }}</div>
      <p v-else-if="!revisions.length" class="muted">{{ t('studio.noRevisions') }}</p>
      <ul v-else class="revisions-list">
        <li v-for="revision in revisions" :key="revision.revision">
          <button class="studio-item" :class="{ on: revisionPreview?.revision === revision.revision }"
            @click="revisionPreview = revision">
            <strong>#{{ revision.revision }} · {{ revision.note || t('studio.edit') }}</strong>
            <span class="muted">{{ revision.author }} · {{ formatStamp(revision.savedAt) }}</span>
          </button>
        </li>
      </ul>
      <div v-if="revisionPreview" class="revision-preview">
        <h4 style="margin: 8px 0 4px;">{{ revisionPreview.title }}</h4>
        <div class="markdown" v-html="renderMarkdown(revisionPreview.body)" />
        <button class="btn btn-primary mt" :disabled="!canWrite" @click="restoreRevision(revisionPreview)">
          {{ t('studio.restoreThis') }}
        </button>
      </div>
    </aside>
  </div>
</template>

<style scoped>
.editor { display: grid; gap: 10px; }
.editor-head { display: flex; gap: 10px; align-items: center; }
.editor-title { font-size: 22px; font-weight: 700; }
.editor-head-right { display: flex; gap: 8px; align-items: center; white-space: nowrap; }
.editor-savestate { font-size: 12px; }
.editor-actions { display: flex; flex-wrap: wrap; gap: 6px; align-items: center; }
.spacer { flex: 1; }
.viewtoggle { display: flex; gap: 4px; }
.publish-wrap { position: relative; }
.publish-menu {
  position: absolute; z-index: 30; top: calc(100% + 6px); left: 0;
  background: var(--surface); border: 1px solid var(--faint); border-radius: var(--radius);
  padding: 8px; min-width: 240px; display: grid; gap: 4px; box-shadow: var(--shadow-lg);
}
.publish-menu .menubtn { text-align: left; }
.conflict {
  display: flex; flex-wrap: wrap; gap: 8px; align-items: center;
  border: 1px solid var(--warning); background: var(--warning-wash); border-radius: var(--radius-sm); padding: 8px 10px;
}
.share { display: flex; gap: 8px; align-items: center; flex-wrap: wrap; }
.share-url { font-size: 12px; overflow-wrap: anywhere; }
.editor-grid { display: grid; gap: 12px; align-items: start; }
.editor-grid.mode-split { grid-template-columns: 1fr 1fr; }
.editor-fields, .editor-preview { min-width: 0; }
.editor-body { font-family: ui-monospace, SFMono-Regular, Menlo, monospace; font-size: 14px; line-height: 1.6; }
.body-toolbar { display: flex; gap: 4px; align-items: center; margin: 8px 0 6px; }
.toolbtn {
  font: inherit; min-width: 32px; padding: 3px 7px;
  border: 1px solid var(--faint); border-radius: var(--radius-sm); background: var(--surface); cursor: pointer;
  color: var(--ink); transition: background 0.15s, border-color 0.15s;
}
.toolbtn:hover { background: var(--wash); border-color: var(--line); }
.toolbtn:disabled { opacity: 0.5; cursor: not-allowed; }
.seo { margin-top: 10px; }
.preview-head { display: flex; justify-content: space-between; gap: 8px; align-items: baseline; }
.preview-body {
  border: 1px solid var(--faint); border-radius: var(--radius);
  padding: 16px 18px; background: var(--surface); min-height: 200px;
}
.preview-excerpt { font-size: 15px; }
.preview-hero { max-width: 100%; border-radius: 8px; }
.revisions {
  position: fixed; top: 0; right: 0; bottom: 0; width: min(420px, 92vw); z-index: 60;
  background: var(--surface); border-left: 1px solid var(--faint); padding: 14px; overflow-y: auto;
  box-shadow: var(--shadow-lg);
}
.revisions-head { display: flex; justify-content: space-between; align-items: center; margin-bottom: 8px; }
.revisions-list { list-style: none; margin: 0; padding: 0; display: grid; gap: 6px; }
.revision-preview { border-top: 1px dashed var(--line); margin-top: 10px; }
@media (max-width: 900px) {
  .editor-grid.mode-split { grid-template-columns: 1fr; }
}
</style>
