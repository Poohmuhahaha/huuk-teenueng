<script setup lang="ts">
import Sparkline from './Sparkline.vue'

defineProps<{
  label: string
  value: string | number
  hint?: string
  to?: string
  interactive?: boolean
  spark?: number[]
}>()
const emit = defineEmits<{ select: [] }>()
function pick(): void {
  emit('select')
}
</script>

<template>
  <RouterLink v-if="to" class="group border border-line rounded-xl bg-surface shadow-xs hover:shadow-md hover:border-ink transition-all relative flex flex-col justify-between p-4 text-left no-underline text-inherit focus-visible:border-ink focus-visible:ring-2 focus-visible:ring-accent" :to="to">
    <div class="flex flex-col gap-2">
      <div class="flex items-center justify-between gap-2">
        <span class="text-muted text-[11px] font-bold uppercase tracking-wider">{{ label }}</span>
        <svg class="opacity-0 transition-all transform group-hover:opacity-60 group-hover:translate-x-0.5 group-hover:-translate-y-0.5" viewBox="0 0 16 16" width="13" height="13" fill="none" stroke="currentColor"
          stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
          <path d="M4 12 12 4M6 4h6v6" />
        </svg>
      </div>
      <div class="text-2xl sm:text-[26px] font-black tracking-tight leading-none text-ink font-mono">{{ value }}</div>
      <div v-if="hint" class="text-muted text-xs leading-[1.3]">{{ hint }}</div>
    </div>
    <div v-if="spark && spark.length" class="text-accent/70 group-hover:text-accent block mt-3 h-[22px] transition-colors"><Sparkline :values="spark" /></div>
  </RouterLink>
  <div v-else-if="interactive" class="group border border-line rounded-xl bg-surface shadow-xs hover:shadow-md hover:border-ink transition-all relative flex flex-col justify-between p-4 text-left no-underline text-inherit focus-visible:border-ink focus-visible:ring-2 focus-visible:ring-accent cursor-pointer" role="button" tabindex="0" :aria-label="label"
    @click="pick" @keydown.enter.prevent="pick" @keydown.space.prevent="pick">
    <div class="flex flex-col gap-2">
      <div class="flex items-center justify-between gap-2">
        <span class="text-muted text-[11px] font-bold uppercase tracking-wider">{{ label }}</span>
        <svg class="opacity-0 transition-all transform group-hover:opacity-60 group-hover:translate-x-0.5 group-hover:-translate-y-0.5" viewBox="0 0 16 16" width="13" height="13" fill="none" stroke="currentColor"
          stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
          <path d="M4 12 12 4M6 4h6v6" />
        </svg>
      </div>
      <div class="text-2xl sm:text-[26px] font-black tracking-tight leading-none text-ink font-mono">{{ value }}</div>
      <div v-if="hint" class="text-muted text-xs leading-[1.3]">{{ hint }}</div>
    </div>
    <div v-if="spark && spark.length" class="text-accent/70 group-hover:text-accent block mt-3 h-[22px] transition-colors"><Sparkline :values="spark" /></div>
  </div>
  <div v-else class="border border-line rounded-xl bg-surface shadow-xs relative flex flex-col justify-between p-4 text-left">
    <div class="flex flex-col gap-2">
      <div class="flex items-center justify-between gap-2">
        <span class="text-muted text-[11px] font-bold uppercase tracking-wider">{{ label }}</span>
        <span v-if="hint" class="text-2xs px-2 py-0.5 rounded-full font-medium" :class="hint.includes('over') ? 'bg-rose-50 text-rose-600 border border-rose-200' : 'bg-wash text-muted'">{{ hint }}</span>
      </div>
      <div class="text-2xl sm:text-[26px] font-black tracking-tight leading-none text-ink font-mono">{{ value }}</div>
    </div>
    <div v-if="spark && spark.length" class="text-accent/70 block mt-3 h-[22px]"><Sparkline :values="spark" /></div>
  </div>
</template>
