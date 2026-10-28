<script setup lang="ts">
import { computed, onBeforeUnmount, ref, watch } from 'vue'
import { monthName, t } from '@/core/i18n'

const props = defineProps<{
  startDate: string | null
  endDate: string | null
  disabled?: boolean
}>()

const emit = defineEmits<{
  'update:startDate': [value: string | null]
  'update:endDate': [value: string | null]
}>()

const DOW = ['Sun', 'Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat']

const now = new Date()
const viewY = ref(now.getFullYear())
const viewM = ref(now.getMonth() + 1)

// Drag & Click state (declared before immediate watchers to avoid TDZ ReferenceError)
const isDragging = ref(false)
const dragAnchor = ref<string | null>(null)
const dragCurrent = ref<string | null>(null)
const dragDir = ref<'forward' | 'backward'>('forward')
const didDragMove = ref(false)
const pickingStep = ref<'start' | 'end'>('start')

function parseYMD(value: string | null): { y: number; m: number; d: number } | null {
  if (!value) return null
  const m = /^(\d{4})-(\d{2})-(\d{2})$/.exec(value.trim())
  if (!m) return null
  const y = Number(m[1])
  const mo = Number(m[2])
  const d = Number(m[3])
  if (mo < 1 || mo > 12 || d < 1 || d > 31) return null
  return { y, m: mo, d }
}

function formatYMD(d: Date): string {
  const y = d.getFullYear()
  const m = String(d.getMonth() + 1).padStart(2, '0')
  const day = String(d.getDate()).padStart(2, '0')
  return `${y}-${m}-${day}`
}

const todayKey = computed(() => formatYMD(new Date()))

// Keep calendar month in sync when startDate changes (e.g. switching campaigns or using DatePickerPopup)
watch(
  () => props.startDate,
  (val) => {
    if (isDragging.value) return
    const parsed = parseYMD(val) ?? parseYMD(props.endDate)
    if (parsed) {
      viewY.value = parsed.y
      viewM.value = parsed.m
    }
  },
  { immediate: true },
)

function stepMonth(delta: number): void {
  const d = new Date(viewY.value, viewM.value - 1 + delta, 1)
  viewY.value = d.getFullYear()
  viewM.value = d.getMonth() + 1
}

interface CalendarCell {
  key: string
  day: number
  inMonth: boolean
  isToday: boolean
  col: number
}

const cells = computed<CalendarCell[]>(() => {
  const firstDay = new Date(viewY.value, viewM.value - 1, 1)
  const lead = firstDay.getDay()
  const daysInMonth = new Date(viewY.value, viewM.value, 0).getDate()
  const out: CalendarCell[] = []

  // Leading days from previous month
  for (let i = lead - 1; i >= 0; i--) {
    const d = new Date(viewY.value, viewM.value - 1, -i)
    const key = formatYMD(d)
    out.push({
      key,
      day: d.getDate(),
      inMonth: false,
      isToday: key === todayKey.value,
      col: out.length % 7,
    })
  }

  // Current month days
  for (let day = 1; day <= daysInMonth; day++) {
    const d = new Date(viewY.value, viewM.value - 1, day)
    const key = formatYMD(d)
    out.push({
      key,
      day,
      inMonth: true,
      isToday: key === todayKey.value,
      col: out.length % 7,
    })
  }

  // Trailing days from next month to complete the last week
  let nextDay = 1
  while (out.length % 7 !== 0) {
    const d = new Date(viewY.value, viewM.value, nextDay++)
    const key = formatYMD(d)
    out.push({
      key,
      day: d.getDate(),
      inMonth: false,
      isToday: key === todayKey.value,
      col: out.length % 7,
    })
  }

  return out
})

const normalizedRange = computed<{ start: string | null; end: string | null }>(() => {
  const s = props.startDate
  const e = props.endDate
  if (s && e) {
    return s <= e ? { start: s, end: e } : { start: e, end: s }
  }
  if (s) return { start: s, end: s }
  if (e) return { start: e, end: e }
  return { start: null, end: null }
})

const dayCount = computed<number>(() => {
  const { start, end } = normalizedRange.value
  if (!start || !end) return 0
  const a = new Date(`${start}T00:00:00`)
  const b = new Date(`${end}T00:00:00`)
  const diff = Math.round((b.getTime() - a.getTime()) / 86400000) + 1
  return diff > 0 ? diff : 0
})

function isStart(key: string): boolean {
  return normalizedRange.value.start === key
}

function isEnd(key: string): boolean {
  return normalizedRange.value.end === key
}

function isInRange(key: string): boolean {
  const { start, end } = normalizedRange.value
  if (!start || !end) return false
  return key > start && key < end
}

function applyRange(a: string, b: string): void {
  const [start, end] = a <= b ? [a, b] : [b, a]
  if (props.startDate !== start) emit('update:startDate', start)
  if (props.endDate !== end) emit('update:endDate', end)
}

function onCellPointerDown(key: string, e: PointerEvent): void {
  if (props.disabled || e.button !== 0) return
  e.preventDefault()

  isDragging.value = true
  didDragMove.value = false
  dragCurrent.value = key

  const { start, end } = normalizedRange.value
  // If dragging an existing range endpoint, anchor the opposite end so the user can stretch/shrink the bar
  if (start && end && start !== end) {
    if (key === start) {
      dragAnchor.value = end
    } else if (key === end) {
      dragAnchor.value = start
    } else {
      dragAnchor.value = key
    }
  } else {
    dragAnchor.value = key
  }

  window.addEventListener('pointermove', onWindowPointerMove)
  window.addEventListener('pointerup', onWindowPointerUp)
  window.addEventListener('pointercancel', onWindowPointerUp)
}

function onWindowPointerMove(e: PointerEvent): void {
  if (!isDragging.value || !dragAnchor.value) return
  const el = document.elementFromPoint(e.clientX, e.clientY)
  const cellEl = el?.closest?.('[data-cal-date]') as HTMLElement | null
  const targetKey = cellEl?.dataset.calDate
  if (!targetKey || targetKey === dragCurrent.value) return

  if (dragCurrent.value) {
    dragDir.value = targetKey >= dragCurrent.value ? 'forward' : 'backward'
  }
  didDragMove.value = true
  dragCurrent.value = targetKey
  applyRange(dragAnchor.value, targetKey)
}

function onWindowPointerUp(): void {
  if (!isDragging.value) return
  const clickedKey = dragCurrent.value
  const moved = didDragMove.value

  isDragging.value = false
  window.removeEventListener('pointermove', onWindowPointerMove)
  window.removeEventListener('pointerup', onWindowPointerUp)
  window.removeEventListener('pointercancel', onWindowPointerUp)

  if (!moved && clickedKey) {
    // Single click behavior: 1st click sets start date, 2nd click sets end date
    if (pickingStep.value === 'end' && props.startDate) {
      applyRange(props.startDate, clickedKey)
      pickingStep.value = 'start'
    } else {
      emit('update:startDate', clickedKey)
      emit('update:endDate', clickedKey)
      pickingStep.value = 'end'
    }
  } else if (moved) {
    pickingStep.value = 'start'
  }
}

function selectToday(): void {
  if (props.disabled) return
  const tKey = todayKey.value
  const parsed = parseYMD(tKey)
  if (parsed) {
    viewY.value = parsed.y
    viewM.value = parsed.m
  }
  emit('update:startDate', tKey)
  emit('update:endDate', tKey)
  pickingStep.value = 'end'
}

function clearDates(): void {
  if (props.disabled) return
  emit('update:startDate', null)
  emit('update:endDate', null)
  pickingStep.value = 'start'
}

onBeforeUnmount(() => {
  window.removeEventListener('pointermove', onWindowPointerMove)
  window.removeEventListener('pointerup', onWindowPointerUp)
  window.removeEventListener('pointercancel', onWindowPointerUp)
})
</script>

<template>
  <div
    class="camp-range-cal mt-2.5 mb-4 rounded-[20px] border border-[#ded9d1] bg-white p-3.5 sm:p-4 select-none transition"
    :class="{ 'opacity-60 pointer-events-none': disabled }"
  >
    <!-- Calendar Header -->
    <div class="flex flex-wrap items-center justify-between gap-2.5 pb-3.5 mb-3 border-b border-black/[0.06]">
      <div class="flex items-center gap-2">
        <button
          type="button"
          class="cal-nav-btn w-9 h-9 inline-flex items-center justify-center rounded-full border border-[#ded9d1] bg-white text-ink hover:border-[#111] transition cursor-pointer p-0 text-base leading-none"
          aria-label="Previous month"
          :disabled="disabled"
          @click="stepMonth(-1)"
        >
          ‹
        </button>
        <strong class="text-sm sm:text-[15px] font-bold text-ink tracking-tight min-w-[124px] text-center">
          {{ monthName(viewM) }} {{ viewY }}
        </strong>
        <button
          type="button"
          class="cal-nav-btn w-9 h-9 inline-flex items-center justify-center rounded-full border border-[#ded9d1] bg-white text-ink hover:border-[#111] transition cursor-pointer p-0 text-base leading-none"
          aria-label="Next month"
          :disabled="disabled"
          @click="stepMonth(1)"
        >
          ›
        </button>
      </div>

      <div class="flex items-center gap-2 ml-auto">
        <span
          v-if="dayCount > 0"
          class="inline-flex items-center justify-center h-9 px-3.5 rounded-full text-xs font-mono font-bold bg-[#111] text-white"
        >
          {{ dayCount }} {{ dayCount === 1 ? 'day' : 'days' }}
        </span>
        <span v-else class="text-xs text-muted hidden sm:inline mr-1">
          Drag across days to select range
        </span>
        <button
          type="button"
          class="cal-nav-btn w-auto h-9 inline-flex items-center justify-center px-4 rounded-full text-xs sm:text-[13px] font-semibold border border-[#ded9d1] bg-white text-black/80 hover:border-[#111] hover:text-black transition cursor-pointer"
          :disabled="disabled"
          @click="selectToday"
        >
          {{ t('campaign.today') }}
        </button>
        <button
          v-if="startDate || endDate"
          type="button"
          class="cal-nav-btn w-auto h-9 inline-flex items-center justify-center px-4 rounded-full text-xs sm:text-[13px] font-semibold border border-[#ded9d1] bg-white text-black/80 hover:border-[#111] hover:text-black transition cursor-pointer"
          :disabled="disabled"
          @click="clearDates"
        >
          {{ t('campaign.clearDate') }}
        </button>
      </div>
    </div>

    <!-- Day-of-week header -->
    <div class="grid grid-cols-7 mb-1">
      <span
        v-for="d in DOW"
        :key="d"
        class="text-center text-[10.5px] font-bold uppercase tracking-wider text-muted py-1"
      >
        {{ d }}
      </span>
    </div>

    <!-- Draggable Calendar Grid -->
    <div
      class="grid grid-cols-7 gap-y-1.5 touch-none pt-1"
      :class="{ 'is-dragging-cal': isDragging }"
      role="grid"
      aria-label="Campaign schedule calendar"
    >
      <div
        v-for="cell in cells"
        :key="cell.key"
        :data-cal-date="cell.key"
        class="cal-cell relative h-9 flex items-center justify-center cursor-ew-resize"
        :class="[
          isInRange(cell.key) ? 'in-range' : '',
          isStart(cell.key) ? 'is-start' : '',
          isEnd(cell.key) ? 'is-end' : '',
          cell.col === 0 ? 'col-start' : '',
          cell.col === 6 ? 'col-end' : '',
          dragDir === 'backward' ? 'dir-back' : 'dir-fwd',
        ]"
        @pointerdown="onCellPointerDown(cell.key, $event)"
      >
        <!-- Animated continuous range track -->
        <span class="cal-range-track" aria-hidden="true" />

        <!-- Floating live drag badge on active cursor cell -->
        <Transition name="cal-tip">
          <span
            v-if="isDragging && dragCurrent === cell.key"
            class="cal-drag-tip"
            aria-hidden="true"
          >
            {{ cell.key.slice(5) }} · {{ dayCount }}d
          </span>
        </Transition>

        <button
          type="button"
          tabindex="-1"
          :disabled="disabled"
          class="cal-day-btn w-8 h-8 rounded-full inline-flex items-center justify-center text-xs font-mono p-0 border-0 cursor-pointer relative z-10"
          :class="[
            isStart(cell.key) || isEnd(cell.key)
              ? 'is-endpoint bg-[#111] text-white font-bold'
              : isInRange(cell.key)
                ? 'is-mid bg-transparent text-ink font-semibold hover:bg-black/10'
                : cell.inMonth
                  ? 'bg-transparent text-black/80 hover:bg-black/[0.06] font-medium'
                  : 'bg-transparent text-black/30 hover:bg-black/[0.04]',
            isDragging && dragCurrent === cell.key ? 'is-active-drag' : '',
            cell.isToday && !isStart(cell.key) && !isEnd(cell.key)
              ? 'ring-1 ring-[#111]/40 font-bold'
              : '',
          ]"
          :aria-label="cell.key"
          :aria-selected="isStart(cell.key) || isEnd(cell.key) || isInRange(cell.key)"
        >
          {{ cell.day }}
        </button>
      </div>
    </div>

    <div class="mt-2.5 pt-2 border-t border-black/[0.05] flex items-center justify-between text-[11px] text-muted">
      <span>Drag or click to adjust start &amp; end dates</span>
      <span class="font-mono text-ink font-semibold">
        {{ startDate || '—' }} → {{ endDate || '—' }}
      </span>
    </div>
  </div>
</template>
