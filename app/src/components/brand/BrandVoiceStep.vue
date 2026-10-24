<script setup lang="ts">
import { ref } from 'vue'
import { t } from '@/core/i18n'
import type { Brand } from '@/mock/db'

const props = defineProps<{
  brand: Brand
  canWrite: boolean
  busy: boolean
  hasCustomBg?: boolean
}>()

const emit = defineEmits<{
  (e: 'save', key: 'positioning' | 'slogan' | 'audience' | 'voice', event: Event): void
  (e: 'add-do', text: string): void
  (e: 'remove-do', index: number): void
  (e: 'add-dont', text: string): void
  (e: 'remove-dont', index: number): void
}>()

const doDraft = ref('')
const dontDraft = ref('')

function handleAddDo(): void {
  const val = doDraft.value.trim()
  if (!val || !props.canWrite || props.busy) return
  emit('add-do', val)
  doDraft.value = ''
}

function handleAddDont(): void {
  const val = dontDraft.value.trim()
  if (!val || !props.canWrite || props.busy) return
  emit('add-dont', val)
  dontDraft.value = ''
}
</script>

<template>
  <section class="brand-body brand-voice-step" :class="{ 'has-glass-bg': hasCustomBg }">
    <div class="brand-voice-layout">
      <!-- Left Column: Attributes -->
      <div class="brand-voice-fields">
        <div class="brand-field-group">
          <label class="lbl field-lbl" for="brand-positioning">POSITIONING</label>
          <input
            id="brand-positioning"
            class="field brand-input pill-input"
            :value="brand.positioning"
            :disabled="!canWrite || busy"
            :title="canWrite ? '' : t('auth.noPerm')"
            @change="emit('save', 'positioning', $event)"
          />
        </div>

        <div class="brand-field-group">
          <label class="lbl field-lbl" for="brand-slogan">SLOGAN (BIO)</label>
          <input
            id="brand-slogan"
            class="field brand-input pill-input"
            :value="brand.slogan"
            :disabled="!canWrite || busy"
            :title="canWrite ? '' : t('auth.noPerm')"
            @change="emit('save', 'slogan', $event)"
          />
        </div>

        <div class="brand-field-group">
          <label class="lbl field-lbl" for="brand-audience">AUDIENCE</label>
          <input
            id="brand-audience"
            class="field brand-input pill-input"
            :value="brand.audience"
            :disabled="!canWrite || busy"
            :title="canWrite ? '' : t('auth.noPerm')"
            @change="emit('save', 'audience', $event)"
          />
        </div>

        <div class="brand-field-group">
          <label class="lbl field-lbl" for="brand-voice">VOICE + TONE</label>
          <input
            id="brand-voice"
            class="field brand-input pill-input"
            :value="brand.voice"
            :disabled="!canWrite || busy"
            :title="canWrite ? '' : t('auth.noPerm')"
            @change="emit('save', 'voice', $event)"
          />
        </div>
      </div>

      <!-- Right Column: DO & DON'T Cards -->
      <div class="brand-voice-panels">
        <!-- DO Card -->
        <div class="brand-rule-card card-do">
          <div class="brand-rule-head">
            <span class="rule-title">DO</span>
          </div>
          <ul class="brand-rule-list">
            <li v-for="(d, i) in brand.dos" :key="`${d}-${i}`" class="brand-rule-item">
              <span class="rule-text">{{ d }}</span>
              <button
                v-if="canWrite"
                type="button"
                class="rule-remove"
                title="Remove rule"
                aria-label="Remove rule"
                @click="emit('remove-do', i)"
              >
                ×
              </button>
            </li>
            <li v-if="!brand.dos.length" class="empty-rules">No rules yet.</li>
          </ul>
          <form class="brand-rule-add" @submit.prevent="handleAddDo">
            <input
              class="rule-input pill-rule-input"
              v-model="doDraft"
              placeholder="add rule"
              aria-label="Add DO rule"
              :disabled="!canWrite || busy"
              :title="canWrite ? '' : t('auth.noPerm')"
            />
            <button
              type="submit"
              class="rule-btn pill-rule-btn"
              :disabled="!canWrite || busy || !doDraft.trim()"
              :title="canWrite ? '' : t('auth.noPerm')"
            >
              ADD
            </button>
          </form>
        </div>

        <!-- DON'T Card -->
        <div class="brand-rule-card card-dont">
          <div class="brand-rule-head">
            <span class="rule-title">DON'T</span>
          </div>
          <ul class="brand-rule-list">
            <li v-for="(d, i) in brand.donts" :key="`${d}-${i}`" class="brand-rule-item">
              <span class="rule-text">{{ d }}</span>
              <button
                v-if="canWrite"
                type="button"
                class="rule-remove"
                title="Remove rule"
                aria-label="Remove rule"
                @click="emit('remove-dont', i)"
              >
                ×
              </button>
            </li>
            <li v-if="!brand.donts.length" class="empty-rules">No rules yet.</li>
          </ul>
          <form class="brand-rule-add" @submit.prevent="handleAddDont">
            <input
              class="rule-input pill-rule-input"
              v-model="dontDraft"
              placeholder="add rule"
              aria-label="Add DON'T rule"
              :disabled="!canWrite || busy"
              :title="canWrite ? '' : t('auth.noPerm')"
            />
            <button
              type="submit"
              class="rule-btn pill-rule-btn"
              :disabled="!canWrite || busy || !dontDraft.trim()"
              :title="canWrite ? '' : t('auth.noPerm')"
            >
              ADD
            </button>
          </form>
        </div>
      </div>
    </div>
  </section>
</template>
