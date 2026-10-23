<script setup lang="ts">
// Content Calendar — Unified Month Calendar Grid with per-day post chips,
// paired with an Agenda view for daily timeline focus.
import { computed, ref, watch } from 'vue'
import { useRouter } from 'vue-router'
import { usePosts } from '@/core/queries'
import { navDate, yearOfDate } from '@/core/navdate'
import { monthOfDate } from '@/core/deeplink'
import { deckFull } from '@/core/deck'
import { useWidePane } from '@/composables/useWidePane'
import { monthName } from '@/core/i18n'
import type { Post } from '@/mock/db'

const router = useRouter()
const root = ref<HTMLElement | null>(null)
const { wide } = useWidePane(root, 'cal-scroll')
const isWide = computed(() => deckFull.value || wide.value)

const month = ref<number>(new Date().getMonth() + 1)
const year = ref<number>(new Date().getFullYear())
const { data: posts, isPending, isError, error: loadError, refetch } = usePosts(month, year)

// View mode: 'grid' (Month grid) or 'agenda' (daily timeline)
const viewMode = ref<'grid' | 'agenda'>('grid')

// Sync with navbar date
watch(
  navDate,
  (value) => {
    const m = monthOfDate(value)
    if (m !== null) month.value = m
    const y = yearOfDate(value)
    if (y !== null) year.value = y
  },
  { immediate: true },
)

const list = computed<Post[]>(() => posts.value ?? [])
const isEmpty = computed(() => list.value.length === 0)

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

// Month Calendar Grid calculations
interface MonthCell {
  date: string
  dayNum: number
  isCurrentMonth: boolean
  isToday: boolean
  posts: Post[]
}

const DOW_HEADERS = ['Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat', 'Sun']

const monthCells = computed<MonthCell[]>(() => {
  const y = year.value
  const m = month.value
  const first = new Date(y, m - 1, 1)
  // Monday start: 0 = Mon, ..., 6 = Sun
  const startDay = (first.getDay() + 6) % 7
  const totalDays = new Date(y, m, 0).getDate()
  const prevMonthTotal = new Date(y, m - 1, 0).getDate()
  
  const cells: MonthCell[] = []
  const pad = (n: number): string => String(n).padStart(2, '0')

  // Previous month trailing days
  const prevM = m === 1 ? 12 : m - 1
  const prevY = m === 1 ? y - 1 : y
  for (let i = startDay - 1; i >= 0; i--) {
    const d = prevMonthTotal - i
    const dateStr = `${prevY}-${pad(prevM)}-${pad(d)}`
    cells.push({
      date: dateStr,
      dayNum: d,
      isCurrentMonth: false,
      isToday: dateStr === todayISO.value,
      posts: [],
    })
  }

  // Current month days
  for (let d = 1; d <= totalDays; d++) {
    const dateStr = `${y}-${pad(m)}-${pad(d)}`
    const dayPosts = list.value.filter((p) => p.date === dateStr).sort(byTime)
    cells.push({
      date: dateStr,
      dayNum: d,
      isCurrentMonth: true,
      isToday: dateStr === todayISO.value,
      posts: dayPosts,
    })
  }

  // Next month leading days to complete full 7-col grid
  const nextM = m === 12 ? 1 : m + 1
  const nextY = m === 12 ? y + 1 : y
  let nextDayNum = 1
  while (cells.length % 7 !== 0 || cells.length < 35) {
    const dateStr = `${nextY}-${pad(nextM)}-${pad(nextDayNum)}`
    cells.push({
      date: dateStr,
      dayNum: nextDayNum,
      isCurrentMonth: false,
      isToday: dateStr === todayISO.value,
      posts: [],
    })
    nextDayNum++
  }

  return cells
})

const rowCount = computed(() => Math.ceil(monthCells.value.length / 7))

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

function statusDotClass(status?: string): string {
  const s = (status ?? '').toLowerCase()
  if (s === 'done') return 'bg-emerald-500'
  if (s === 'wip' || s === 'design') return 'bg-amber-500'
  return 'bg-blue-400'
}

function openPost(p: Post): void {
  void router.push({ path: '/plan', query: { tab: 'monthly', m: String(month.value), post: p.id } })
}

function goAdd(date?: string): void {
  void router.push({ path: '/plan', query: { tab: 'monthly', m: String(month.value), ...(date ? { date } : {}) } })
}
</script>

<template>
  <div ref="root" class="cal-container">
    <!-- Header with Action (matching Huuk design system in Dashboard) -->
    <header class="flex items-center justify-between flex-wrap gap-4 pb-2 border-b border-line flex-none">
      <div>
        <div class="text-xs font-bold uppercase tracking-wider text-muted mb-0.5">
          Content Schedule · {{ monthName(month) }} {{ year }}
        </div>
        <h2 class="text-2xl sm:text-3xl font-extrabold uppercase tracking-tight m-0 font-display text-ink leading-tight">
          Calendar
        </h2>
      </div>

      <!-- Right: View Mode Capsule (Month / Agenda) + New Post Button -->
      <div class="flex items-center gap-3">
        <div class="inline-flex items-center gap-1 p-1 bg-wash border border-line rounded-full shadow-2xs">
          <button
            type="button"
            class="px-4 py-1.5 rounded-full text-xs font-semibold transition-all cursor-pointer select-none whitespace-nowrap"
            :class="viewMode === 'grid' ? '!bg-ink !text-white shadow-xs font-bold' : 'text-muted hover:text-ink hover:bg-surface/60'"
            @click="viewMode = 'grid'"
          >
            Month
          </button>
          <button
            type="button"
            class="px-4 py-1.5 rounded-full text-xs font-semibold transition-all cursor-pointer select-none whitespace-nowrap"
            :class="viewMode === 'agenda' ? '!bg-ink !text-white shadow-xs font-bold' : 'text-muted hover:text-ink hover:bg-surface/60'"
            @click="viewMode = 'agenda'"
          >
            Agenda
          </button>
        </div>

        <button
          type="button"
          class="btn btn-primary rounded-full px-4 py-2 text-xs font-semibold flex items-center gap-1.5 cursor-pointer shadow-xs hover:shadow-sm transition-all"
          title="Add new post"
          @click="() => goAdd()"
        >
          <span class="text-sm leading-none font-bold">+</span>
          <span>New post</span>
        </button>
      </div>
    </header>

    <div v-if="isPending" class="muted py-8 text-center" role="status">Loading calendar…</div>
    <div v-else-if="isError" class="card" role="alert">
      <p class="muted">Could not load calendar: {{ loadError?.message }}</p>
      <button type="button" class="btn" @click="() => refetch()">Retry</button>
    </div>

    <!-- VIEW 1: MONTH CALENDAR GRID FRAME -->
    <div v-else-if="viewMode === 'grid'" class="cal-frame">
      <!-- Days of Week Header -->
      <div class="cal-dow-grid">
        <div v-for="dow in DOW_HEADERS" :key="dow" class="cal-dow-cell">
          {{ dow }}
        </div>
      </div>

      <!-- Days Grid Cells -->
      <div
        class="cal-days-grid"
        :style="{ gridTemplateRows: `repeat(${rowCount}, minmax(0, 1fr))` }"
      >
        <div
          v-for="cell in monthCells"
          :key="cell.date"
          class="cal-day-cell"
          :class="{
            outside: !cell.isCurrentMonth,
            today: cell.isToday,
          }"
          @click="selectDay(cell.date)"
        >
          <!-- Cell Header: Day Number -->
          <div class="flex items-center justify-between mb-1 flex-none">
            <span
              class="text-xs font-semibold select-none"
              :class="[
                cell.isToday
                  ? 'w-5 h-5 rounded-full bg-ink text-white flex items-center justify-center font-bold text-[10px] shadow-2xs'
                  : cell.isCurrentMonth ? 'text-ink pl-0.5' : 'text-muted/40 pl-0.5'
              ]"
            >
              {{ cell.dayNum }}
            </span>
          </div>

          <!-- List of posts inside this day cell -->
          <div class="flex flex-col gap-1 overflow-hidden flex-1 min-h-0">
            <button
              v-for="p in cell.posts.slice(0, 3)"
              :key="p.id"
              type="button"
              class="cal-chip flex-none"
              :title="`${p.time || ''} · ${p.topic} (${p.status})`"
              @click.stop="openPost(p)"
            >
              <span class="cal-dot" :class="statusDotClass(p.status)" />
              <span v-if="p.time" class="cal-time">{{ p.time }}</span>
              <span class="cal-title">{{ p.topic || 'Untitled' }}</span>
            </button>
            <div v-if="cell.posts.length > 3" class="text-[10px] text-muted font-medium pl-1 truncate flex-none">
              +{{ cell.posts.length - 3 }} more
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- VIEW 2: AGENDA / TIMELINE SPLIT VIEW -->
    <div v-else class="smartcal" :class="{ split: isWide }">
      <!-- TODAY (left column) -->
      <section class="sc-today">
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

      <!-- CALENDAR TIMELINE (right column) -->
      <section class="sc-cal">
        <div class="sc-month">
          <strong>Scheduled Days</strong>
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
        <p v-if="isEmpty || !days.length" class="muted">No posts in {{ monthName(month) }} {{ year }}.</p>
      </section>
    </div>
  </div>
</template>
