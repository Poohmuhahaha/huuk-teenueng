<script setup lang="ts">
import { ref } from 'vue'
import type { HashtagGroup } from '@/mock/db'
import Chip from './Chip.vue'

const props = defineProps<{
  groups: HashtagGroup[]
  /** Async add handler; the draft clears only when it resolves. */
  add?: (groupId: string, tag: string) => Promise<void>
}>()

const drafts = ref<Record<string, string>>({})
const error = ref('')
const busy = ref(false)

async function submit(groupId: string): Promise<void> {
  const value = (drafts.value[groupId] ?? '').trim().replace(/^#/, '')
  if (!value || !props.add || busy.value) return
  error.value = ''
  busy.value = true
  try {
    await props.add(groupId, value)
    drafts.value[groupId] = ''
  } catch (e) {
    error.value = e instanceof Error ? e.message : String(e)
  } finally {
    busy.value = false
  }
}
</script>

<template>
  <div class="grid2">
    <div v-for="g in groups" :key="g.id" class="panel">
      <strong>{{ g.title }}</strong>
      <div class="mt">
        <Chip v-for="(t, i) in g.tags" :key="`${t}-${i}`" :label="`#${t}`" />
      </div>
      <div class="row mt">
        <input class="field" style="flex: 1;" :value="drafts[g.id] ?? ''" placeholder="new tag"
          :aria-label="`New tag for ${g.title}`" :disabled="!add || busy"
          @input="drafts[g.id] = ($event.target as HTMLInputElement).value"
          @keyup.enter="submit(g.id)" />
        <button class="btn" :disabled="!add || busy" @click="submit(g.id)">Add</button>
      </div>
    </div>
  </div>
  <p v-if="error" class="autherr" role="alert">{{ error }}</p>
</template>
