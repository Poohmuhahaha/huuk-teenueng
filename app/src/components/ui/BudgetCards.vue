<script setup lang="ts">
import KpiCard from './KpiCard.vue'

defineProps<{ income: number; expense: number; balance: number; currency: string }>()

function money(n: number, cur: string): string {
  return `${cur}${n.toLocaleString('en-US')}`
}
</script>

<template>
  <div class="grid3">
    <KpiCard label="Income (real)" :value="money(income, currency)" />
    <KpiCard label="Expense (real)" :value="money(expense, currency)" />
    <KpiCard label="Balance" :value="money(balance, currency)" />
  </div>
</template>
