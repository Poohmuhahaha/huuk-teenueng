<script setup lang="ts">
import { computed } from 'vue'
import {
  FlexRender, tableFeatures, useTable,
  rowSortingFeature, createSortedRowModel, sortFn_alphanumeric, sortFn_basic,
} from '@tanstack/vue-table'
import type { ColumnDef } from '@tanstack/vue-table'
import type { Txn } from '@/mock/db'
import TableFrame from '@/components/tables/TableFrame.vue'

const props = defineProps<{ rows: Txn[]; currency: string }>()

const features = tableFeatures({
  rowSortingFeature,
  sortedRowModel: createSortedRowModel(),
  sortFns: { alphanumeric: sortFn_alphanumeric, basic: sortFn_basic },
})

const columns: ColumnDef<typeof features, Txn, unknown>[] = [
  { accessorKey: 'date', header: 'Date' },
  {
    accessorKey: 'amount', header: 'Amount', sortFn: 'basic',
    cell: (info) => {
      const v = Number(info.getValue())
      const isOut = info.row.original.kind === 'OUT'
      const sign = isOut ? '-' : '+'
      return Number.isFinite(v) ? `${sign}${props.currency}${v.toLocaleString('en-US')}` : '—'
    },
  },
  { accessorKey: 'kind', header: 'In/Out' },
  { accessorKey: 'category', header: 'Category' },
  { accessorKey: 'sub', header: 'Sub-category' },
]

const data = computed(() => props.rows)
const table = useTable({ features, columns, data })

function ariaSort(canSort: boolean, sorted: false | 'asc' | 'desc'): 'ascending' | 'descending' | 'none' {
  if (!canSort) return 'none'
  if (sorted === 'asc') return 'ascending'
  if (sorted === 'desc') return 'descending'
  return 'none'
}

function toggleSort(column: { getCanSort: () => boolean; toggleSorting: () => void }): void {
  if (column.getCanSort()) column.toggleSorting()
}
</script>

<template>
  <div class="txn-table-wrapper w-full">
    <!-- Desktop Table View (>= 768px) -->
    <div class="hidden md:block">
      <TableFrame>
        <thead>
          <tr>
            <th v-for="header in table.getHeaderGroups()[0]?.headers ?? []" :key="header.id"
              :class="[header.column.id === 'amount' ? 'text-right' : '', header.column.getCanSort() ? 'sortable' : '']"
              :aria-sort="ariaSort(header.column.getCanSort(), header.column.getIsSorted())"
              :tabindex="header.column.getCanSort() ? 0 : undefined"
              @click="toggleSort(header.column)"
              @keydown.enter.prevent="toggleSort(header.column)"
              @keydown.space.prevent="toggleSort(header.column)">
              <FlexRender v-if="!header.isPlaceholder" :header="header" />
              {{ header.column.getIsSorted() === 'asc' ? '↑' : header.column.getIsSorted() === 'desc' ? '↓' : '' }}
            </th>
          </tr>
        </thead>
        <tbody>
          <tr v-if="!rows.length">
            <td colspan="5" class="text-center text-muted py-8 text-sm">No transactions yet.</td>
          </tr>
          <tr v-for="row in table.getRowModel().rows" :key="row.id" class="group transition-colors hover:bg-wash/50">
            <td v-for="cell in row.getAllCells()" :key="cell.id" :class="cell.column.id === 'amount' ? 'num' : ''">
              <!-- Custom Pill for Kind -->
              <template v-if="cell.column.id === 'kind'">
                <span
                  class="inline-flex items-center gap-1 px-2 py-0.5 rounded-full text-[11px] font-semibold font-mono tracking-tight"
                  :class="row.original.kind === 'IN' ? 'bg-emerald-500/10 text-emerald-700 dark:text-emerald-400 border border-emerald-500/20' : 'bg-rose-500/10 text-rose-700 dark:text-rose-400 border border-rose-500/20'"
                >
                  <span class="w-1.5 h-1.5 rounded-full" :class="row.original.kind === 'IN' ? 'bg-emerald-500' : 'bg-rose-500'" />
                  {{ row.original.kind }}
                </span>
              </template>

              <!-- Custom Amount with Color & Mono -->
              <template v-else-if="cell.column.id === 'amount'">
                <span
                  class="font-mono text-sm font-bold tabular-nums"
                  :class="row.original.kind === 'IN' ? 'text-emerald-700 dark:text-emerald-400' : 'text-rose-600 dark:text-rose-400'"
                >
                  <FlexRender :cell="cell" />
                </span>
              </template>

              <!-- Date formatted in monospace -->
              <template v-else-if="cell.column.id === 'date'">
                <span class="font-mono text-xs text-muted">
                  <FlexRender :cell="cell" />
                </span>
              </template>

              <!-- Category in bold -->
              <template v-else-if="cell.column.id === 'category'">
                <span class="font-semibold text-ink">
                  <FlexRender :cell="cell" />
                </span>
              </template>

              <template v-else>
                <FlexRender :cell="cell" />
              </template>
            </td>
          </tr>
        </tbody>
      </TableFrame>
    </div>

    <!-- Mobile Card Feed View (< 768px) - Zero Horizontal Scroll, Thumb Friendly -->
    <div class="block md:hidden border border-line rounded-2xl bg-surface divide-y divide-line/60 overflow-hidden shadow-xs">
      <div v-if="!rows.length" class="p-8 text-center text-sm text-muted">
        No transactions yet.
      </div>
      <div
        v-for="row in table.getRowModel().rows"
        :key="row.id"
        class="p-3.5 sm:p-4 flex items-center justify-between gap-3 active:bg-wash/60 transition-colors cursor-pointer"
      >
        <!-- Left Column: Icon + Category + Sub/Date -->
        <div class="flex items-center gap-2.5 min-w-0">
          <!-- Inward arrow ↙ for IN (No background circle) -->
          <svg
            v-if="row.original.kind === 'IN'"
            class="w-4 h-4 flex-none text-emerald-600 dark:text-emerald-400"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2.5"
            stroke-linecap="round"
            stroke-linejoin="round"
            aria-hidden="true"
          >
            <line x1="17" y1="7" x2="7" y2="17" />
            <polyline points="17 17 7 17 7 7" />
          </svg>
          <!-- Outward arrow ↗ for OUT (No background circle) -->
          <svg
            v-else
            class="w-4 h-4 flex-none text-rose-600 dark:text-rose-400"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2.5"
            stroke-linecap="round"
            stroke-linejoin="round"
            aria-hidden="true"
          >
            <line x1="7" y1="17" x2="17" y2="7" />
            <polyline points="7 7 17 7 17 17" />
          </svg>

          <div class="flex flex-col min-w-0">
            <span class="text-[13.5px] font-bold text-ink truncate leading-snug">
              {{ row.original.category }}
            </span>
            <span class="text-xs text-muted truncate mt-0.5">
              <span v-if="row.original.sub">{{ row.original.sub }} · </span>
              <span class="font-mono text-[11px]">{{ row.original.date }}</span>
            </span>
          </div>
        </div>

        <!-- Right Column: Amount + Status Pill -->
        <div class="flex flex-col items-end flex-none text-right">
          <span
            class="text-[14.5px] font-bold font-mono tracking-tight leading-snug"
            :class="row.original.kind === 'IN' ? 'text-emerald-700 dark:text-emerald-400' : 'text-rose-600 dark:text-rose-400'"
          >
            {{ row.original.kind === 'IN' ? '+' : '-' }}{{ currency }}{{ row.original.amount.toLocaleString('en-US') }}
          </span>
          <span
            class="inline-flex items-center px-1.5 py-0.2 rounded-full text-[10px] font-bold font-mono tracking-wide mt-0.5 uppercase"
            :class="row.original.kind === 'IN' ? 'bg-emerald-500/10 text-emerald-700 dark:text-emerald-400' : 'bg-rose-500/10 text-rose-700 dark:text-rose-400'"
          >
            {{ row.original.kind }}
          </span>
        </div>
      </div>
    </div>
  </div>
</template>
