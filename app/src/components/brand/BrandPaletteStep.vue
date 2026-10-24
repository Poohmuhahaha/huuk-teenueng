<script setup lang="ts">
import { t } from '@/core/i18n'

defineProps<{
  paletteDraft: string[]
  canWrite: boolean
  busy: boolean
  copiedSlot: string | null
  readableOn: (hex: string) => string
  paletteSlots: readonly string[]
  paletteRoles: Record<string, string>
  paletteUsage: Record<string, string>
}>()

const emit = defineEmits<{
  (e: 'update-slot', index: number, value: string): void
  (e: 'save-palette'): void
  (e: 'copy-hex', slot: string): void
}>()
</script>

<template>
  <section class="brand-body">
    <div class="panel">
      <div class="palette-strip">
        <span class="strip-label muted">{{ t('brand.palettePreview') }}</span>
        <span class="strip-btn" :style="{ background: paletteDraft[0] || 'var(--accent)' }">
          {{ t('brand.paletteSample') }}
        </span>
        <span class="strip-link" :style="{ color: paletteDraft[0] || 'var(--accent)' }">
          {{ t('brand.paletteLink') }}
        </span>
        <span class="strip-bars" :aria-label="t('brand.paletteBars')">
          <span
            v-for="i in 4"
            :key="i"
            class="strip-bar"
            :style="{ background: paletteDraft[i - 1] || 'var(--wash)', height: `${10 + i * 3}px` }"
          />
        </span>
      </div>
      <div class="palette-grid">
        <div
          v-for="slot in paletteSlots"
          :key="slot"
          class="palette-card"
          :class="{ 'is-primary': slot === '1' }"
        >
          <label class="palette-tile-label" :title="t('brand.palettePick')">
            <input
              type="color"
              :value="paletteDraft[Number(slot) - 1] || '#0a0a0a'"
              :disabled="!canWrite || busy"
              :aria-label="`Color ${slot}`"
              @input="emit('update-slot', Number(slot) - 1, ($event.target as HTMLInputElement).value)"
              @change="emit('save-palette')"
            />
            <span
              class="palette-tile"
              :style="{ background: paletteDraft[Number(slot) - 1] || 'var(--wash)' }"
            >
              <span class="palette-tile-role">{{ t(paletteRoles[slot]) }}</span>
              <span v-if="slot === '1'" class="palette-tile-star" aria-hidden="true">★</span>
              <span
                class="palette-tile-sample"
                :style="{ color: readableOn(paletteDraft[Number(slot) - 1] || '#0a0a0a') }"
              >
                Aa
              </span>
            </span>
          </label>
          <div class="palette-meta">
            <p class="palette-caption">{{ t(paletteUsage[slot]) }}</p>
            <div class="palette-stats">
              <label class="palette-field">
                <input
                  class="field palette-hex"
                  :value="paletteDraft[Number(slot) - 1]"
                  :disabled="!canWrite || busy"
                  placeholder="#0a0a0a"
                  :aria-label="`Color hex ${slot}`"
                  @input="emit('update-slot', Number(slot) - 1, ($event.target as HTMLInputElement).value)"
                  @change="emit('save-palette')"
                />
              </label>
              <button
                type="button"
                class="palette-copy"
                :disabled="!paletteDraft[Number(slot) - 1]"
                @click="emit('copy-hex', slot)"
              >
                {{ copiedSlot === slot ? t('brand.paletteCopied') : t('brand.paletteCopy') }}
              </button>
            </div>
          </div>
        </div>
      </div>
    </div>
  </section>
</template>
