<script setup lang="ts">
import { t } from '@/core/i18n'

function capitalize(text: string): string {
  return text.charAt(0).toUpperCase() + text.slice(1)
}

defineProps<{
  styleDraft: { radius: number; fillOpacity: number; strokeWidth: number; shadow: string }
  canWrite: boolean
  busy: boolean
  styleBusy: boolean
  styleNotice: string
  styleErr: string
}>()

const emit = defineEmits<{
  (e: 'save-style'): void
  (e: 'set-shadow', shadow: string): void
  (e: 'copy-style-css'): void
  (e: 'download-brand-json'): void
  (e: 'import-brand-json', event: Event): void
}>()
</script>

<template>
  <section class="brand-body">
    <div class="panel style-panel">
      <div class="style-row">
        <span class="style-label">{{ t('brand.styleRadius') }}</span>
        <input
          class="field style-range"
          type="range"
          min="0"
          max="24"
          step="1"
          v-model.number="styleDraft.radius"
          :disabled="!canWrite || busy"
          :aria-label="t('brand.styleRadius')"
          @change="emit('save-style')"
        />
        <span class="style-value">
          <input
            class="field style-num"
            type="number"
            min="0"
            max="24"
            step="1"
            v-model.number="styleDraft.radius"
            :disabled="!canWrite || busy"
            :aria-label="t('brand.styleRadius')"
            @change="emit('save-style')"
          />
          <span class="muted">px</span>
        </span>
      </div>

      <div class="style-row">
        <span class="style-label">{{ t('brand.styleFill') }}</span>
        <input
          class="field style-range"
          type="range"
          min="5"
          max="100"
          step="1"
          v-model.number="styleDraft.fillOpacity"
          :disabled="!canWrite || busy"
          :aria-label="t('brand.styleFill')"
          @change="emit('save-style')"
        />
        <span class="style-value">
          <input
            class="field style-num"
            type="number"
            min="5"
            max="100"
            step="1"
            v-model.number="styleDraft.fillOpacity"
            :disabled="!canWrite || busy"
            :aria-label="t('brand.styleFill')"
            @change="emit('save-style')"
          />
          <span class="muted">%</span>
        </span>
      </div>

      <div class="style-row">
        <span class="style-label">{{ t('brand.styleStroke') }}</span>
        <input
          class="field style-range"
          type="range"
          min="0"
          max="3"
          step="1"
          v-model.number="styleDraft.strokeWidth"
          :disabled="!canWrite || busy"
          :aria-label="t('brand.styleStroke')"
          @change="emit('save-style')"
        />
        <span class="style-value">
          <input
            class="field style-num"
            type="number"
            min="0"
            max="3"
            step="1"
            v-model.number="styleDraft.strokeWidth"
            :disabled="!canWrite || busy"
            :aria-label="t('brand.styleStroke')"
            @change="emit('save-style')"
          />
          <span class="muted">px</span>
        </span>
      </div>

      <div class="style-row">
        <span class="style-label">{{ t('brand.styleShadow') }}</span>
        <div class="style-segmented" role="group" :aria-label="t('brand.styleShadow')">
          <button
            v-for="option in ['none', 'soft', 'strong']"
            :key="option"
            type="button"
            :class="{ on: styleDraft.shadow === option }"
            :disabled="!canWrite || busy"
            @click="emit('set-shadow', option)"
          >
            {{ t('brand.styleShadow' + capitalize(option)) }}
          </button>
        </div>
      </div>

      <div class="style-sample">
        <span class="btn btn-primary" :class="{ ghost: true }">{{ t('brand.paletteSample') }}</span>
        <span class="pv-link" :style="{ color: 'var(--accent)' }">{{ t('brand.paletteLink') }}</span>
        <span class="strip-bars">
          <span v-for="i in 4" :key="i" class="strip-bar" :style="{ height: `${8 + i * 3}px` }" />
        </span>
      </div>

      <div class="style-row style-export">
        <span class="style-label">{{ t('brand.styleExport') }}</span>
        <div class="row" style="gap: 8px;">
          <button class="btn" :disabled="styleBusy" @click="emit('copy-style-css')">
            {{ t('brand.styleCopyCss') }}
          </button>
          <button class="btn" :disabled="styleBusy" @click="emit('download-brand-json')">
            {{ t('brand.styleDownloadJson') }}
          </button>
          <label class="btn" :class="{ disabled: !canWrite || styleBusy }" :title="canWrite ? '' : t('auth.noPerm')">
            {{ t('brand.styleUploadJson') }}
            <input
              type="file"
              accept=".json,application/json"
              hidden
              :disabled="!canWrite || styleBusy"
              @change="emit('import-brand-json', $event)"
            />
          </label>
          <span v-if="styleNotice" class="oknote" role="status">{{ styleNotice }}</span>
          <span v-if="styleErr" class="autherr" role="alert">{{ styleErr }}</span>
        </div>
      </div>
    </div>
  </section>
</template>
