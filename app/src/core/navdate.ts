// The navbar date chooser is global: picking a date here drives the
// Monthly Planner (via routing) and the Smart Calendar (via this ref).
import { ref } from 'vue'

export const navDate = ref<string | null>(null)
