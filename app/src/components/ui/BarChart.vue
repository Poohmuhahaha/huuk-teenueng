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

const props = defineProps<{
  items: BarItem[]
}>()

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

const summary = computed(() => props.items.map((i) => `${i.label}:${num(i.value)}`).join(', '))
</script>

<template>
  <div
    class="border border-line rounded-xl bg-surface p-4 shadow-xs transition-all"
    role="img"
    :aria-label="summary"
  >
    <div v-if="!items.length" class="text-muted text-sm py-8 text-center">No data yet.</div>
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
