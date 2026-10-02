<script setup lang="ts">
// Smart Calendar — minimal card design: a Today / Calendar toggle, a big date
// readout, and pastel day cards with a per-hour timeline of planned posts.
import { computed, ref, watch } from 'vue'
import { useRouter } from 'vue-router'
import { usePosts, useSetup } from '@/core/queries'
import { navDate } from '@/core/navdate'
import { monthOfDate } from '@/core/deeplink'
import { monthName } from '@/core/i18n'
import type { Post } from '@/mock/db'

const router = useRouter()
const month = ref<number>(new Date().getMonth() + 1)
const view = ref<'today' | 'calendar'>('today')

const { data: posts } = usePosts(month)
const { data: setup } = useSetup()

// The Smart Calendar follows the navbar date chooser.
watch(
  navDate,
  (value) => {
    const m = monthOfDate(value)
    if (m !== null) month.value = m
  },
  { immediate: true },
)

const year = computed(() => setup.value?.year ?? new Date().getFullYear())
const list = computed<Post[]>(() => posts.value ?? [])

function isoOf(d: Date): string {
  const m = String(d.getMonth() + 1).padStart(2, '0')
  const day = String(d.getDate()).padStart(2, '0')
  return `${d.getFullYear()}-${m}-${day}`
}
const todayISO = computed(() =>
  typeof navDate.value === 'string' && /^\d{4}-\d{2}-\d{2}$/.test(navDate.value)
    ? navDate.value
    : isoOf(new Date()),
)

// Monochrome: only the focused day is black; every other day is ghost (outline).
const selectedDay = ref<string>('')
watch(todayISO, (v) => { if (!selectedDay.value) selectedDay.value = v }, { immediate: true })
function isOn(date: string): boolean {
  return date === selectedDay.value
}
function selectDay(date: string): void {
  selectedDay.value = date
}

function byTime(a: Post, b: Post): number {
  return (a.time ?? '').localeCompare(b.time ?? '')
}

const todayPosts = computed(() => list.value.filter((p) => p.date === todayISO.value).sort(byTime))

const days = computed(() => {
  const map = new Map<string, Post[]>()
  for (const p of list.value) {
    if (!p.date) continue
    const arr = map.get(p.date) ?? []
    arr.push(p)
    map.set(p.date, arr)
  }
  return [...map.entries()]
    .sort((a, b) => a[0].localeCompare(b[0]))
    .map(([date, ps]) => ({ date, posts: ps.sort(byTime) }))
})

function fmtTime(value: string | undefined): string {
  if (!value) return '—'
  const [h, m] = value.split(':').map(Number)
  if (!Number.isFinite(h)) return value
  const ampm = h >= 12 ? 'PM' : 'AM'
  const hh = h % 12 === 0 ? 12 : h % 12
  return `${hh}:${String(m ?? 0).padStart(2, '0')} ${ampm}`
}
function fmtHour(value: string | undefined): string {
  if (!value) return ''
  const h = Number(value.split(':')[0])
  return Number.isFinite(h) ? `${h % 12 === 0 ? 12 : h % 12} ${h >= 12 ? 'pm' : 'am'}` : ''
}
function parseISO(iso: string): Date {
  const [y, m, d] = iso.split('-').map(Number)
  return new Date(y, (m ?? 1) - 1, d ?? 1)
}
const WD = ['Sunday', 'Monday', 'Tuesday', 'Wednesday', 'Thursday', 'Friday', 'Saturday']
function weekday(iso: string): string {
  return WD[parseISO(iso).getDay()] ?? ''
}
function dayNum(iso: string): string {
  return String(parseISO(iso).getDate()).padStart(2, '0')
}
function monShort(iso: string): string {
  return ['JAN', 'FEB', 'MAR', 'APR', 'MAY', 'JUN', 'JUL', 'AUG', 'SEP', 'OCT', 'NOV', 'DEC'][
    parseISO(iso).getMonth()
  ]
}
const shortMonths = ['JAN', 'FEB', 'MAR', 'APR', 'MAY', 'JUN', 'JUL', 'AUG', 'SEP', 'OCT', 'NOV', 'DEC']
const prevMon = computed(() => shortMonths[(month.value + 10) % 12])
const nextMon = computed(() => shortMonths[month.value % 12])

function shiftMonth(delta: number): void {
  month.value = ((month.value - 1 + delta + 12) % 12) + 1
}

function openPost(p: Post): void {
  void router.push({ path: '/plan', query: { tab: 'monthly', m: String(month.value), post: p.id } })
}
function goAdd(): void {
  void router.push({ path: '/plan', query: { tab: 'monthly', m: String(month.value) } })
}
</script>

<template>
  <div class="smartcal">
    <!-- top: Today / Calendar toggle + add -->
    <div class="sc-top">
      <div class="sc-seg" role="tablist">
        <button type="button" :class="{ on: view === 'today' }" @click="view = 'today'">Today</button>
        <button type="button" :class="{ on: view === 'calendar' }" @click="view = 'calendar'">Calendar</button>
      </div>
      <button type="button" class="sc-add" aria-label="Add post" @click="goAdd">+</button>
    </div>

    <!-- TODAY -->
    <section v-if="view === 'today'" class="sc-today">
      <div class="sc-weekday">{{ weekday(todayISO) }}</div>
      <div class="sc-daterow">
        <div class="sc-bigdate">
          <span class="sc-dd">{{ dayNum(todayISO) }}</span>
          <span class="sc-sep">.</span>
          <span class="sc-dd">{{ String(parseISO(todayISO).getMonth() + 1).padStart(2, '0') }}</span>
          <span class="sc-mon">{{ monShort(todayISO) }}</span>
        </div>
        <div class="sc-times">
          <div v-for="p in todayPosts.slice(0, 3)" :key="p.id" class="sc-time">
            <strong>{{ fmtTime(p.time) }}</strong>
            <span class="muted">{{ p.platforms[0] || p.pillar || '—' }}</span>
          </div>
          <p v-if="!todayPosts.length" class="muted">Nothing planned</p>
        </div>
      </div>

      <div class="sc-taskshead">
        <strong>Today's tasks</strong>
        <span class="sc-pill">Reminders</span>
      </div>

      <div class="sc-tasks">
        <button
          v-for="p in todayPosts"
          :key="p.id"
          type="button"
          class="sc-task"
          @click="openPost(p)"
        >
          <span class="sc-task-title">{{ p.topic || 'Untitled' }}</span>
          <span class="sc-task-foot">
            <span>{{ fmtTime(p.time) }}</span>
            <span class="sc-min">{{ p.status }}</span>
            <span>{{ (p.platforms || []).join(' · ') }}</span>
          </span>
        </button>
        <p v-if="!todayPosts.length" class="muted">Nothing planned for today.</p>
      </div>
    </section>

    <!-- CALENDAR -->
    <section v-else class="sc-cal">
      <div class="sc-month">
        <button type="button" class="sc-navtext" @click="shiftMonth(-1)">{{ prevMon }}</button>
        <strong>{{ shortMonths[month - 1] }}</strong>
        <button type="button" class="sc-navtext" @click="shiftMonth(1)">{{ nextMon }}</button>
      </div>

      <div
        v-for="d in days"
        :key="d.date"
        class="sc-day"
        :class="{ on: isOn(d.date) }"
        @click="selectDay(d.date)"
      >
        <div class="sc-day-left">
          <span class="sc-day-wd">{{ weekday(d.date) }}</span>
          <span class="sc-day-dd">{{ dayNum(d.date) }}</span>
          <span class="sc-day-mm">{{ monShort(d.date) }}</span>
        </div>
        <div class="sc-timeline">
          <div v-for="p in d.posts" :key="p.id" class="sc-slot">
            <span class="sc-hour">{{ fmtHour(p.time) }}</span>
            <button type="button" class="sc-event" @click.stop="openPost(p)">
              {{ p.topic || 'Untitled' }}
            </button>
          </div>
        </div>
      </div>
      <p v-if="!days.length" class="muted">No posts in {{ monthName(month) }} {{ year }}.</p>
    </section>
  </div>
</template>
