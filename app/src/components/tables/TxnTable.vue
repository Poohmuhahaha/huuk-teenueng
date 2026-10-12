<script setup lang="ts">
import { computed } from 'vue'
import {
  FlexRender, tableFeatures, useTable,
  rowSortingFeature, createSortedRowModel, sortFn_alphanumeric, sortFn_basic,
} from '@tanstack/vue-table'
import type { ColumnDef } from '@tanstack/vue-table'
import type { Txn } from '@/mock/db'

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
      return Number.isFinite(v) ? `${props.currency}${v.toLocaleString('en-US')}` : '—'
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
  <div class="overflow-x-auto border border-faint bg-surface rounded-none">
    <table class="tbl w-full border-collapse text-md [&_tr:last-child_td]:border-b-0">
      <thead>
        <tr>
          <th v-for="header in table.getHeaderGroups()[0]?.headers ?? []" :key="header.id" class="cursor-pointer border-b border-faint px-3.5 py-2.5 uppercase tracking-[0.05em] text-muted bg-surface-2 text-[11.5px] font-bold whitespace-nowrap hover:text-ink"
            :class="header.column.id === 'amount' ? 'text-right' : 'text-left'"
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
          <td colspan="5" class="text-center text-muted text-sm border-b border-faint px-3.5 py-2.5">No transactions yet.</td>
        </tr>
        <tr v-for="row in table.getRowModel().rows" :key="row.id" class="transition-colors hover:bg-surface-2">
          <td v-for="cell in row.getAllCells()" :key="cell.id" class="text-left border-b border-faint px-3.5 py-2.5" :class="cell.column.id === 'amount' ? 'num text-right tabular-nums' : ''">
            <FlexRender :cell="cell" />
          </td>
        </tr>
      </tbody>
    </table>
  </div>
</template>
