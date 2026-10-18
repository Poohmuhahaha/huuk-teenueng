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
        <td colspan="5" class="text-center text-muted">No transactions yet.</td>
      </tr>
      <tr v-for="row in table.getRowModel().rows" :key="row.id">
        <td v-for="cell in row.getAllCells()" :key="cell.id" :class="cell.column.id === 'amount' ? 'num' : ''">
          <FlexRender :cell="cell" />
        </td>
      </tr>
    </tbody>
  </TableFrame>
</template>
