<script setup lang="ts">
// Everything Huuk mirrors from the connected accounts, in one place: the
// Page/Instagram stats plus every post with its engagement. Same component as
// the Dashboard/Feed sections, without the row limit.
import LiveSection from '@/components/live/LiveSection.vue'
import { t } from '@/core/i18n'
</script>

<template>
  <h1 class="sr-only">{{ t('live.pageTitle') }}</h1>
  <LiveSection variant="cards" />
</template>
