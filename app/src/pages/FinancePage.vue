<script setup lang="ts">
import { computed, ref } from 'vue'
import { useTxns, useAddTxn, usePermission } from '@/core/queries'
import { t } from '@/core/i18n'
import type { TxnKind } from '@/mock/db'
import BudgetCards from '@/components/ui/BudgetCards.vue'
import BarChart from '@/components/ui/BarChart.vue'
import TxnTable from '@/components/tables/TxnTable.vue'

const CURRENCY = '฿' // Finance sheet currency symbol
const { data: txns, isPending } = useTxns()
const addTxn = useAddTxn()
const { can } = usePermission()
const canWrite = computed(() => can('finance.write'))

function localToday(): string {
  const d = new Date()
  const pad = (n: number): string => String(n).padStart(2, '0')
  return `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())}`
}

const dDate = ref(localToday())
const dAmount = ref<number>(0)
const dKind = ref<TxnKind>('OUT')
const dSub = ref('')
const dCategory = ref('')
const addError = ref('')

const sorted = (m: Record<string, number>): { label: string; value: number }[] =>
  Object.entries(m).sort(([a], [b]) => a.localeCompare(b)).map(([label, value]) => ({ label, value }))

const income = computed(() => (txns.value ?? []).filter((t) => t.kind === 'IN').reduce((s, t) => s + t.amount, 0))
const expense = computed(() => (txns.value ?? []).filter((t) => t.kind === 'OUT').reduce((s, t) => s + t.amount, 0))
const balance = computed(() => income.value - expense.value)

const monthlyIn = computed(() => {
  const m: Record<string, number> = {}
  for (const t of txns.value ?? []) {
    if (t.kind !== 'IN') continue
    const k = t.date.slice(0, 7)
    m[k] = (m[k] ?? 0) + t.amount
  }
  return sorted(m)
})
const monthlyOut = computed(() => {
  const m: Record<string, number> = {}
  for (const t of txns.value ?? []) {
    if (t.kind !== 'OUT') continue
    const k = t.date.slice(0, 7)
    m[k] = (m[k] ?? 0) + t.amount
  }
  return sorted(m)
})

async function add(): Promise<void> {
  if (!canWrite.value || addTxn.isPending.value) return
  addError.value = ''
  if (!dDate.value || !Number.isFinite(dAmount.value) || dAmount.value <= 0) {
    addError.value = 'Enter a date and a positive amount first.'
    return
  }
  try {
    await addTxn.mutateAsync({
      date: dDate.value, amount: dAmount.value, kind: dKind.value,
      category: dCategory.value.trim() || (dKind.value === 'IN' ? 'Income' : 'Expense'),
      sub: dSub.value.trim(),
    })
    dAmount.value = 0
    dSub.value = ''
  } catch (e) {
    addError.value = e instanceof Error ? e.message : String(e)
  }
}
</script>

<template>
  <h1>Finance</h1>
  <BudgetCards :income="income" :expense="expense" :balance="balance" :currency="CURRENCY" />
  <div class="grid2 mt">
    <div><h2>Income by month</h2><BarChart :items="monthlyIn" /></div>
    <div><h2>Expense by month</h2><BarChart :items="monthlyOut" /></div>
  </div>
  <h2>Transactions</h2>
  <div v-if="isPending" class="muted">Loading ledger…</div>
  <TxnTable v-else :rows="txns ?? []" :currency="CURRENCY" />
  <h2>Log transaction</h2>
  <div class="grid4">
    <div><label class="lbl">Date</label><input class="field" type="date" v-model="dDate" :disabled="!canWrite"
      :title="canWrite ? '' : t('auth.noPerm')" /></div>
    <div><label class="lbl">Amount</label><input class="field" type="number" v-model.number="dAmount" :disabled="!canWrite"
      :title="canWrite ? '' : t('auth.noPerm')" /></div>
    <div><label class="lbl">Kind</label>
      <select class="field" v-model="dKind" :disabled="!canWrite" :title="canWrite ? '' : t('auth.noPerm')">
        <option value="IN">IN</option><option value="OUT">OUT</option></select></div>
    <div><label class="lbl">Sub-category</label><input class="field" v-model="dSub" :disabled="!canWrite"
      :title="canWrite ? '' : t('auth.noPerm')" /></div>
  </div>
  <p v-if="addError" class="autherr" role="alert">{{ addError }}</p>
  <div class="mt"><button class="btn btn-primary" :disabled="!canWrite || addTxn.isPending.value"
    :title="canWrite ? '' : t('auth.noPerm')" @click="add">
    {{ addTxn.isPending.value ? t('common.loading') : 'Add transaction' }}</button></div>
</template>
