<script setup lang="ts">
import KpiCard from './KpiCard.vue'

defineProps<{ income: number; expense: number; balance: number; currency: string }>()

function money(n: number, cur: string): string {
  return Number.isFinite(n) ? `${cur}${n.toLocaleString('en-US')}` : '—'
}
</script>

<template>
  <div class="grid3 grid gap-3">
    <KpiCard label="Income (real)" :value="money(income, currency)" />
    <KpiCard label="Expense (real)" :value="money(expense, currency)" />
    <KpiCard label="Balance" :value="money(balance, currency)" :hint="balance < 0 ? 'over budget' : undefined"
      :class="balance < 0 ? 'text-danger' : ''" />
  </div>
</template>
