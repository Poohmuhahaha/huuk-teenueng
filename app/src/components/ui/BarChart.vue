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
    variant?: 'bars' | 'towers' | 'area'
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

const gradientId = `wave-grad-${Math.random().toString(36).slice(2, 7)}`

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

const areaPoints = computed(() => {
  const vals = props.items.map((it) => num(it.value))
  if (!vals.length) return null

  const maxV = Math.max(...vals, 1)
  const width = 100
  const height = 44
  const padTop = 6
  const padBottom = 2
  const usableH = height - padTop - padBottom

  const n = props.items.length
  const sum = totalValue.value || 1

  const pts = props.items.map((it, i) => {
    const val = num(it.value)
    const norm = Math.max(0, Math.min(1, val / maxV))
    const x = n === 1 ? 50 : (i / (n - 1)) * width
    const y = height - padBottom - norm * usableH
    const pct = totalValue.value > 0 ? Math.round((val / sum) * 100) : 0

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
      x,
      y,
      val,
      pct,
      label: it.label,
      primaryLabel,
      secondaryLabel,
      color: itemColor(i, it.color),
    }
  })

  if (pts.length === 1) {
    const y = pts[0].y
    return {
      pts,
      linePath: `M 0 ${y} L 100 ${y}`,
      areaPath: `M 0 ${y} L 100 ${y} L 100 ${height} L 0 ${height} Z`,
      lastPt: pts[0],
    }
  }

  const segments: string[] = [`M ${pts[0].x.toFixed(2)} ${pts[0].y.toFixed(2)}`]
  for (let i = 0; i < pts.length - 1; i++) {
    const p0 = pts[Math.max(0, i - 1)]
    const p1 = pts[i]
    const p2 = pts[i + 1]
    const p3 = pts[Math.min(pts.length - 1, i + 2)]
    const cp1x = p1.x + (p2.x - p0.x) / 6
    const cp1y = p1.y + (p2.y - p0.y) / 6
    const cp2x = p2.x - (p3.x - p1.x) / 6
    const cp2y = p2.y - (p3.y - p1.y) / 6
    segments.push(
      `C ${cp1x.toFixed(2)} ${cp1y.toFixed(2)}, ${cp2x.toFixed(2)} ${cp2y.toFixed(2)}, ${p2.x.toFixed(2)} ${p2.y.toFixed(2)}`,
    )
  }

  const linePath = segments.join(' ')
  const areaPath = `${linePath} L 100.00 ${height.toFixed(2)} L 0 ${height.toFixed(2)} Z`

  return {
    pts,
    linePath,
    areaPath,
    lastPt: pts[pts.length - 1],
  }
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

    <!-- Variant: Area (Edge-to-Edge Smooth Spline Wave with Gradient Fade) -->
    <template v-else-if="variant === 'area'">
      <div class="w-full flex flex-col justify-end">
        <!-- Month Data Markers / Legend Row above the wave -->
        <div class="w-full flex items-center justify-between gap-2 px-1 pt-3 pb-3 text-ink">
          <div
            v-for="pt in areaPoints?.pts"
            :key="pt.label"
            class="flex flex-col items-center group/pt cursor-default transition-transform duration-200 hover:-translate-y-0.5"
            :title="`${pt.label}: ${currency}${pt.val.toLocaleString('en-US')} (${pt.pct}%)`"
          >
            <span class="text-[9.5px] font-semibold text-muted bg-wash border border-line/60 px-1.5 py-0.5 rounded-full mb-1 group-hover/pt:bg-black group-hover/pt:text-white transition-colors">
              {{ pt.pct }}%
            </span>
            <span class="text-xs sm:text-[13px] font-bold font-mono text-ink tracking-tight">
              {{ formatTowerValue(pt.val) }}
            </span>
            <span class="text-[11px] font-bold text-ink mt-0.5">
              {{ pt.primaryLabel }}
              <span v-if="pt.secondaryLabel" class="text-[9.5px] font-medium text-muted font-mono">{{ pt.secondaryLabel }}</span>
            </span>
          </div>
        </div>

        <!-- Full-width smooth bezier sparkline spanning edge-to-edge into card-fade corners -->
        <div class="relative -mx-4 -mb-4 sm:-mx-5 sm:-mb-5 mt-2 overflow-hidden rounded-b-[24px]">
          <svg
            viewBox="0 0 100 44"
            fill="none"
            preserveAspectRatio="none"
            class="h-16 sm:h-20 w-full block overflow-visible"
            aria-hidden="true"
          >
            <defs>
              <linearGradient :id="gradientId" x1="0" y1="0" x2="0" y2="1">
                <stop offset="0%" stop-color="#18181b" stop-opacity="0.22" />
                <stop offset="60%" stop-color="#18181b" stop-opacity="0.06" />
                <stop offset="100%" stop-color="#18181b" stop-opacity="0" />
              </linearGradient>
            </defs>

            <g v-if="areaPoints" class="sparkline-reveal">
              <!-- Gradient Area -->
              <path
                :d="areaPoints.areaPath"
                :fill="`url(#${gradientId})`"
                class="transition-all duration-500"
              />

              <!-- Stroke Line -->
              <path
                :d="areaPoints.linePath"
                stroke="#18181b"
                stroke-width="2"
                stroke-linecap="round"
                stroke-linejoin="round"
                vector-effect="non-scaling-stroke"
                class="transition-all duration-500"
              />

              <!-- Highlight Dots at each month point -->
              <circle
                v-for="(pt, idx) in areaPoints.pts"
                :key="pt.label"
                :cx="pt.x"
                :cy="pt.y"
                :r="idx === areaPoints.pts.length - 1 ? 3 : 2.5"
                fill="#18181b"
                stroke="#ffffff"
                stroke-width="1.5"
                vector-effect="non-scaling-stroke"
                class="transition-all duration-200 hover:scale-125"
              />
            </g>
          </svg>
        </div>
      </div>
    </template>

    <!-- Variant: Towers (Vertical 1:1 Square Blocks Matrix) -->
    <template v-else-if="variant === 'towers'">
      <div class="w-full flex items-end justify-center gap-5 sm:gap-8 pt-1 pb-1">
        <div
          v-for="(col, ci) in towersData"
          :key="col.label"
          class="flex-none w-12 sm:w-14 flex flex-col items-center justify-end group/col cursor-default"
          :title="`${col.label}: ${currency}${col.value.toLocaleString('en-US')} (${col.pct}% of total)`"
        >
          <!-- Top Metrics: Share Pill & Formatted Value -->
          <div class="flex flex-col items-center mb-2 transition-transform duration-200 group-hover/col:-translate-y-0.5">
            <span class="text-[9.5px] font-semibold text-muted bg-wash border border-line/60 px-1.5 py-0.5 rounded-full mb-1 transition-colors group-hover/col:bg-black group-hover/col:text-white">
              {{ col.pct }}%
            </span>
            <span class="text-[11px] sm:text-[12px] font-bold font-mono text-ink tracking-tight whitespace-nowrap">
              {{ formatTowerValue(col.value) }}
            </span>
          </div>

          <!-- Vertical Stack of 1:1 Squares -->
          <div class="flex flex-col-reverse gap-[2.5px] sm:gap-[3px] items-center my-1 select-none">
            <div
              v-for="r in numRows"
              :key="r"
              class="w-[18px] h-[18px] sm:w-[22px] sm:h-[22px] aspect-square rounded-[2px] sm:rounded-[2.5px] transition-all duration-200"
              :class="r <= col.activeLevel ? 'bg-ink tower-square-active group-hover/col:scale-110 group-hover/col:brightness-125' : 'bg-ink/[0.06] tower-square-ghost'"
              :style="{
                animationDelay: `${ci * 45 + (r - 1) * 20}ms`,
                opacity: r <= col.activeLevel ? getTowerSquareOpacity(r, col.activeLevel) : undefined,
                backgroundColor: r <= col.activeLevel && col.color && col.color !== '#0a0a0a' ? col.color : undefined,
              }"
            />
          </div>

          <!-- Bottom Label: Month / Category Name with subtle baseline separator -->
          <div class="w-full border-t border-line/40 pt-1.5 mt-1 flex flex-col items-center text-center min-w-0 max-w-full">
            <span class="text-[11px] sm:text-xs font-bold text-ink truncate max-w-full group-hover/col:text-brand-600 transition-colors">
              {{ col.primaryLabel }}
            </span>
            <span v-if="col.secondaryLabel" class="text-[9.5px] font-medium text-muted font-mono leading-none mt-0.5">
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
