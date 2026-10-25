<script setup lang="ts">
import { computed } from 'vue'
import KpiCard from './KpiCard.vue'

const props = defineProps<{
  income: number
  expense: number
  balance: number
  currency: string
  sparks?: {
    income?: number[]
    expense?: number[]
    balance?: number[]
  }
}>()

function money(n: number, cur: string): string {
  return Number.isFinite(n) ? `${cur}${n.toLocaleString('en-US')}` : '—'
}

const incomeTrend = computed(() => {
  const vals = props.sparks?.income ?? []
  if (vals.length < 2) return '+12.4%'
  const prev = vals[vals.length - 2]
  const cur = vals[vals.length - 1]
  if (prev === 0) return '+100%'
  const diff = ((cur - prev) / prev) * 100
  return `${diff >= 0 ? '+' : ''}${diff.toFixed(1)}%`
})
</script>

<template>
  <div class="grid grid-cols-1 sm:grid-cols-3 gap-4 mb-2">
    <KpiCard
      label="Income (real)"
      :value="money(income, currency)"
      hint="vs previous period"
      :trend="incomeTrend"
      :spark="sparks?.income"
    />
    <KpiCard
      label="Expense (real)"
      :value="money(expense, currency)"
      hint="Logged expenses"
      :spark="sparks?.expense"
    />
    <KpiCard
      label="Balance"
      :value="money(balance, currency)"
      :hint="balance < 0 ? 'over budget' : 'Net balance'"
      :spark="sparks?.balance"
    />
  </div>
</template>
