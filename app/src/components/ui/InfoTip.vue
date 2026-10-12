<script setup lang="ts">
// Info button: replaces always-visible helper paragraphs. The hint hides
// behind a small (i) toggle next to the heading or label it explains.
import { ref } from 'vue'
import { t } from '@/core/i18n'
import { useDismiss } from '@/composables/useDismiss'

defineProps<{ text: string }>()

const open = ref(false)
const root = ref<HTMLElement | null>(null)
function close(): void {
  open.value = false
}
// Outside-click + Escape dismiss, same pattern as DropdownMenu.
useDismiss(root, close)
</script>

<template>
  <span ref="root" class="inline-flex align-middle ml-2 relative">
    <button type="button" class="infotip w-[18px] h-[18px] flex-none p-0 rounded-full border border-line bg-surface text-muted text-xs italic font-bold leading-none cursor-pointer hover:border-accent hover:text-accent" :aria-expanded="open" :aria-label="t('common.moreInfo')"
      @click="open = !open">i</button>
    <span v-if="open" class="infotip-bubble z-[200] bg-surface border border-faint rounded-none w-max max-w-[min(280px,72vw)] shadow-none text-ink whitespace-normal text-left px-3 py-2.5 text-sm font-normal absolute top-[calc(100%+8px)] right-0" role="tooltip">{{ text }}</span>
  </span>
</template>
