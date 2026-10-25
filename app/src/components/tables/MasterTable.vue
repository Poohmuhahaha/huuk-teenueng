<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import {
  FlexRender, tableFeatures, useTable,
  rowSortingFeature, createSortedRowModel, sortFn_alphanumeric,
  rowPaginationFeature, createPaginatedRowModel,
} from '@tanstack/vue-table'
import type { ColumnDef } from '@tanstack/vue-table'
import type { Post } from '@/mock/db'
import StatusChip from '@/components/ui/StatusChip.vue'
import TableFrame from '@/components/tables/TableFrame.vue'

const props = withDefaults(
  defineProps<{
    rows: Post[]
    loading?: boolean
    selectedId?: string | null
    pageSize?: number
  }>(),
  {
    loading: false,
    selectedId: null,
    pageSize: 10,
  }
)
const emit = defineEmits<{ select: [post: Post] }>()

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
  sortFns: { alphanumeric: sortFn_alphanumeric },
  rowPaginationFeature,
  paginatedRowModel: createPaginatedRowModel(),
})

const columns: ColumnDef<typeof features, Post, unknown>[] = [
  { id: 'select', header: '', enableSorting: false },
  { accessorKey: 'topic', header: 'Topic' },
  { accessorKey: 'pillar', header: 'Pillar' },
  { accessorKey: 'date', header: 'Date', cell: (info) => (info.getValue() as string | null) ?? '—' },
  { accessorKey: 'status', header: 'Status' },
  {
    accessorKey: 'platforms', header: 'Platforms', enableSorting: false,
    cell: (info) => ((info.getValue() as string[] | null) ?? []).join(' · ') || '—',
  },
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

const checked = ref<Set<string>>(new Set())
const visibleIds = computed(() => table.getRowModel().rows.map((r) => r.original.id))
const allChecked = computed(() =>
  visibleIds.value.length > 0 && visibleIds.value.every((id) => checked.value.has(id)),
)
const someChecked = computed(() => visibleIds.value.some((id) => checked.value.has(id)))

// Drop selections for posts that no longer exist in rows
const existingIds = computed(() => new Set(props.rows.map((r) => r.id)))
watch(existingIds, (ids) => {
  const next = new Set([...checked.value].filter((id) => ids.has(id)))
  if (next.size !== checked.value.size) checked.value = next
})

function toggleAll(): void {
  const next = new Set(checked.value)
  if (allChecked.value) visibleIds.value.forEach((id) => next.delete(id))
  else visibleIds.value.forEach((id) => next.add(id))
  checked.value = next
}

function toggleOne(id: string): void {
  const next = new Set(checked.value)
  if (next.has(id)) next.delete(id)
  else next.add(id)
  checked.value = next
}

// Clicking a cell must still select the row (row editor), except on the
// checkbox column where selection is handled by the checkbox itself.
function onCellClick(e: MouseEvent, columnId: string): void {
  if (columnId === 'select') e.stopPropagation()
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

function onRowKeydown(e: KeyboardEvent, post: Post): void {
  if ((e.target as HTMLElement).closest('input[type=checkbox]')) return
  emit('select', post)
}
</script>

<template>
  <div class="w-full flex-1 flex flex-col min-h-0">
    <div v-if="$slots.actions" class="tbl-tools flex items-center justify-center gap-2.5 mb-2">
      <slot name="actions" />
    </div>
    <div v-if="loading" class="text-muted text-sm p-4" role="status">Loading rows…</div>
    <TableFrame v-else class="flex-1 min-h-0 overflow-auto">
      <thead>
        <tr>
          <th v-for="header in table.getHeaderGroups()[0]?.headers ?? []" :key="header.id"
            :class="header.column.getCanSort() ? 'sortable' : ''"
            :aria-sort="ariaSort(header.column.getCanSort(), header.column.getIsSorted())"
            :tabindex="header.column.getCanSort() ? 0 : undefined"
            @click="toggleSort(header.column)"
            @keydown.enter.prevent="toggleSort(header.column)"
            @keydown.space.prevent="toggleSort(header.column)">
            <input v-if="header.id === 'select'" type="checkbox" :checked="allChecked"
              :indeterminate.prop="someChecked && !allChecked" aria-label="Select all rows"
              :aria-checked="allChecked ? 'true' : someChecked ? 'mixed' : 'false'"
              @click.stop @change="toggleAll" />
            <template v-else>
              <FlexRender v-if="!header.isPlaceholder" :header="header" />
              {{ header.column.getIsSorted() === 'asc' ? '↑' : header.column.getIsSorted() === 'desc' ? '↓' : '' }}
            </template>
          </th>
        </tr>
      </thead>
      <tbody>
        <tr v-if="!rows.length">
          <td colspan="6" class="text-center text-muted">No rows yet.</td>
        </tr>
        <tr v-for="row in table.getRowModel().rows" :key="row.id"
          :class="[
            checked.has(row.original.id) ? 'selected' : '',
            selectedId === row.original.id ? '!bg-wash font-semibold' : ''
          ]" tabindex="0"
          @click="emit('select', row.original)"
          @keydown.enter.prevent="onRowKeydown($event, row.original)"
          @keydown.space.prevent="onRowKeydown($event, row.original)">
          <td v-for="cell in row.getAllCells()" :key="cell.id" @click="onCellClick($event, cell.column.id)">
            <input v-if="cell.column.id === 'select'" type="checkbox"
              :checked="checked.has(row.original.id)" :aria-label="`Select ${row.original.topic}`"
              @change="toggleOne(row.original.id)" />
            <StatusChip v-else-if="cell.column.id === 'status'" :status="row.original.status" />
            <FlexRender v-else :cell="cell" />
          </td>
        </tr>
      </tbody>
    </TableFrame>
    <div class="text-muted text-xs sm:text-sm px-4 py-2.5 m-0 border-t border-faint bg-surface flex-none flex flex-wrap items-center justify-between gap-3">
      <div class="flex items-center gap-2">
        <span>{{ checked.size }} selected</span>
        <span class="text-muted/60">·</span>
        <span>Showing <strong class="font-semibold text-ink">{{ rangeStart }}–{{ rangeEnd }}</strong> of <strong class="font-semibold text-ink">{{ totalRows }}</strong> topics</span>
        <div v-if="totalRows > 10" class="hidden sm:flex items-center gap-1 ml-2 text-xs">
          <span class="text-muted/70">Show:</span>
          <button
            v-for="size in [10, 25, 50]"
            :key="size"
            type="button"
            class="px-2 py-0.5 rounded text-xs font-mono transition-colors cursor-pointer"
            :class="pageSize === size ? 'bg-ink text-white font-bold' : 'hover:bg-wash text-muted'"
            @click="setPageSize(size)"
          >
            {{ size }}
          </button>
        </div>
      </div>

      <div class="flex items-center gap-2 ml-auto">
        <span class="text-xs text-muted/70 hidden md:inline">click a row to edit</span>
        <div v-if="totalPages > 1" class="flex items-center gap-1.5 ml-2">
          <button
            type="button"
            class="px-2.5 py-1 rounded-lg border border-line bg-surface hover:bg-wash disabled:opacity-40 disabled:cursor-not-allowed font-medium text-xs text-ink transition-colors cursor-pointer"
            :disabled="!table.getCanPreviousPage()"
            aria-label="Previous page"
            @click="table.previousPage()"
          >
            ‹ Prev
          </button>
          <span class="font-mono text-xs px-1 text-ink">
            {{ pageIndex + 1 }} / {{ totalPages }}
          </span>
          <button
            type="button"
            class="px-2.5 py-1 rounded-lg border border-line bg-surface hover:bg-wash disabled:opacity-40 disabled:cursor-not-allowed font-medium text-xs text-ink transition-colors cursor-pointer"
            :disabled="!table.getCanNextPage()"
            aria-label="Next page"
            @click="table.nextPage()"
          >
            Next ›
          </button>
        </div>
      </div>
    </div>
  </div>
</template>
