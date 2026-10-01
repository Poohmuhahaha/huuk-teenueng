<script setup lang="ts">
// Info button: replaces always-visible helper paragraphs. The hint hides
// behind a small (i) toggle next to the heading or label it explains.
import { ref } from 'vue'
import { t } from '@/core/i18n'

defineProps<{ text: string }>()

const open = ref(false)
</script>

<template>
  <span class="infotip-wrap">
    <button type="button" class="infotip" :aria-expanded="open" :aria-label="t('common.moreInfo')"
      @click="open = !open">i</button>
    <span v-if="open" class="infotip-bubble" role="note">{{ text }}</span>
  </span>
</template>
