<script setup lang="ts">
// Calendar popup date picker: a field-like trigger button that opens a month
// grid in a modal. Emits YYYY-MM-DD (or null when cleared) like a date input.
import { computed, onUnmounted, ref, watch } from 'vue'
import { monthName, t } from '@/core/i18n'

const props = defineProps<{
  modelValue: string | null
  disabled?: boolean
  id?: string
  title?: string
}>()
const emit = defineEmits<{ 'update:modelValue': [v: string | null] }>()

// Short DOW headers by design (single letters); full i18n via t() later.
const DOW = ['S', 'M', 'T', 'W', 'T', 'F', 'S']

const open = ref(false)
const viewY = ref(0)
const viewM = ref(0)

function fromValue(value: string | null): { y: number; m: number } {
  const fallback = new Date()
  const d = value ? new Date(`${value}T00:00:00`) : fallback
  if (Number.isNaN(d.getTime())) return { y: fallback.getFullYear(), m: fallback.getMonth() + 1 }
  return { y: d.getFullYear(), m: d.getMonth() + 1 }
}

function show(): void {
  if (props.disabled) return
  const v = fromValue(props.modelValue)
  viewY.value = v.y
  viewM.value = v.m
  open.value = true
}

function close(): void {
  open.value = false
}

function onKey(e: KeyboardEvent): void {
  if (e.key === 'Escape') close()
}

watch(open, (v) => {
  if (v) window.addEventListener('keydown', onKey)
  else window.removeEventListener('keydown', onKey)
})
onUnmounted(() => window.removeEventListener('keydown', onKey))

function step(delta: number): void {
  const d = new Date(viewY.value, viewM.value - 1 + delta, 1)
  viewY.value = d.getFullYear()
  viewM.value = d.getMonth() + 1
}

const cells = computed<Array<number | null>>(() => {
  const lead = new Date(viewY.value, viewM.value - 1, 1).getDay()
  const days = new Date(viewY.value, viewM.value, 0).getDate()
  const out: Array<number | null> = []
  for (let i = 0; i < lead; i++) out.push(null)
  for (let d = 1; d <= days; d++) out.push(d)
  while (out.length % 7 !== 0) out.push(null)
  return out
})

function keyOf(day: number): string {
  return `${viewY.value}-${String(viewM.value).padStart(2, '0')}-${String(day).padStart(2, '0')}`
}

const todayKey = computed(() => {
  const n = new Date()
  return `${n.getFullYear()}-${String(n.getMonth() + 1).padStart(2, '0')}-${String(n.getDate()).padStart(2, '0')}`
})

function pick(day: number): void {
  emit('update:modelValue', keyOf(day))
  close()
}

function pickToday(): void {
  emit('update:modelValue', todayKey.value)
  close()
}

function clear(): void {
  emit('update:modelValue', null)
  close()
}
</script>

<template>
  <button type="button" class="dp-field w-full flex justify-between items-center gap-2 text-left cursor-pointer bg-transparent text-ink border-0 border-b-[1.5px] border-b-ink rounded-none px-0.5 py-2.5 disabled:opacity-55 disabled:cursor-default" :id="id" :disabled="disabled" @click="show">
    <span :class="!modelValue ? 'text-muted text-sm' : ''">{{ modelValue ?? t('campaign.pickDate') }}</span>
    <span class="flex-none text-muted text-xs" aria-hidden="true">▾</span>
  </button>
  <Teleport to="body">
  <div v-if="open" class="backdrop fixed inset-0 z-50 flex justify-center items-center p-4 bg-backdrop backdrop-blur-[4px]" @click.self="close">
    <div class="dp-modal w-full max-w-[340px] bg-surface border border-line rounded-none shadow-none p-5.5" role="dialog" aria-modal="true" :aria-label="title ?? t('campaign.pickDate')">
      <div class="dp-head flex justify-between items-center gap-2">
        <button type="button" class="dp-nav w-auto flex-none font-sans uppercase tracking-[0.08em] rounded-full px-5.5 py-3 text-[12.5px] font-semibold border-[1.5px] border-ink bg-transparent text-ink cursor-pointer shadow-none hover:bg-ink hover:text-white hover:border-ink active:transform-none disabled:opacity-50 disabled:cursor-default leading-[1.2]" aria-label="Previous month" @click="step(-1)">‹</button>
        <strong class="text-base">{{ monthName(viewM) }} {{ viewY }}</strong>
        <button type="button" class="dp-nav w-auto flex-none font-sans uppercase tracking-[0.08em] rounded-full px-5.5 py-3 text-[12.5px] font-semibold border-[1.5px] border-ink bg-transparent text-ink cursor-pointer shadow-none hover:bg-ink hover:text-white hover:border-ink active:transform-none disabled:opacity-50 disabled:cursor-default leading-[1.2]" aria-label="Next month" @click="step(1)">›</button>
      </div>
      <div class="grid grid-cols-7 gap-1 mt-4">
        <span v-for="(d, i) in DOW" :key="i" class="text-center text-muted text-2xs font-bold">{{ d }}</span>
        <template v-for="(c, i) in cells" :key="i">
          <span v-if="c === null" class="aspect-square min-w-0 p-0 text-sm border border-transparent rounded-none invisible" />
          <button v-else type="button" class="dp-day aspect-square min-w-0 p-0 text-sm border rounded-none cursor-pointer hover:bg-wash hover:border-accent-line" :class="[(keyOf(c) === modelValue ? 'selected bg-accent text-accent-ink border-accent font-bold' : keyOf(c) === todayKey ? 'border-accent bg-transparent text-ink' : 'border-transparent bg-transparent text-ink')]" :aria-selected="keyOf(c) === modelValue" :aria-current="keyOf(c) === todayKey ? 'date' : undefined" @click="pick(c)">{{ c }}</button>
        </template>
      </div>
      <div class="actions flex justify-end gap-2 mt-4">
        <button type="button" class="btn flex-1 min-w-0 font-sans uppercase tracking-[0.08em] rounded-full px-5.5 py-3 text-[12.5px] font-semibold border-[1.5px] border-ink bg-transparent text-ink cursor-pointer shadow-none hover:bg-ink hover:text-white hover:border-ink active:transform-none disabled:opacity-50 disabled:cursor-default" @click="clear">{{ t('campaign.clearDate') }}</button>
        <button type="button" class="btn flex-1 min-w-0 font-sans uppercase tracking-[0.08em] rounded-full px-5.5 py-3 text-[12.5px] font-semibold border-[1.5px] border-ink bg-transparent text-ink cursor-pointer shadow-none hover:bg-ink hover:text-white hover:border-ink active:transform-none disabled:opacity-50 disabled:cursor-default" @click="pickToday">{{ t('campaign.today') }}</button>
        <button type="button" class="btn flex-1 min-w-0 font-sans uppercase tracking-[0.08em] rounded-full px-5.5 py-3 text-[12.5px] font-semibold border-[1.5px] border-ink bg-ink text-white cursor-pointer shadow-none hover:bg-accent-hover hover:border-accent-hover hover:text-white active:transform-none disabled:opacity-50 disabled:cursor-default" @click="close">{{ t('common.close') }}</button>
      </div>
    </div>
  </div>
  </Teleport>
</template>
