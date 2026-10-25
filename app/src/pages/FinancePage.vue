<script setup lang="ts">
import { computed, ref } from 'vue'
import { useTxns, useAddTxn, usePermission } from '@/core/queries'
import { t } from '@/core/i18n'
import type { TxnKind } from '@/mock/db'
import BudgetCards from '@/components/ui/BudgetCards.vue'
import BarChart from '@/components/ui/BarChart.vue'
import TxnTable from '@/components/tables/TxnTable.vue'

const CURRENCY = '฿' // Finance sheet currency symbol
const { data: txns, isPending, isError, error: loadError, refetch } = useTxns()
const isEmpty = computed(() => !txns.value || txns.value.length === 0)
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

const financeSparks = computed(() => ({
  income: monthlyIn.value.map((m) => m.value),
  expense: monthlyOut.value.map((m) => m.value),
  balance: monthlyIn.value.map((m, i) => m.value - (monthlyOut.value[i]?.value ?? 0)),
}))

const totalMonthlyIn = computed(() => monthlyIn.value.reduce((s, it) => s + it.value, 0))
const totalMonthlyOut = computed(() => monthlyOut.value.reduce((s, it) => s + it.value, 0))

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
  <div class="financepage text-left w-full flex flex-col gap-5">
    <BudgetCards :income="income" :expense="expense" :balance="balance" :currency="CURRENCY" :sparks="financeSparks" />

    <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
      <div class="card-fade p-4 sm:p-5 flex flex-col justify-between">
        <div>
          <div class="flex items-center justify-between gap-2 mb-2">
            <div class="flex items-center gap-2">
              <h2 class="text-[12.5px] font-semibold text-ink-soft uppercase tracking-wider m-0">Income by month</h2>
              <span class="inline-flex items-center px-2 py-0.5 rounded-full text-[11px] font-semibold bg-black text-white">
                {{ monthlyIn.length }} active mos
              </span>
            </div>
            <span class="text-xs font-semibold text-muted font-mono">2026</span>
          </div>
          <p class="tabular text-[22px] sm:text-[24px] font-bold leading-tight tracking-[-0.03em] text-ink font-mono mb-2">
            ฿{{ totalMonthlyIn.toLocaleString('en-US') }}
          </p>
        </div>
        <BarChart :items="monthlyIn" variant="towers" currency="฿" :card="false" />
      </div>

      <div class="card-fade p-4 sm:p-5 flex flex-col justify-between">
        <div>
          <div class="flex items-center justify-between gap-2 mb-2">
            <div class="flex items-center gap-2">
              <h2 class="text-[12.5px] font-semibold text-ink-soft uppercase tracking-wider m-0">Expense by month</h2>
              <span class="inline-flex items-center px-2 py-0.5 rounded-full text-[11px] font-semibold bg-black text-white">
                {{ monthlyOut.length }} active mos
              </span>
            </div>
            <span class="text-xs font-semibold text-muted font-mono">2026</span>
          </div>
          <p class="tabular text-[22px] sm:text-[24px] font-bold leading-tight tracking-[-0.03em] text-ink font-mono mb-2">
            ฿{{ totalMonthlyOut.toLocaleString('en-US') }}
          </p>
        </div>
        <BarChart :items="monthlyOut" variant="towers" currency="฿" :card="false" />
      </div>
    </div>

    <div>
      <div class="flex items-center justify-between gap-2 mb-2.5">
        <div class="flex items-center gap-2">
          <h2 class="text-xs font-bold uppercase tracking-widest text-muted m-0">Transactions</h2>
          <span class="inline-flex items-center px-2 py-0.5 rounded-full text-[11px] font-semibold bg-black text-white font-mono">
            {{ txns?.length ?? 0 }} entries
          </span>
        </div>
      </div>
      <div v-if="isPending" class="muted">Loading ledger…</div>
      <div v-else-if="isError" class="card" role="alert">
        <p class="muted">Could not load ledger: {{ loadError?.message }}</p>
        <button type="button" class="btn w-auto" @click="() => refetch()">{{ t('common.tryAgain') }}</button>
      </div>
      <div v-else-if="isEmpty" class="empty-state">
        <p class="muted">No transactions recorded yet.</p>
        <TxnTable :rows="[]" :currency="CURRENCY" />
      </div>
      <TxnTable v-else :rows="txns ?? []" :currency="CURRENCY" />
    </div>

    <div class="card-fade p-4 sm:p-5 flex flex-col gap-3.5">
      <div class="flex items-center justify-between gap-2 flex-wrap">
        <h2 class="text-[12.5px] font-semibold text-ink-soft uppercase tracking-wider m-0">Log transaction</h2>
        <!-- Kind Toggle Switch (Income vs Expense) -->
        <div class="inline-flex items-center p-0.5 rounded-full bg-wash border border-line select-none">
          <button
            type="button"
            class="px-3 py-1 rounded-full text-xs font-semibold transition-all cursor-pointer"
            :class="dKind === 'IN' ? 'bg-emerald-600 text-white shadow-xs' : 'text-muted hover:text-ink'"
            :disabled="!canWrite"
            @click="dKind = 'IN'"
          >
            + Income (IN)
          </button>
          <button
            type="button"
            class="px-3 py-1 rounded-full text-xs font-semibold transition-all cursor-pointer"
            :class="dKind === 'OUT' ? 'bg-black text-white shadow-xs' : 'text-muted hover:text-ink'"
            :disabled="!canWrite"
            @click="dKind = 'OUT'"
          >
            - Expense (OUT)
          </button>
        </div>
      </div>

      <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-3 items-end">
        <div>
          <label class="lbl block text-xs font-semibold text-muted mb-1.5">Date</label>
          <input class="field w-full m-0" type="date" v-model="dDate" :disabled="!canWrite"
            :title="canWrite ? '' : t('auth.noPerm')" />
        </div>
        <div>
          <label class="lbl block text-xs font-semibold text-muted mb-1.5">Amount ({{ CURRENCY }})</label>
          <input class="field w-full font-mono m-0" type="number" v-model.number="dAmount" :disabled="!canWrite"
            :title="canWrite ? '' : t('auth.noPerm')" placeholder="0" />
        </div>
        <div>
          <label class="lbl block text-xs font-semibold text-muted mb-1.5">Sub-category</label>
          <input class="field w-full m-0" v-model="dSub" :disabled="!canWrite"
            :title="canWrite ? '' : t('auth.noPerm')" placeholder="e.g. Software, Ad spend, Brand" />
        </div>
        <div>
          <button
            class="btn btn-primary w-full h-10 font-semibold rounded-xl flex items-center justify-center gap-1.5 m-0"
            :disabled="!canWrite || addTxn.isPending.value"
            :title="canWrite ? '' : t('auth.noPerm')"
            @click="add"
          >
            <span v-if="!addTxn.isPending.value">+</span>
            {{ addTxn.isPending.value ? t('common.loading') : 'Add transaction' }}
          </button>
        </div>
      </div>
      <p v-if="addError" class="autherr m-0 text-xs" role="alert">{{ addError }}</p>
    </div>
  </div>
</template>
