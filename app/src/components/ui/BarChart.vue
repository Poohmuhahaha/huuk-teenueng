<script setup lang="ts">
import { computed, ref, watch } from 'vue'

export interface BarItem {
  label: string
  value: number
  total?: number
  sublabel?: string
  meta?: string
  posted?: number
  pending?: number
  wip?: number
}

const props = withDefaults(
  defineProps<{
    items: BarItem[]
    variant?: 'auto' | 'bars' | 'breakdown'
    unit?: string
  }>(),
  { variant: 'auto', unit: 'topics' },
)

function num(v: unknown): number {
  return typeof v === 'number' && Number.isFinite(v) ? Math.max(0, v) : 0
}

// Strictly monochrome palette — ink black, charcoal, and grayscale shades
const colors = ['#0a0a0a', '#404040', '#666666', '#8c8c8c', '#262626', '#b3b3b3']
function itemColor(i: number): string {
  return colors[i % colors.length]
}

const totalValue = computed(() => props.items.reduce((acc, it) => acc + num(it.value), 0))
const max = computed(() => Math.max(1, ...props.items.map((x) => num(x.value))))

const viewMode = ref<'breakdown' | 'bars'>(
  props.variant === 'bars' ? 'bars' : props.variant === 'breakdown' || props.items.length <= 4 ? 'breakdown' : 'bars',
)

watch(
  () => props.items.length,
  (len) => {
    if (props.variant === 'auto') {
      viewMode.value = len <= 4 ? 'breakdown' : 'bars'
    }
  },
)

const itemsWithShare = computed(() => {
  const sum = totalValue.value || 1
  return props.items.map((it, i) => {
    const val = num(it.value)
    const pct = totalValue.value > 0 ? Math.round((val / sum) * 100) : 0
    return {
      ...it,
      value: val,
      pct,
      color: itemColor(i),
    }
  })
})

const summary = computed(() => props.items.map((i) => `${i.label}:${num(i.value)}`).join(', '))

function height(value: number): string {
  const v = num(value)
  if (v <= 0) return '0%'
  return `${Math.max(6, Math.round((v / max.value) * 100))}%`
}
</script>

<template>
  <div
    class="border border-line rounded-xl bg-surface p-4 shadow-xs transition-all"
    role="img"
    :aria-label="summary"
  >
    <div v-if="!items.length" class="text-muted text-sm py-8 text-center">No data yet.</div>
    <template v-else>
      <!-- Top header with quick stats & view switcher -->
      <div class="flex items-center justify-between gap-2 pb-3 mb-2 border-b border-line/60">
        <div class="flex items-center gap-2">
          <span class="text-xs font-semibold text-ink">
            {{ totalValue.toLocaleString('en-US') }} {{ unit }}
          </span>
          <span class="text-[11px] font-medium text-muted bg-wash border border-line px-2 py-0.5 rounded-full">
            {{ items.length }} {{ items.length === 1 ? 'item' : 'items' }}
          </span>
        </div>

        <div class="flex items-center gap-1 bg-wash p-0.5 rounded-lg border border-line/70">
          <button
            type="button"
            class="px-2 py-1 text-[11px] font-semibold rounded-md transition-all cursor-pointer flex items-center gap-1"
            :class="viewMode === 'breakdown' ? 'bg-surface text-ink shadow-xs border border-line/50' : 'text-muted hover:text-ink'"
            title="Rich breakdown view"
            @click="viewMode = 'breakdown'"
          >
            <svg viewBox="0 0 16 16" width="12" height="12" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round">
              <line x1="2" y1="4" x2="14" y2="4" />
              <line x1="2" y1="8" x2="14" y2="8" />
              <line x1="2" y1="12" x2="14" y2="12" />
            </svg>
            <span>Breakdown</span>
          </button>
          <button
            type="button"
            class="px-2 py-1 text-[11px] font-semibold rounded-md transition-all cursor-pointer flex items-center gap-1"
            :class="viewMode === 'bars' ? 'bg-surface text-ink shadow-xs border border-line/50' : 'text-muted hover:text-ink'"
            title="Chart view"
            @click="viewMode = 'bars'"
          >
            <svg viewBox="0 0 16 16" width="12" height="12" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round">
              <rect x="2" y="7" width="3" height="7" rx="0.5" />
              <rect x="6.5" y="3" width="3" height="11" rx="0.5" />
              <rect x="11" y="9" width="3" height="5" rx="0.5" />
            </svg>
            <span>Bars</span>
          </button>
        </div>
      </div>

      <!-- VIEW 1: Rich Detailed Breakdown (Choice 2) -->
      <div v-if="viewMode === 'breakdown'" class="space-y-3">
        <!-- Segmented Distribution Track (when there are multiple items) -->
        <div v-if="itemsWithShare.length > 1" class="space-y-1">
          <div class="w-full h-2.5 bg-wash rounded-full overflow-hidden flex gap-[2px] p-[2px] border border-line">
            <div
              v-for="it in itemsWithShare"
              :key="it.label"
              class="h-full rounded-full transition-all duration-300"
              :style="{ width: `${it.pct}%`, background: it.color }"
              :title="`${it.label}: ${it.value} (${it.pct}%)`"
            />
          </div>
        </div>

        <!-- Granular item cards -->
        <div class="grid gap-2.5">
          <div
            v-for="it in itemsWithShare"
            :key="it.label"
            class="group rounded-lg border border-line/60 p-3 bg-surface hover:bg-wash/50 transition-colors"
          >
            <!-- Top line: label + swatch + count + percentage -->
            <div class="flex items-center justify-between gap-2 mb-1.5">
              <div class="flex items-center gap-2 min-w-0">
                <span class="w-2.5 h-2.5 rounded-full flex-none shadow-2xs" :style="{ background: it.color }" />
                <span class="text-[13px] font-bold text-ink truncate">{{ it.label }}</span>
              </div>
              <div class="flex items-center gap-2 flex-none">
                <span class="text-sm font-bold text-ink font-mono">{{ it.value.toLocaleString('en-US') }}</span>
                <span class="text-[11px] font-semibold text-muted bg-wash border border-line px-1.5 py-0.5 rounded-full">
                  {{ it.pct }}%
                </span>
              </div>
            </div>

            <!-- Proportional progress fill bar -->
            <div class="w-full h-2 bg-wash rounded-full overflow-hidden border border-line/40 mb-2">
              <div
                class="h-full rounded-full transition-all duration-500"
                :style="{ width: `${it.pct}%`, background: it.color }"
              />
            </div>

            <!-- Sub-metrics metadata (Planned vs Posted, etc.) -->
            <div v-if="it.sublabel || it.posted !== undefined" class="flex items-center justify-between text-[11px] text-muted pt-0.5">
              <span class="truncate">{{ it.sublabel }}</span>
              <span v-if="it.posted !== undefined" class="font-semibold text-ink flex-none ml-2">
                {{ Math.round((it.posted / (it.value || 1)) * 100) }}% posted
              </span>
            </div>
          </div>
        </div>
      </div>

      <!-- VIEW 2: Classic Bars Chart -->
      <div v-else class="relative h-[130px] pt-4 pb-1 px-2">
        <!-- Subtle horizontal grid lines -->
        <div class="absolute inset-x-2 inset-y-2 flex flex-col justify-between pointer-events-none opacity-25">
          <div class="border-b border-line w-full"></div>
          <div class="border-b border-line w-full"></div>
          <div class="border-b border-line w-full"></div>
        </div>

        <!-- Bars container -->
        <div class="relative flex items-end justify-around gap-3 h-full z-1">
          <div
            v-for="(it, i) in itemsWithShare"
            :key="`${it.label}-${i}`"
            class="group flex flex-col items-center flex-1 max-w-[56px] min-w-[28px] h-full justify-end cursor-pointer"
            :title="`${it.label}: ${it.value}`"
          >
            <!-- Value label above bar -->
            <span class="text-[11px] font-bold text-ink mb-1 group-hover:scale-110 transition-transform">
              {{ it.value.toLocaleString('en-US') }}
            </span>

            <!-- Column track with filled bar -->
            <div class="w-full h-full max-h-[85px] bg-wash/80 rounded-t-lg flex items-end overflow-hidden p-0.5">
              <div
                class="w-full rounded-t-md transition-all duration-300 group-hover:brightness-110"
                :style="{ height: height(it.value), background: it.color }"
              />
            </div>

            <!-- X-axis label -->
            <span class="text-[11px] text-muted font-medium mt-1.5 truncate max-w-full text-center group-hover:text-ink transition-colors">
              {{ it.label }}
            </span>
          </div>
        </div>
      </div>
    </template>
  </div>
</template>
