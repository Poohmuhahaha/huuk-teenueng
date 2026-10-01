<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { usePosts, useSetup } from '@/core/queries'
import { MONTHS } from '@/mock/db'
import { monthName, t } from '@/core/i18n'
import { monthOfDate } from '@/core/deeplink'
import { navDate } from '@/core/navdate'
import CarouselTabs from '@/components/ui/CarouselTabs.vue'
import CalendarGrid from '@/components/ui/CalendarGrid.vue'

const month = ref<number>(2)
const weekStart = ref<0 | 1>(0)
const fPillar = ref('')
const fPlatform = ref('')
const fFormat = ref('')
const fStatus = ref('')
const showPillar = ref(true)
const showPlatform = ref(true)
const showStatus = ref(true)

const { data: posts } = usePosts(month)
const { data: setup } = useSetup()

// The Smart Calendar follows the navbar date chooser: picking a date there
// moves this view to that month (manual tab switches still work after).
watch(
  navDate,
  (value) => {
    const m = monthOfDate(value)
    if (m !== null) month.value = m
  },
  { immediate: true },
)

const year = computed(() => setup.value?.year ?? new Date().getFullYear())

const filtered = computed(() =>
  (posts.value ?? []).filter((p) =>
    (!fPillar.value || p.pillar === fPillar.value) &&
    (!fPlatform.value || p.platforms.includes(fPlatform.value)) &&
    (!fFormat.value || p.format === fFormat.value) &&
    (!fStatus.value || p.status === fStatus.value),
  ),
)

function opt(list: string[] | undefined): string[] {
  return list ?? []
}
</script>

<template>
  <h1>{{ t('calendar.title') }}</h1>
  <CarouselTabs :items="MONTHS" v-model="month" />
  <div class="row mt">
    <button class="btn" :class="{ 'btn-primary': weekStart === 0 }" @click="weekStart = 0">Sunday</button>
    <button class="btn" :class="{ 'btn-primary': weekStart === 1 }" @click="weekStart = 1">Monday</button>
    <span class="muted">{{ monthName(month) }}</span>
  </div>
  <div class="grid4 mt" v-if="setup">
    <div><label class="lbl">Pillar</label>
      <select class="field" v-model="fPillar"><option value="">all</option><option v-for="o in opt(setup.pillars)" :key="o" :value="o">{{ o }}</option></select></div>
    <div><label class="lbl">Platform</label>
      <select class="field" v-model="fPlatform"><option value="">all</option><option v-for="o in opt(setup.platforms)" :key="o" :value="o">{{ o }}</option></select></div>
    <div><label class="lbl">Format</label>
      <select class="field" v-model="fFormat"><option value="">all</option><option v-for="o in opt(setup.formats)" :key="o" :value="o">{{ o }}</option></select></div>
    <div><label class="lbl">Status</label>
      <select class="field" v-model="fStatus"><option value="">all</option><option v-for="o in opt(setup.statuses)" :key="o" :value="o">{{ o }}</option></select></div>
  </div>
  <div class="row mt">
    <label class="muted"><input type="checkbox" v-model="showPillar" /> pillar</label>
    <label class="muted"><input type="checkbox" v-model="showPlatform" /> platform</label>
    <label class="muted"><input type="checkbox" v-model="showStatus" /> status</label>
  </div>
  <div class="mt">
    <CalendarGrid :year="year" :month="month" :posts="filtered"
      :week-start="weekStart" :show-pillar="showPillar" :show-platform="showPlatform" :show-status="showStatus" />
  </div>
</template>
