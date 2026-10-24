<script setup lang="ts">
import { t } from '@/core/i18n'
import { boundsHint } from '@/core/images'

defineProps<{
  moodboard: string[]
  canWrite: boolean
  uploadingSlot: string | null
  oversized: Record<string, boolean>
  moodboardDraft: string
  imageNotice: string
  imageError: string
}>()

const emit = defineEmits<{
  (e: 'update:moodboard-draft', value: string): void
  (e: 'note-rendered-size', url: string, event: Event): void
  (e: 'remove-moodboard', index: number): void
  (e: 'add-moodboard', event: Event): void
  (e: 'add-moodboard-url'): void
  (e: 'fix-stored-image', kind: 'moodboard', url: string): void
}>()
</script>

<template>
  <section class="brand-body">
    <div class="moodboard">
      <div v-for="(url, i) in moodboard" :key="`${url}-${i}`" class="mood-item">
        <img :src="url" :alt="`${t('brand.moodboard')} ${i + 1}`" @load="emit('note-rendered-size', url, $event)" />
        <button
          v-if="oversized[url]"
          type="button"
          class="btn oversize-fix mood-fix"
          :disabled="!canWrite || uploadingSlot === `fix-${url}`"
          @click="emit('fix-stored-image', 'moodboard', url)"
        >
          {{ uploadingSlot === `fix-${url}` ? t('brand.imageFixing') : t('brand.imageFix') }}
        </button>
        <button
          type="button"
          class="logo-remove"
          :disabled="!canWrite"
          :title="t('brand.imageRemove')"
          :aria-label="t('brand.imageRemove')"
          @click="emit('remove-moodboard', i)"
        >
          ×
        </button>
      </div>
      <label v-if="moodboard.length < 12" class="mood-add" :class="{ disabled: !canWrite }">
        <span>{{ uploadingSlot === 'moodboard' ? t('brand.imageUploading') : t('brand.moodboardAdd') }}</span>
        <input
          type="file"
          accept=".png,.jpg,.jpeg,.webp,.gif"
          multiple
          hidden
          :disabled="!canWrite || uploadingSlot === 'moodboard'"
          @change="emit('add-moodboard', $event)"
        />
      </label>
    </div>
    <div class="row mt">
      <input
        class="field mood-url"
        style="flex: 1;"
        :value="moodboardDraft"
        :disabled="!canWrite"
        :placeholder="t('brand.imageUrl')"
        @input="emit('update:moodboard-draft', ($event.target as HTMLInputElement).value)"
        @keyup.enter="emit('add-moodboard-url')"
      />
      <button
        class="btn mood-add-btn"
        :disabled="!canWrite || !moodboardDraft.trim()"
        @click="emit('add-moodboard-url')"
      >
        {{ t('brand.imageAdd') }}
      </button>
    </div>
    <p class="muted img-bounds">{{ boundsHint('moodboard') }}</p>
    <p v-if="imageNotice" class="oknote" role="status">{{ imageNotice }}</p>
    <p v-if="imageError" class="autherr" role="alert">{{ imageError }}</p>
  </section>
</template>
