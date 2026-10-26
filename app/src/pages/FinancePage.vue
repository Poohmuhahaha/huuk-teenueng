<script setup lang="ts">
import { computed, ref, onMounted, onBeforeUnmount } from 'vue'
import { useTxns, useAddTxn, usePermission } from '@/core/queries'
import { t } from '@/core/i18n'
import type { TxnKind } from '@/mock/db'
import BudgetCards from '@/components/ui/BudgetCards.vue'
import BarChart from '@/components/ui/BarChart.vue'
import TxnTable from '@/components/tables/TxnTable.vue'
import Drawer from '@/components/overlays/Drawer.vue'

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
const addOpen = ref(false)

const isMobile = ref(typeof window !== 'undefined' ? window.innerWidth < 768 : false)
let mql: MediaQueryList | null = null
const onMql = (e: MediaQueryListEvent | MediaQueryList): void => {
  isMobile.value = e.matches
}
onMounted(() => {
  if (typeof window !== 'undefined' && window.matchMedia) {
    mql = window.matchMedia('(max-width: 767px)')
    isMobile.value = mql.matches
    mql.addEventListener('change', onMql)
  }
})
onBeforeUnmount(() => {
  mql?.removeEventListener('change', onMql)
})

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
    addOpen.value = false
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
        <button
          type="button"
          class="btn btn-primary h-8 sm:h-9 px-3.5 rounded-xl text-xs font-semibold inline-flex items-center gap-1.5 cursor-pointer shadow-xs transition-transform hover:scale-[1.02]"
          :disabled="!canWrite"
          @click="addOpen = true"
        >
          <span class="text-sm font-bold leading-none">+</span>
          <span>Log transaction</span>
        </button>
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

    <!-- Mobile Floating Action Button (FAB) for instant one-tap logging -->
    <div class="fixed bottom-6 right-5 z-40 sm:hidden">
      <button
        type="button"
        class="w-13 h-13 rounded-full bg-ink text-white shadow-xl flex items-center justify-center cursor-pointer active:scale-95 transition-all"
        :disabled="!canWrite"
        title="Log transaction"
        aria-label="Log transaction"
        @click="addOpen = true"
      >
        <svg width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
          <line x1="12" y1="5" x2="12" y2="19" />
          <line x1="5" y1="12" x2="19" y2="12" />
        </svg>
      </button>
    </div>

    <!-- Responsive Log Transaction Sheet: Bottom Sheet on mobile, Right Drawer on desktop -->
    <Drawer
      v-model="addOpen"
      :side="isMobile ? 'bottom' : 'right'"
      :width="'400px'"
      title="Log Transaction"
    >
      <div class="flex flex-col gap-4 text-left p-1">
        <!-- Segmented Toggle Switch: 50/50 Balanced Width, Red for Expense, Green for Income -->
        <div>
          <label class="lbl block text-xs font-semibold text-muted mb-1.5">Type</label>
          <div class="w-full grid grid-cols-2 p-1 rounded-xl bg-wash border border-line select-none gap-1">
            <button
              type="button"
              class="h-9 px-3 rounded-lg text-xs sm:text-sm font-semibold transition-all cursor-pointer flex items-center justify-center gap-1.5 whitespace-nowrap"
              :class="dKind === 'IN' ? 'bg-emerald-600 text-white shadow-xs' : 'text-muted hover:text-ink'"
              :disabled="!canWrite"
              @click="dKind = 'IN'"
            >
              <span class="text-sm font-bold leading-none">+</span>
              <span>Income</span>
            </button>
            <button
              type="button"
              class="h-9 px-3 rounded-lg text-xs sm:text-sm font-semibold transition-all cursor-pointer flex items-center justify-center gap-1.5 whitespace-nowrap"
              :class="dKind === 'OUT' ? 'bg-rose-600 text-white shadow-xs' : 'text-muted hover:text-ink'"
              :disabled="!canWrite"
              @click="dKind = 'OUT'"
            >
              <span class="text-sm font-bold leading-none">-</span>
              <span>Expense</span>
            </button>
          </div>
        </div>

        <div>
          <label class="lbl block text-xs font-semibold text-muted mb-1.5">Date</label>
          <input class="field w-full m-0" type="date" v-model="dDate" :disabled="!canWrite"
            :title="canWrite ? '' : t('auth.noPerm')" />
        </div>

        <div>
          <label class="lbl block text-xs font-semibold text-muted mb-1.5">Amount ({{ CURRENCY }})</label>
          <input class="field w-full font-mono m-0" type="number" v-model.number="dAmount" :disabled="!canWrite"
            :title="canWrite ? '' : t('auth.noPerm')" placeholder="0" min="0" step="100" />
        </div>

        <div>
          <label class="lbl block text-xs font-semibold text-muted mb-1.5">Sub-category</label>
          <input class="field w-full m-0" v-model="dSub" :disabled="!canWrite"
            :title="canWrite ? '' : t('auth.noPerm')" placeholder="e.g. Software, Ad spend, Brand" />
        </div>

        <p v-if="addError" class="autherr m-0 text-xs text-rose-600" role="alert">{{ addError }}</p>
      </div>

      <template #footer>
        <div class="flex items-center justify-end gap-2.5 w-full">
          <button type="button" class="btn text-xs py-2 px-4 cursor-pointer" @click="addOpen = false">
            Cancel
          </button>
          <button
            type="button"
            class="btn btn-primary text-xs py-2 px-5 font-semibold cursor-pointer shadow-xs"
            :disabled="!canWrite || addTxn.isPending.value"
            @click="add"
          >
            {{ addTxn.isPending.value ? t('common.loading') : '+ Add transaction' }}
          </button>
        </div>
      </template>
    </Drawer>
  </div>
</template>
