<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import {
  FlexRender, tableFeatures, useTable,
  rowSortingFeature, createSortedRowModel, sortFn_alphanumeric,
  columnFilteringFeature, createFilteredRowModel, filterFn_includesString,
} from '@tanstack/vue-table'
import type { ColumnDef } from '@tanstack/vue-table'
import type { Post } from '@/mock/db'
import StatusChip from '@/components/ui/StatusChip.vue'

const props = defineProps<{ rows: Post[]; loading?: boolean }>()
const emit = defineEmits<{ select: [post: Post] }>()

const features = tableFeatures({
  rowSortingFeature,
  sortedRowModel: createSortedRowModel(),
  sortFns: { alphanumeric: sortFn_alphanumeric },
  columnFilteringFeature,
  filteredRowModel: createFilteredRowModel(),
  filterFns: { includesString: filterFn_includesString },
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
const table = useTable({ features, columns, data })

const search = ref('')
function onSearch(e: Event): void {
  search.value = (e.target as HTMLInputElement).value
  table.getColumn('topic')?.setFilterValue(search.value)
}

const checked = ref<Set<string>>(new Set())
const visibleIds = computed(() => table.getRowModel().rows.map((r) => r.original.id))
const allChecked = computed(() =>
  visibleIds.value.length > 0 && visibleIds.value.every((id) => checked.value.has(id)),
)
const someChecked = computed(() => visibleIds.value.some((id) => checked.value.has(id)))

// Drop selections that scrolled out of the filtered/sorted view.
watch(visibleIds, (ids) => {
  const visible = new Set(ids)
  const next = new Set([...checked.value].filter((id) => visible.has(id)))
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
</script>

<template>
  <div>
    <input class="field" style="margin-bottom: 8px;" placeholder="Search topic" aria-label="Search topic"
      :value="search" @input="onSearch" />
    <div v-if="loading" class="muted">Loading rows…</div>
    <div class="tblwrap">
      <table class="tbl">
        <thead>
          <tr>
            <th v-for="header in table.getHeaderGroups()[0].headers" :key="header.id"
              :class="{ sortable: header.column.getCanSort() }"
              :aria-sort="ariaSort(header.column.getCanSort(), header.column.getIsSorted())"
              :tabindex="header.column.getCanSort() ? 0 : undefined"
              @click="toggleSort(header.column)"
              @keydown.enter.prevent="toggleSort(header.column)"
              @keydown.space.prevent="toggleSort(header.column)">
              <input v-if="header.id === 'select'" type="checkbox" :checked="allChecked"
                :indeterminate.prop="someChecked && !allChecked" aria-label="Select all rows"
                @click.stop @change="toggleAll" />
              <template v-else>
                <FlexRender v-if="!header.isPlaceholder" :header="header" />
                {{ header.column.getIsSorted() === 'asc' ? '↑' : header.column.getIsSorted() === 'desc' ? '↓' : '' }}
              </template>
            </th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="row in table.getRowModel().rows" :key="row.id"
            :class="{ selected: checked.has(row.original.id) }" tabindex="0"
            @click="emit('select', row.original)"
            @keydown.enter.prevent="emit('select', row.original)"
            @keydown.space.prevent="emit('select', row.original)">
            <td v-for="cell in row.getAllCells()" :key="cell.id" @click="onCellClick($event, cell.column.id)">
              <input v-if="cell.column.id === 'select'" type="checkbox"
                :checked="checked.has(row.original.id)" :aria-label="`Select ${row.original.topic}`"
                @change="toggleOne(row.original.id)" />
              <StatusChip v-else-if="cell.column.id === 'status'" :status="row.original.status" />
              <FlexRender v-else :cell="cell" />
            </td>
          </tr>
        </tbody>
      </table>
    </div>
    <p class="muted">{{ checked.size }} selected · click a row to edit · headers sort</p>
  </div>
</template>
