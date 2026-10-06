// The navbar date chooser is global: picking a date here drives the
// Monthly Planner (via routing) and the Smart Calendar (via this ref).
import { ref, watch } from 'vue'

export const navDate = ref<string | null>(null)

// One working month across the deck — Dashboard, Monthly Planner, Performance
// and the idea bank all read and write this single value, so two cards can
// never show different months.
export const activeMonth = ref<number>(new Date().getMonth() + 1)

watch(navDate, (value) => {
  const month = typeof value === 'string' ? Number(value.slice(5, 7)) : NaN
  if (Number.isInteger(month) && month >= 1 && month <= 12) activeMonth.value = month
})
