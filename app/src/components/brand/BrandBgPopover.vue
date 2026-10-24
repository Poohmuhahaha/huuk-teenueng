<script setup lang="ts">
import { ref } from 'vue'

defineProps<{
  isOpen: boolean
  bgTargetDevice: 'desktop' | 'mobile'
  customBgDesktop: string
  customBgMobile: string
  currentActiveBg: string
}>()

const emit = defineEmits<{
  (e: 'close'): void
  (e: 'set-target-device', device: 'desktop' | 'mobile'): void
  (e: 'pick-bg-file', event: Event): void
  (e: 'clear-bg'): void
  (e: 'apply-bg-url', url: string): void
}>()

const bgUrlDraft = ref('')

function handleApplyUrl(): void {
  const url = bgUrlDraft.value.trim()
  if (!url) return
  emit('apply-bg-url', url)
  bgUrlDraft.value = ''
}
</script>

<template>
  <div v-if="isOpen" class="brand-bg-popover edit-image-modal" @click.stop>
    <div class="bg-popover-head">
      <strong class="edit-image-title">Edit image</strong>
      <button type="button" class="btn-close w-auto" aria-label="Close" @click="emit('close')">
        <svg
          width="14"
          height="14"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
          stroke-linecap="round"
          stroke-linejoin="round"
          aria-hidden="true"
        >
          <path d="M18 6 6 18M6 6l12 12" />
        </svg>
      </button>
    </div>

    <!-- Device Target Toggle (Desktop vs Mobile) -->
    <div class="bg-device-toggle">
      <button
        type="button"
        class="bg-device-btn"
        :class="{ active: bgTargetDevice === 'desktop' }"
        @click="emit('set-target-device', 'desktop')"
      >
        <svg
          width="14"
          height="14"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
          stroke-linecap="round"
          stroke-linejoin="round"
        >
          <rect width="20" height="14" x="2" y="3" rx="2" />
          <line x1="8" x2="16" y1="21" y2="21" />
          <line x1="12" x2="12" y1="17" y2="21" />
        </svg>
        <span>Desktop (16:9)</span>
        <span v-if="customBgDesktop" class="device-dot">●</span>
      </button>
      <button
        type="button"
        class="bg-device-btn"
        :class="{ active: bgTargetDevice === 'mobile' }"
        @click="emit('set-target-device', 'mobile')"
      >
        <svg
          width="14"
          height="14"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
          stroke-linecap="round"
          stroke-linejoin="round"
        >
          <rect width="14" height="20" x="5" y="2" rx="2" ry="2" />
          <line x1="12" x2="12.01" y1="18" y2="18" />
        </svg>
        <span>Mobile (9:16)</span>
        <span v-if="customBgMobile" class="device-dot">●</span>
      </button>
    </div>

    <!-- Preview Area styled like uploaded image -->
    <div class="edit-image-preview-wrap">
      <!-- Desktop preview -->
      <div
        v-if="bgTargetDevice === 'desktop'"
        class="edit-preview-desktop"
        :style="customBgDesktop ? { backgroundImage: `url(${customBgDesktop})` } : {}"
      >
        <div class="edit-preview-badge">16 : 9</div>
        <div v-if="!customBgDesktop" class="edit-preview-empty">Default background active</div>
      </div>
      <!-- Mobile preview -->
      <div
        v-else
        class="edit-preview-mobile"
        :style="customBgMobile ? { backgroundImage: `url(${customBgMobile})` } : {}"
      >
        <div class="edit-preview-badge">9 : 16</div>
        <div v-if="!customBgMobile" class="edit-preview-empty">Default mobile background</div>
      </div>
    </div>

    <div class="bg-popover-body">
      <div class="row" style="gap: 8px;">
        <label class="btn btn-sm btn-upload w-auto">
          <svg
            width="14"
            height="14"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
            aria-hidden="true"
          >
            <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4" />
            <polyline points="17 8 12 3 7 8" />
            <line x1="12" x2="12" y1="3" y2="15" />
          </svg>
          <span>Upload {{ bgTargetDevice === 'desktop' ? 'Desktop' : 'Mobile' }} Image</span>
          <input type="file" accept="image/*" hidden @change="emit('pick-bg-file', $event)" />
        </label>
        <button
          v-if="currentActiveBg"
          type="button"
          class="btn btn-sm btn-remove-bg w-auto"
          @click="emit('clear-bg')"
        >
          Remove
        </button>
      </div>
      <div class="bg-url-row">
        <input
          class="field field-sm"
          v-model="bgUrlDraft"
          :placeholder="`or paste ${bgTargetDevice} image URL`"
          @keyup.enter="handleApplyUrl"
        />
        <button
          type="button"
          class="btn btn-sm w-auto"
          :disabled="!bgUrlDraft.trim()"
          @click="handleApplyUrl"
        >
          Apply
        </button>
      </div>
    </div>
  </div>
</template>
