<script setup lang="ts">
import { computed } from 'vue'
import PlatformLogo from './PlatformLogo.vue'

export interface BarItem {
  label: string
  value: number
  total?: number
  sublabel?: string
  meta?: string
  posted?: number
  pending?: number
  wip?: number
  /** Optional brand color; falls back to the monochrome palette. */
  color?: string
  /** Optional platform id/name so the row shows the brand glyph. */
  platform?: string
}

const props = withDefaults(
  defineProps<{
    items: BarItem[]
    variant?: 'bars' | 'towers'
    currency?: string
    card?: boolean
    rows?: number
  }>(),
  {
    variant: 'bars',
    currency: '',
    card: true,
    rows: 8,
  },
)

function num(v: unknown): number {
  return typeof v === 'number' && Number.isFinite(v) ? Math.max(0, v) : 0
}

// Monochrome palette — ink black, charcoal, and grayscale shades. Items can
// override with a brand color (e.g. platform charts).
const colors = ['#0a0a0a', '#404040', '#666666', '#8c8c8c', '#262626', '#b3b3b3']
function itemColor(i: number, custom?: string): string {
  return custom ?? colors[i % colors.length]
}

const totalValue = computed(() => props.items.reduce((acc, it) => acc + num(it.value), 0))
const maxVal = computed(() => {
  const vals = props.items.map((it) => num(it.value))
  return Math.max(...vals, 1)
})

const numRows = computed(() => Math.max(3, Math.min(16, props.rows ?? 8)))

const itemsWithShare = computed(() => {
  const sum = totalValue.value || 1
  return props.items.map((it, i) => {
    const val = num(it.value)
    const pct = totalValue.value > 0 ? Math.round((val / sum) * 100) : 0
    // When posted counts are available the fill bar tracks real posting
    // progress (matching the "% posted" caption) instead of repeating the
    // share-of-total already shown by the stacked track and the % badge.
    const progress = it.posted !== undefined ? Math.round((num(it.posted) / (val || 1)) * 100) : pct
    return {
      ...it,
      value: val,
      pct,
      progress,
      color: itemColor(i, it.color),
    }
  })
})

const towersData = computed(() => {
  const max = maxVal.value || 1
  const sum = totalValue.value || 1
  const rowCount = numRows.value

  return props.items.map((it, colIdx) => {
    const val = num(it.value)
    const pct = totalValue.value > 0 ? Math.round((val / sum) * 100) : 0
    const activeLevel = val > 0 ? Math.max(1, Math.min(rowCount, Math.round((val / max) * rowCount))) : 0

    // Date formatting if label is YYYY-MM
    const dateMatch = /^(\d{4})-(\d{2})$/.exec(it.label)
    let primaryLabel = it.label
    let secondaryLabel = ''
    if (dateMatch) {
      const monthNames = ['Jan', 'Feb', 'Mar', 'Apr', 'May', 'Jun', 'Jul', 'Aug', 'Sep', 'Oct', 'Nov', 'Dec']
      const mIdx = parseInt(dateMatch[2], 10) - 1
      primaryLabel = monthNames[mIdx] ?? dateMatch[2]
      secondaryLabel = `'${dateMatch[1].slice(2)}`
    }

    return {
      ...it,
      value: val,
      pct,
      activeLevel,
      primaryLabel,
      secondaryLabel,
      color: itemColor(colIdx, it.color),
    }
  })
})

function formatTowerValue(v: number): string {
  const cur = props.currency ? props.currency : ''
  if (v >= 1_000_000) {
    return `${cur}${(v / 1_000_000).toFixed(1)}M`
  }
  if (v >= 100_000) {
    return `${cur}${(v / 1_000).toFixed(0)}k`
  }
  return `${cur}${v.toLocaleString('en-US')}`
}

function getTowerSquareOpacity(row: number, activeLevel: number): number {
  if (row === activeLevel) return 1.0
  return Math.max(0.5, 1.0 - (activeLevel - row) * 0.08)
}

const summary = computed(() => props.items.map((i) => `${i.label}:${num(i.value)}`).join(', '))
</script>

<template>
  <div
    :class="card ? 'border border-line rounded-xl bg-surface p-4 shadow-xs transition-all' : 'w-full transition-all'"
    role="img"
    :aria-label="summary"
  >
    <div v-if="!items.length" class="text-muted text-sm py-8 text-center">No data yet.</div>

    <!-- Variant: Towers (Vertical 1:1 Square Blocks Matrix) -->
    <template v-else-if="variant === 'towers'">
      <div class="w-full flex items-end justify-between gap-2 sm:gap-3 pt-2 pb-1">
        <div
          v-for="(col, ci) in towersData"
          :key="col.label"
          class="flex-1 min-w-0 flex flex-col items-center justify-end max-w-[64px] group/col cursor-default"
          :title="`${col.label}: ${currency}${col.value.toLocaleString('en-US')} (${col.pct}% of total)`"
        >
          <!-- Top Metrics: Share Pill & Formatted Value -->
          <div class="flex flex-col items-center mb-2.5 transition-transform duration-200 group-hover/col:-translate-y-0.5">
            <span class="text-[10px] font-semibold text-muted bg-wash border border-line/60 px-1.5 py-0.5 rounded-full mb-1 transition-colors group-hover/col:bg-black group-hover/col:text-white">
              {{ col.pct }}%
            </span>
            <span class="text-xs sm:text-[13px] font-bold font-mono text-ink tracking-tight whitespace-nowrap">
              {{ formatTowerValue(col.value) }}
            </span>
          </div>

          <!-- Vertical Stack of 1:1 Squares -->
          <div class="w-full flex flex-col-reverse gap-1 sm:gap-1.5 items-center my-1 select-none">
            <div
              v-for="r in numRows"
              :key="r"
              class="w-full max-w-[34px] sm:max-w-[40px] aspect-square rounded-[2px] sm:rounded-[3px] transition-all duration-200"
              :class="r <= col.activeLevel ? 'bg-ink tower-square-active group-hover/col:scale-105 group-hover/col:brightness-125' : 'bg-ink/[0.06] tower-square-ghost'"
              :style="{
                animationDelay: `${ci * 45 + (r - 1) * 20}ms`,
                opacity: r <= col.activeLevel ? getTowerSquareOpacity(r, col.activeLevel) : undefined,
                backgroundColor: r <= col.activeLevel && col.color && col.color !== '#0a0a0a' ? col.color : undefined,
              }"
            />
          </div>

          <!-- Bottom Label: Month / Category Name -->
          <div class="flex flex-col items-center mt-2 text-center min-w-0 max-w-full">
            <span class="text-xs font-bold text-ink truncate max-w-full group-hover/col:text-brand-600 transition-colors">
              {{ col.primaryLabel }}
            </span>
            <span v-if="col.secondaryLabel" class="text-[10px] font-medium text-muted font-mono leading-none mt-0.5">
              {{ col.secondaryLabel }}
            </span>
          </div>
        </div>
      </div>
    </template>

    <!-- Variant: Bars (Horizontal Progress Rows) -->
    <template v-else>
      <div class="space-y-3">
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

        <!-- Granular rows (no nested cards) -->
        <div class="grid">
          <div
            v-for="it in itemsWithShare"
            :key="it.label"
            class="group border-b border-line/40 py-2.5 last:border-b-0"
          >
            <!-- Top line: label + swatch + count + percentage -->
            <div class="flex items-center justify-between gap-2 mb-1.5">
              <div class="flex items-center gap-2 min-w-0">
                <span v-if="it.platform" class="flex-none" :style="{ color: it.color }">
                  <PlatformLogo :name="it.platform" :size="13" />
                </span>
                <span v-else class="w-2.5 h-2.5 rounded-full flex-none shadow-2xs" :style="{ background: it.color }" />
                <span class="text-[13px] font-bold text-ink truncate">{{ it.label }}</span>
              </div>
              <div class="flex items-center gap-2 flex-none">
                <span class="text-sm font-bold text-ink font-mono">{{ it.value.toLocaleString('en-US') }}</span>
                <span class="text-[11px] font-semibold text-muted bg-wash border border-line px-1.5 py-0.5 rounded-full">
                  {{ it.pct }}%
                </span>
              </div>
            </div>

            <!-- Progress fill bar: posted progress when known, share otherwise -->
            <div class="w-full h-2 bg-wash rounded-full overflow-hidden border border-line/40 mb-2">
              <div
                class="h-full rounded-full transition-all duration-500"
                :style="{ width: `${it.progress}%`, background: it.color }"
              />
            </div>

            <!-- Sub-metrics metadata (Planned vs Posted, etc.) -->
            <div v-if="it.sublabel || it.posted !== undefined" class="flex items-center justify-between text-[11px] text-muted pt-0.5">
              <span class="truncate">{{ it.sublabel }}</span>
              <span
                v-if="it.posted !== undefined"
                class="font-semibold flex-none ml-2"
                :class="it.progress > 0 ? 'text-ink' : 'text-muted'"
              >
                {{ it.progress }}% posted
              </span>
            </div>
          </div>
        </div>
      </div>
    </template>
  </div>
</template>

<style scoped>
.tower-square-active {
  animation: squarePopIn 0.35s cubic-bezier(0.16, 1, 0.3, 1) both;
}

.tower-square-ghost {
  animation: ghostFadeIn 0.4s ease-out both;
}

@keyframes squarePopIn {
  0% {
    opacity: 0;
    transform: scale(0.65);
  }
  60% {
    transform: scale(1.06);
  }
  100% {
    opacity: 1;
    transform: scale(1);
  }
}

@keyframes ghostFadeIn {
  0% {
    opacity: 0;
  }
  100% {
    opacity: 1;
  }
}
</style>
