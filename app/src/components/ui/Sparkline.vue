<script setup lang="ts">
import { computed, ref, useId, watch } from 'vue'

const props = withDefaults(
  defineProps<{
    values: number[]
    variant?: 'squares' | 'line'
    width?: number
    height?: number
    color?: string
    rows?: number
    columnsCount?: number
    showDot?: boolean
    showArea?: boolean
  }>(),
  {
    variant: 'squares',
    width: 100,
    height: 36,
    color: '#18181b',
    rows: 4,
    columnsCount: 24,
    showDot: true,
    showArea: true,
  },
)

const uid = useId()
const gradientId = computed(() => `spark-${uid.replace(/:/g, '')}`)

const animKey = ref(0)
watch(
  () => props.values,
  () => {
    animKey.value += 1
  },
  { deep: true },
)

function num(v: unknown): number {
  return typeof v === 'number' && Number.isFinite(v) ? Math.max(0, v) : 0
}

const clean = computed(() => props.values.map(num))

/** Resamples values into dense 1:1 square columns */
const matrixColumns = computed(() => {
  const vals = clean.value
  const targetLen = Math.max(16, Math.min(32, props.columnsCount || 24))
  if (!vals.length) {
    return Array.from({ length: targetLen }, () => ({ activeLevel: 0, val: 0 }))
  }

  const n = vals.length
  const resampled: number[] = []
  if (n === 1) {
    for (let i = 0; i < targetLen; i++) resampled.push(vals[0])
  } else {
    for (let i = 0; i < targetLen; i++) {
      const t = (i / (targetLen - 1)) * (n - 1)
      const idx = Math.floor(t)
      const frac = t - idx
      if (idx >= n - 1) {
        resampled.push(vals[n - 1])
      } else {
        resampled.push(vals[idx] * (1 - frac) + vals[idx + 1] * frac)
      }
    }
  }

  const minV = Math.min(...resampled)
  const maxV = Math.max(...resampled)
  const range = maxV === minV ? 1 : maxV - minV
  const maxRows = props.rows || 4

  return resampled.map((v) => {
    if (v === 0 && minV === 0) return { activeLevel: 1, val: v }
    const norm = Math.max(0, Math.min(1, (v - minV) / range))
    const activeLevel = Math.max(1, Math.min(maxRows, Math.round(norm * (maxRows - 1)) + 1))
    return { activeLevel, val: v }
  })
})

function getSquareOpacity(row: number, activeLevel: number): number {
  if (row === activeLevel) return 0.95
  return Math.max(0.4, 0.95 - (activeLevel - row) * 0.16)
}

/** Smooth Bezier Line Curve (fallback / alternative mode) */
const curve = computed(() => {
  const vals = clean.value
  if (!vals.length) return null
  const minV = Math.min(...vals)
  const maxV = Math.max(...vals)
  const range = maxV === minV ? 1 : maxV - minV
  const padTop = 4
  const padBottom = 4
  const usableH = props.height - padTop - padBottom
  const n = vals.length

  if (n === 1) {
    return {
      single: { x: props.width / 2, y: props.height / 2 },
      linePath: '',
      areaPath: '',
      lastPt: { x: props.width / 2, y: props.height / 2 },
    }
  }

  const pts: { x: number; y: number }[] = vals.map((v, i) => ({
    x: Number(((i / (n - 1)) * props.width).toFixed(2)),
    y: Number((props.height - padBottom - ((v - minV) / range) * usableH).toFixed(2)),
  }))

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
  const areaPath = `${linePath} L ${props.width.toFixed(2)} ${props.height.toFixed(2)} L 0 ${props.height.toFixed(2)} Z`

  return {
    single: null,
    linePath,
    areaPath,
    lastPt: pts[pts.length - 1],
  }
})
</script>

<template>
  <!-- Variant: Dense 1:1 Square Block Matrix with Staggered Entrance Animation -->
  <div
    v-if="variant === 'squares'"
    :key="`sq-${animKey}`"
    class="w-full flex items-end gap-[2.5px] sm:gap-[3px] select-none py-1 group/matrix"
    aria-hidden="true"
  >
    <div
      v-for="(col, ci) in matrixColumns"
      :key="ci"
      class="flex-1 flex flex-col-reverse gap-[2.5px] sm:gap-[3px] transition-transform duration-200 group-hover:scale-y-[1.04] origin-bottom"
    >
      <div
        v-for="r in rows"
        :key="r"
        class="w-full aspect-square rounded-[1.5px] sm:rounded-[2px] transition-all duration-200"
        :class="r <= col.activeLevel ? 'bg-ink square-active' : 'bg-ink/[0.06] square-ghost'"
        :style="{
          opacity: r <= col.activeLevel ? getSquareOpacity(r, col.activeLevel) : undefined,
          '--target-opacity': r <= col.activeLevel ? getSquareOpacity(r, col.activeLevel) : 1,
          animationDelay: `${ci * 25 + (r - 1) * 35}ms`,
        }"
      />
    </div>
  </div>

  <!-- Variant: Smooth Bezier Line Curve -->
  <svg
    v-else-if="curve"
    :viewBox="`0 0 ${width} ${height}`"
    fill="none"
    preserveAspectRatio="none"
    class="h-9 w-full block"
    aria-hidden="true"
  >
    <defs v-if="showArea">
      <linearGradient :id="gradientId" x1="0" y1="0" x2="0" y2="1">
        <stop offset="0%" :stop-color="color" stop-opacity="0.18" />
        <stop offset="100%" :stop-color="color" stop-opacity="0" />
      </linearGradient>
    </defs>
    <g :key="`ln-${animKey}`" class="sparkline-reveal">
      <circle v-if="curve.single" :cx="curve.single.x" :cy="curve.single.y" r="2.5" :fill="color" />
      <template v-else>
        <path v-if="showArea" :d="curve.areaPath" :fill="`url(#${gradientId})`" class="sparkline-area" />
        <path
          :d="curve.linePath"
          :stroke="color"
          stroke-width="1.8"
          stroke-linecap="round"
          stroke-linejoin="round"
          vector-effect="non-scaling-stroke"
        />
        <circle
          v-if="showDot && curve.lastPt"
          :cx="curve.lastPt.x"
          :cy="curve.lastPt.y"
          r="2.5"
          :fill="color"
          vector-effect="non-scaling-stroke"
          class="sparkline-dot"
        />
      </template>
    </g>
  </svg>
</template>

<style scoped>
/* Square Matrix Entrance Animation */
.square-active {
  animation: squarePopIn 0.45s cubic-bezier(0.16, 1, 0.3, 1) both;
}

.square-ghost {
  animation: ghostFadeIn 0.35s ease-out both;
}

@keyframes squarePopIn {
  0% {
    opacity: 0;
    transform: scale(0.3) translateY(4px);
  }
  60% {
    transform: scale(1.15) translateY(-0.5px);
  }
  100% {
    opacity: var(--target-opacity, 1);
    transform: scale(1) translateY(0);
  }
}

@keyframes ghostFadeIn {
  from {
    opacity: 0;
  }
  to {
    opacity: 1;
  }
}

/* Line Mode Animations */
.sparkline-reveal {
  animation: revealSpark 0.95s cubic-bezier(0.16, 1, 0.3, 1) forwards;
}

@keyframes revealSpark {
  from {
    clip-path: inset(0 100% 0 0);
  }
  to {
    clip-path: inset(0 0% 0 0);
  }
}

.sparkline-area {
  animation: fadeInSparkArea 1.1s ease-out forwards;
}

@keyframes fadeInSparkArea {
  from {
    opacity: 0;
  }
  to {
    opacity: 1;
  }
}

.sparkline-dot {
  animation: popInDot 0.4s 0.8s cubic-bezier(0.34, 1.56, 0.64, 1) both;
}

@keyframes popInDot {
  from {
    opacity: 0;
    transform: scale(0);
  }
  to {
    opacity: 1;
    transform: scale(1);
  }
}

@media (prefers-reduced-motion: reduce) {
  .square-active,
  .square-ghost,
  .sparkline-reveal,
  .sparkline-area,
  .sparkline-dot {
    animation: none !important;
  }
}
</style>
