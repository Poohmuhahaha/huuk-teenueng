// The navbar date chooser is global: picking a date here drives the
// Monthly Planner (via routing) and the Smart Calendar (via this ref).
import { ref, watch } from 'vue'

export const navDate = ref<string | null>(null)

// One working month across the deck — Dashboard, Monthly Planner, Performance
// and the idea bank all read and write this single value, so two cards can
// never show different months.
export const activeMonth = ref<number>(new Date().getMonth() + 1)

// The matching working year — always changed together with activeMonth via
// the month toggle, so the whole deck looks at one (year, month).
export const activeYear = ref<number>(new Date().getFullYear())

export function yearOfDate(value: string | null): number | null {
  const year = typeof value === 'string' ? Number(value.slice(0, 4)) : NaN
  return Number.isInteger(year) && year >= 1970 && year <= 9999 ? year : null
}

watch(navDate, (value) => {
  const month = typeof value === 'string' ? Number(value.slice(5, 7)) : NaN
  if (Number.isInteger(month) && month >= 1 && month <= 12) activeMonth.value = month
  const year = yearOfDate(value)
  if (year !== null) activeYear.value = year
})
