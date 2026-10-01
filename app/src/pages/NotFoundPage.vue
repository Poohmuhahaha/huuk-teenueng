<script setup lang="ts">
import { useRouter } from 'vue-router'
import { screens } from '@/core/screens'
import { t } from '@/core/i18n'

const router = useRouter()

function goHome(): void {
  void router.replace('/')
}
</script>

<template>
  <div class="card mt">
    <h1>{{ t('notFound.title') }}</h1>
    <p class="muted">{{ t('notFound.message') }}</p>
    <div class="row mt">
      <button class="btn btn-primary" @click="goHome">{{ t('notFound.home') }}</button>
      <RouterLink class="btn" :to="screens[1].to">{{ t('notFound.planner') }}</RouterLink>
    </div>
  </div>
</template>
