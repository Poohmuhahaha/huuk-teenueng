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
    cell: (info) => `${props.currency}${Number(info.getValue()).toLocaleString('en-US')}`,
  },
  { accessorKey: 'kind', header: 'In/Out' },
  { accessorKey: 'sub', header: 'Sub-category' },
]

const data = computed(() => props.rows)
const table = useTable({ features, columns, data })
</script>

<template>
  <div class="tblwrap">
    <table class="tbl">
      <thead>
        <tr>
          <th v-for="header in table.getHeaderGroups()[0].headers" :key="header.id" class="sortable"
            @click="header.column.toggleSorting()">
            <FlexRender v-if="!header.isPlaceholder" :header="header" />
            {{ header.column.getIsSorted() === 'asc' ? '↑' : header.column.getIsSorted() === 'desc' ? '↓' : '' }}
          </th>
        </tr>
      </thead>
      <tbody>
        <tr v-for="row in table.getRowModel().rows" :key="row.id" style="cursor: default;">
          <td v-for="cell in row.getAllCells()" :key="cell.id" :class="{ num: cell.column.id === 'amount' }">
            <FlexRender :cell="cell" />
          </td>
        </tr>
      </tbody>
    </table>
  </div>
</template>
