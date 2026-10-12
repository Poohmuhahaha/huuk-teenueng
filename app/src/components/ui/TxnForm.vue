<script setup lang="ts">
// Transaction form — used inline (wide card) and inside the log sheet. Owns
// its own mutation so both hosts stay thin.
import { computed, ref, useId } from 'vue'
import { useAddTxn, usePermission } from '@/core/queries'
import { t } from '@/core/i18n'
import type { TxnKind } from '@/mock/db'

const emit = defineEmits<{ done: [] }>()
// useId prefix keeps label/input ids unique when both hosts (wide card +
// log sheet) mount at once.
const uid = useId()
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

async function add(): Promise<void> {
  if (!canWrite.value || addTxn.isPending.value) return
  addError.value = ''
  if (!dDate.value || !Number.isFinite(dAmount.value) || dAmount.value <= 0) {
    addError.value = 'Enter a date and a positive amount first.'
    return
  }
  try {
    await addTxn.mutateAsync({
      date: dDate.value,
      amount: dAmount.value,
      kind: dKind.value,
      category: dCategory.value.trim() || (dKind.value === 'IN' ? 'Income' : 'Expense'),
      sub: dSub.value.trim(),
    })
    dAmount.value = 0
    dSub.value = ''
    dCategory.value = ''
    emit('done')
  } catch (e) {
    addError.value = e instanceof Error ? e.message : String(e)
  }
}
</script>

<template>
  <form class="grid gap-3 text-left" @submit.prevent="add">
    <div class="grid grid-cols-2 gap-3">
      <div><label class="block text-muted text-sm font-[650] m-0" :for="`${uid}-date`">Date</label><input :id="`${uid}-date`" type="date" v-model="dDate"
        :disabled="!canWrite" :title="canWrite ? '' : t('auth.noPerm')" /></div>
      <div><label class="block text-muted text-sm font-[650] m-0" :for="`${uid}-amount`">Amount</label><input :id="`${uid}-amount`" type="number" v-model.number="dAmount" min="0.01" step="0.01" inputmode="decimal"
        :disabled="!canWrite" :title="canWrite ? '' : t('auth.noPerm')" /></div>
      <div><label class="block text-muted text-sm font-[650] m-0" :for="`${uid}-kind`">Kind</label>
        <select :id="`${uid}-kind`" v-model="dKind" :disabled="!canWrite" :title="canWrite ? '' : t('auth.noPerm')">
          <option value="IN">IN</option><option value="OUT">OUT</option></select></div>
      <div><label class="block text-muted text-sm font-[650] m-0" :for="`${uid}-cat`">Category</label><input :id="`${uid}-cat`" v-model="dCategory" :disabled="!canWrite"
        :title="canWrite ? '' : t('auth.noPerm')" /></div>
      <div><label class="block text-muted text-sm font-[650] m-0" :for="`${uid}-sub`">Sub-category</label><input :id="`${uid}-sub`" v-model="dSub" :disabled="!canWrite"
        :title="canWrite ? '' : t('auth.noPerm')" /></div>
    </div>
    <p v-if="addError" class="text-danger text-[13.5px] font-semibold mt-2.5" role="alert">{{ addError }}</p>
    <div class="flex gap-2">
      <button type="submit" class="w-auto flex-none font-sans uppercase tracking-[0.08em] rounded-full px-5.5 py-3 text-[12.5px] font-semibold border-[1.5px] border-ink bg-ink text-white cursor-pointer shadow-none hover:bg-accent-hover hover:border-accent-hover hover:text-white active:transform-none disabled:opacity-50 disabled:cursor-default" :disabled="!canWrite || addTxn.isPending.value"
        :title="canWrite ? '' : t('auth.noPerm')">
        {{ addTxn.isPending.value ? t('common.loading') : 'Add transaction' }}</button>
    </div>
  </form>
</template>
