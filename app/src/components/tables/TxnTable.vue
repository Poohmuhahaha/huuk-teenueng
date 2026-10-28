<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import {
  FlexRender, tableFeatures, useTable,
  rowSortingFeature, createSortedRowModel, sortFn_alphanumeric, sortFn_basic,
  rowPaginationFeature, createPaginatedRowModel,
} from '@tanstack/vue-table'
import type { ColumnDef } from '@tanstack/vue-table'
import type { Txn } from '@/mock/db'
import TableFrame from '@/components/tables/TableFrame.vue'

const props = withDefaults(
  defineProps<{
    rows: Txn[]
    currency: string
    pageSize?: number
  }>(),
  {
    pageSize: 8,
  }
)

const pageIndex = ref(0)
const pageSize = ref(props.pageSize)

watch(() => props.pageSize, (newSize) => {
  if (newSize) {
    pageSize.value = newSize
    pageIndex.value = 0
  }
})

watch(() => props.rows.length, () => {
  const maxPage = Math.max(0, Math.ceil(props.rows.length / pageSize.value) - 1)
  if (pageIndex.value > maxPage) pageIndex.value = 0
})

const features = tableFeatures({
  rowSortingFeature,
  sortedRowModel: createSortedRowModel(),
  sortFns: { alphanumeric: sortFn_alphanumeric, basic: sortFn_basic },
  rowPaginationFeature,
  paginatedRowModel: createPaginatedRowModel(),
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
const table = useTable({
  features,
  columns,
  data,
  state: {
    get pagination() {
      return { pageIndex: pageIndex.value, pageSize: pageSize.value }
    },
  },
  onPaginationChange: (updater) => {
    const next = typeof updater === 'function'
      ? updater({ pageIndex: pageIndex.value, pageSize: pageSize.value })
      : updater
    pageIndex.value = next.pageIndex
    pageSize.value = next.pageSize
  },
})

const totalRows = computed(() => props.rows.length)
const totalPages = computed(() => Math.max(1, Math.ceil(totalRows.value / pageSize.value)))
const rangeStart = computed(() => totalRows.value === 0 ? 0 : pageIndex.value * pageSize.value + 1)
const rangeEnd = computed(() => Math.min(totalRows.value, (pageIndex.value + 1) * pageSize.value))

function setPageSize(size: number) {
  pageSize.value = size
  pageIndex.value = 0
}

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
                  :class="row.original.kind === 'IN' ? 'bg-emerald-600 text-white border border-emerald-600' : 'bg-rose-600 text-white border border-rose-600'"
                >
                  <span class="w-1.5 h-1.5 rounded-full bg-white" />
                  {{ row.original.kind }}
                </span>
              </template>

              <!-- Custom Amount with Color & Mono -->
              <template v-else-if="cell.column.id === 'amount'">
                <span
                  class="font-mono text-sm font-bold tabular-nums"
                  :class="row.original.kind === 'IN' ? 'text-emerald-600 dark:text-emerald-400' : 'text-rose-600 dark:text-rose-400'"
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
            :class="row.original.kind === 'IN' ? 'text-emerald-600 dark:text-emerald-400' : 'text-rose-600 dark:text-rose-400'"
          >
            {{ row.original.kind === 'IN' ? '+' : '-' }}{{ currency }}{{ row.original.amount.toLocaleString('en-US') }}
          </span>
          <span
            class="inline-flex items-center px-1.5 py-0.2 rounded-full text-[10px] font-bold font-mono tracking-wide mt-0.5 uppercase"
            :class="row.original.kind === 'IN' ? 'bg-emerald-600 text-white' : 'bg-rose-600 text-white'"
          >
            {{ row.original.kind }}
          </span>
        </div>
      </div>
    </div>

    <!-- Pagination bar (Desktop & Mobile) -->
    <div
      v-if="totalRows > 0"
      class="flex items-center justify-between gap-3 px-5 py-3 mt-2.5 rounded-xl bg-surface border border-line/60 shadow-xs text-sm text-muted min-h-[50px] select-none"
    >
      <div class="flex items-center gap-2 min-w-0 truncate">
        <span class="truncate">
          Showing <strong class="font-semibold text-ink font-mono">{{ rangeStart }}–{{ rangeEnd }}</strong> of <strong class="font-semibold text-ink font-mono">{{ totalRows }}</strong> entries
        </span>
        <div v-if="totalRows > 8" class="hidden sm:flex items-center gap-1.5 ml-3 text-xs">
          <span class="text-muted/70 font-medium">Per page:</span>
          <div class="inline-flex rounded-lg p-0.5 bg-wash border border-line/60">
            <button
              v-for="size in [8, 15, 30]"
              :key="size"
              type="button"
              class="px-2.5 py-1 rounded-md text-xs font-mono transition-all cursor-pointer"
              :class="pageSize === size ? 'bg-surface text-ink font-bold shadow-2xs' : 'text-muted hover:text-ink'"
              @click="setPageSize(size)"
            >
              {{ size }}
            </button>
          </div>
        </div>
      </div>

      <!-- Integrated Stepper Pill [ ‹ | 1 / 3 | › ] -->
      <div v-if="totalPages > 1" class="inline-flex items-center rounded-lg border border-line bg-surface shadow-xs overflow-hidden flex-none ml-auto h-8 sm:h-8.5">
        <button
          type="button"
          class="px-3 h-full inline-flex items-center justify-center text-sm font-semibold text-ink hover:bg-wash transition-colors disabled:opacity-30 disabled:cursor-not-allowed cursor-pointer border-r border-line"
          :disabled="!table.getCanPreviousPage()"
          aria-label="Previous page"
          title="Previous page"
          @click="table.previousPage()"
        >
          ‹
        </button>
        <span class="px-3 h-full inline-flex items-center justify-center font-mono text-xs sm:text-sm font-medium text-ink whitespace-nowrap select-none">
          {{ pageIndex + 1 }} / {{ totalPages }}
        </span>
        <button
          type="button"
          class="px-3 h-full inline-flex items-center justify-center text-sm font-semibold text-ink hover:bg-wash transition-colors disabled:opacity-30 disabled:cursor-not-allowed cursor-pointer border-l border-line"
          :disabled="!table.getCanNextPage()"
          aria-label="Next page"
          title="Next page"
          @click="table.nextPage()"
        >
          ›
        </button>
      </div>
    </div>
  </div>
</template>
