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
  <button type="button" class="field dp-field" :id="id" :disabled="disabled" @click="show">
    <span :class="{ muted: !modelValue }">{{ modelValue ?? t('campaign.pickDate') }}</span>
    <span class="dp-caret" aria-hidden="true">▾</span>
  </button>
  <Teleport to="body">
  <div v-if="open" class="backdrop" @click.self="close">
    <div class="modal dp-modal" role="dialog" aria-modal="true" :aria-label="title ?? t('campaign.pickDate')">
      <div class="dp-head">
        <button type="button" class="btn dp-nav" aria-label="Previous month" @click="step(-1)">‹</button>
        <strong>{{ monthName(viewM) }} {{ viewY }}</strong>
        <button type="button" class="btn dp-nav" aria-label="Next month" @click="step(1)">›</button>
      </div>
      <div class="dp-grid mt">
        <span v-for="(d, i) in DOW" :key="i" class="dp-dow">{{ d }}</span>
        <template v-for="(c, i) in cells" :key="i">
          <span v-if="c === null" class="dp-day empty" />
          <button v-else type="button" class="dp-day" :class="{ selected: keyOf(c) === modelValue, today: keyOf(c) === todayKey }" @click="pick(c)">{{ c }}</button>
        </template>
      </div>
      <div class="actions">
        <button type="button" class="btn" @click="clear">{{ t('campaign.clearDate') }}</button>
        <button type="button" class="btn" @click="pickToday">{{ t('campaign.today') }}</button>
        <button type="button" class="btn btn-primary" @click="close">{{ t('common.close') }}</button>
      </div>
    </div>
  </div>
  </Teleport>
</template>
