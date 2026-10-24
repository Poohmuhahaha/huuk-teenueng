<script setup lang="ts">
import { t } from '@/core/i18n'
import type { Brand, FontAsset } from '@/mock/db'

defineProps<{
  brand: Brand
  canWrite: boolean
  importedFonts: FontAsset[]
  suggestedFonts: readonly string[]
  uploading: boolean
  fontError: string
}>()

const emit = defineEmits<{
  (e: 'select-font', family: string): void
  (e: 'remove-font', font: FontAsset): void
  (e: 'import-fonts', event: Event): void
}>()
</script>

<template>
  <section class="brand-body">
    <div class="panel">
      <div class="font-preview" :style="{ fontFamily: `'${brand.fonts[0] || 'Inter'}', var(--font)` }">
        <span class="font-preview-sample">Aa Bb 123 · สวัสดี</span>
        <span class="muted">{{ t('brand.fontPreview') }}</span>
      </div>
      <div class="font-grid">
        <div
          v-for="f in importedFonts"
          :key="f.name"
          class="font-choice"
          :class="{ active: brand.fonts[0] === f.family }"
        >
          <button
            type="button"
            class="font-pick"
            :disabled="!canWrite"
            @click="emit('select-font', f.family)"
          >
            <span class="font-sample" :style="{ fontFamily: `'${f.family}', var(--font)` }">Aa Bb 123</span>
            <span class="font-name">{{ f.family }}</span>
            <span v-if="brand.fonts[0] === f.family" class="font-check" aria-hidden="true">✓</span>
          </button>
          <button
            type="button"
            class="font-remove"
            :disabled="!canWrite"
            :title="t('brand.fontDelete')"
            :aria-label="`${t('brand.fontDelete')} ${f.family}`"
            @click="emit('remove-font', f)"
          >
            ×
          </button>
        </div>
        <button
          v-for="s in suggestedFonts"
          :key="s"
          type="button"
          class="font-choice font-pick"
          :class="{ active: brand.fonts[0] === s }"
          :style="{ fontFamily: `'${s}', var(--font)` }"
          :disabled="!canWrite"
          @click="emit('select-font', s)"
        >
          <span class="font-sample">Aa Bb 123</span>
          <span class="font-name">{{ s }}</span>
          <span v-if="brand.fonts[0] === s" class="font-check" aria-hidden="true">✓</span>
        </button>
        <label
          class="font-choice font-import"
          :class="{ disabled: !canWrite || uploading }"
          :title="t('brand.fontHint')"
        >
          <span class="font-import-plus" aria-hidden="true">+</span>
          <span class="font-name">{{ uploading ? t('brand.uploading') : t('brand.fontImport') }}</span>
          <input
            type="file"
            accept=".woff2,.woff,.ttf,.otf"
            multiple
            hidden
            :disabled="!canWrite || uploading"
            @change="emit('import-fonts', $event)"
          />
        </label>
      </div>
      <p v-if="fontError" class="autherr" role="alert">{{ fontError }}</p>
      <p class="muted panel-note">{{ t('brand.applied') }}</p>
    </div>
  </section>
</template>
