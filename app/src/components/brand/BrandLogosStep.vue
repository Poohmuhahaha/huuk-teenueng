<script setup lang="ts">
import { t } from '@/core/i18n'
import { boundsHint } from '@/core/images'

defineProps<{
  logoSlots: Array<{ index: number; url: string }>
  canWrite: boolean
  busy: boolean
  uploadingSlot: string | null
  oversized: Record<string, boolean>
  logoLabels: Record<number, string>
}>()

const emit = defineEmits<{
  (e: 'note-rendered-size', url: string, event: Event): void
  (e: 'clear-logo', index: number): void
  (e: 'pick-logo', index: number, event: Event): void
  (e: 'set-logo-url', index: number, event: Event): void
  (e: 'fix-stored-image', kind: 'logo', url: string): void
}>()
</script>

<template>
  <section class="brand-body">
    <div class="grid3">
      <div v-for="slot in logoSlots" :key="slot.index" class="logo-slot">
        <div class="logo-preview">
          <img
            v-if="slot.url"
            :src="slot.url"
            :alt="t(logoLabels[slot.index])"
            @load="emit('note-rendered-size', slot.url, $event)"
          />
          <span v-else class="muted">{{ t(logoLabels[slot.index]) }}</span>
          <button
            v-if="slot.url"
            type="button"
            class="logo-remove"
            :disabled="!canWrite"
            :title="t('brand.imageRemove')"
            :aria-label="t('brand.imageRemove')"
            @click="emit('clear-logo', slot.index)"
          >
            ×
          </button>
        </div>
        <div class="row">
          <label class="btn" :class="{ disabled: !canWrite || uploadingSlot === `logo-${slot.index}` }">
            {{ uploadingSlot === `logo-${slot.index}` ? t('brand.imageUploading') : t('brand.imageUpload') }}
            <input
              type="file"
              accept=".png,.jpg,.jpeg,.webp,.gif"
              hidden
              :disabled="!canWrite || uploadingSlot === `logo-${slot.index}`"
              @change="emit('pick-logo', slot.index, $event)"
            />
          </label>
          <input
            class="field"
            style="flex: 1;"
            :value="slot.url"
            :disabled="!canWrite || busy"
            :placeholder="t('brand.imageUrl')"
            @change="emit('set-logo-url', slot.index, $event)"
          />
        </div>
        <span class="muted img-bounds">{{ boundsHint('logo') }}</span>
        <button
          v-if="oversized[slot.url]"
          type="button"
          class="btn oversize-fix"
          :disabled="!canWrite || uploadingSlot === `fix-${slot.url}`"
          @click="emit('fix-stored-image', 'logo', slot.url)"
        >
          {{ uploadingSlot === `fix-${slot.url}` ? t('brand.imageFixing') : t('brand.imageFix') }}
        </button>
      </div>
    </div>
  </section>
</template>
