<script setup lang="ts">
import { computed, ref } from 'vue'
import { useIdeas, useAddIdea, useToggleIdea, usePromoteIdea, usePermission } from '@/core/queries'
import type { Idea } from '@/mock/db'
import { t } from '@/core/i18n'
import { activeMonth } from '@/core/navdate'
import MonthToggle from '@/components/ui/MonthToggle.vue'

const { data: ideas, isPending } = useIdeas()
const addIdea = useAddIdea()
const toggleIdea = useToggleIdea()
const promoteMonth = activeMonth
const promote = usePromoteIdea(promoteMonth)
const { can } = usePermission()
const canWrite = computed(() => can('ideas.write'))

const topic = ref('')
const format = ref('')
const idea = ref('')
const link = ref('')
const addError = ref('')
const actionError = ref('')
const notice = ref('')

async function add(): Promise<void> {
  if (!canWrite.value || addIdea.isPending.value || !topic.value.trim()) return
  addError.value = ''
  try {
    await addIdea.mutateAsync({
      topic: topic.value.trim(), format: format.value.trim(),
      idea: idea.value.trim(), link: link.value.trim(), done: false,
    })
    topic.value = ''
    format.value = ''
    idea.value = ''
    link.value = ''
  } catch (e) {
    addError.value = e instanceof Error ? e.message : String(e)
  }
}

async function onToggle(it: Idea, event: Event): Promise<void> {
  if (toggleIdea.isPending.value) return
  actionError.value = ''
  try {
    await toggleIdea.mutateAsync(it.id)
  } catch (e) {
    // put the checkbox back to the authoritative value
    ;(event.target as HTMLInputElement).checked = it.done
    actionError.value = e instanceof Error ? e.message : String(e)
  }
}

async function onPromote(it: Idea): Promise<void> {
  if (promote.isPending.value) return
  actionError.value = ''
  notice.value = ''
  try {
    await promote.mutateAsync(it.id)
    notice.value = `Promoted “${it.topic}” to month ${promoteMonth.value}.`
  } catch (e) {
    actionError.value = e instanceof Error ? e.message : String(e)
  }
}
</script>

<template>
  <h1>Idea Bank</h1>
  <p v-if="notice" class="muted" role="status">{{ notice }}</p>
  <p v-if="actionError" class="autherr" role="alert">{{ actionError }}</p>
  <div v-if="isPending" class="muted">Loading ideas…</div>
  <div v-else class="tblwrap">
    <table class="tbl">
      <thead><tr><th>Topic</th><th>Format</th><th>Idea</th><th>Link</th><th>Done</th><th></th></tr></thead>
      <tbody>
        <tr v-for="it in ideas" :key="it.id" style="cursor: default;">
          <td>{{ it.topic }}</td>
          <td>{{ it.format }}</td>
          <td>{{ it.idea }}</td>
          <td class="muted">{{ it.link || '—' }}</td>
          <td><input type="checkbox" :checked="it.done" :disabled="!canWrite || toggleIdea.isPending.value"
            :aria-label="`Mark ${it.topic} done`" :title="canWrite ? '' : t('auth.noPerm')"
            @change="onToggle(it, $event)" /></td>
          <td><button class="btn" :disabled="!canWrite || promote.isPending.value"
            :title="canWrite ? '' : t('auth.noPerm')" @click="onPromote(it)">Promote</button></td>
        </tr>
      </tbody>
    </table>
  </div>
  <h2>Capture</h2>
  <div class="grid3">
    <div><label class="lbl" for="idea-topic">Topic</label><input id="idea-topic" class="field" v-model="topic"
      :disabled="!canWrite" :title="canWrite ? '' : t('auth.noPerm')" /></div>
    <div><label class="lbl" for="idea-format">Format</label><input id="idea-format" class="field" v-model="format"
      :disabled="!canWrite" :title="canWrite ? '' : t('auth.noPerm')" /></div>
    <div>
      <label class="lbl">Promote to month</label>
      <MonthToggle v-model="promoteMonth" :disabled="!canWrite" />
    </div>
  </div>
  <label class="lbl" for="idea-idea">Idea</label><input id="idea-idea" class="field" v-model="idea" :disabled="!canWrite"
    :title="canWrite ? '' : t('auth.noPerm')" />
  <label class="lbl" for="idea-link">Link</label><input id="idea-link" class="field" v-model="link" :disabled="!canWrite"
    :title="canWrite ? '' : t('auth.noPerm')" />
  <p v-if="addError" class="autherr" role="alert">{{ addError }}</p>
  <div class="mt"><button class="btn btn-primary" :disabled="!canWrite || addIdea.isPending.value || !topic.trim()"
    :title="canWrite ? '' : t('auth.noPerm')" @click="add">Add idea</button></div>
</template>
