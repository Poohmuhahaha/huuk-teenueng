<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import {
  FlexRender, tableFeatures, useTable,
  rowSortingFeature, createSortedRowModel, sortFn_alphanumeric,
} from '@tanstack/vue-table'
import type { ColumnDef } from '@tanstack/vue-table'
import type { Post } from '@/mock/db'
import StatusChip from '@/components/ui/StatusChip.vue'
import TableFrame from '@/components/tables/TableFrame.vue'

const props = defineProps<{ rows: Post[]; loading?: boolean }>()
const emit = defineEmits<{ select: [post: Post] }>()

const features = tableFeatures({
  rowSortingFeature,
  sortedRowModel: createSortedRowModel(),
  sortFns: { alphanumeric: sortFn_alphanumeric },
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

function onRowKeydown(e: KeyboardEvent, post: Post): void {
  if ((e.target as HTMLElement).closest('input[type=checkbox]')) return
  emit('select', post)
}
</script>

<template>
  <div>
    <div v-if="$slots.actions" class="tbl-tools flex items-center justify-center gap-2.5 mb-2">
      <slot name="actions" />
    </div>
    <div v-if="loading" class="text-muted text-sm" role="status">Loading rows…</div>
    <TableFrame v-else>
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
          :class="checked.has(row.original.id) ? 'selected' : ''" tabindex="0"
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
    <p class="text-muted text-sm">{{ checked.size }} selected · click a row to edit · headers sort</p>
  </div>
</template>
