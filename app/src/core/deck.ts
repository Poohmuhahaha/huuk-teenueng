// Full-page state of the card deck, shared with pages so they can adapt their
// layout (e.g. the calendar showing Today + Calendar side by side).
import { ref } from 'vue'

export const deckFull = ref(false)
