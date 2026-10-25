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
    <!-- Bottom pagination & selection bar -->
    <div
      class="tbl-footer flex-none flex items-center justify-between gap-3 px-5 py-3 bg-surface border-t border-faint text-sm text-muted min-h-[50px] select-none"
    >
      <!-- Left: Selection status & item count -->
      <div class="flex items-center gap-2 min-w-0 truncate">
        <span
          class="font-mono font-medium transition-colors"
          :class="checked.size > 0 ? 'text-accent font-semibold' : 'text-muted'"
        >
          {{ checked.size }} selected
        </span>
        <span class="text-muted/40">·</span>
        <span class="truncate">
          Showing <strong class="font-semibold text-ink font-mono">{{ rangeStart }}–{{ rangeEnd }}</strong> of <strong class="font-semibold text-ink font-mono">{{ totalRows }}</strong> topics
        </span>
      </div>

      <!-- Right: Per-page selector & Page stepper -->
      <div class="flex items-center gap-3.5 flex-none ml-auto">
        <!-- Page size selector -->
        <div v-if="totalRows > 10" class="hidden sm:flex items-center gap-1.5 text-xs">
          <span class="text-muted/70 font-medium">Show:</span>
          <div class="inline-flex rounded-lg p-0.5 bg-wash border border-line/60">
            <button
              v-for="size in [10, 25, 50]"
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

        <!-- Integrated Stepper Pill [ ‹ | 1 / 3 | › ] -->
        <div v-if="totalPages > 1" class="inline-flex items-center rounded-lg border border-line bg-surface shadow-xs overflow-hidden h-8 sm:h-8.5">
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
  </div>
</template>
