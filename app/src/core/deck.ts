// Full-page state of the card deck, shared with pages so they can adapt their
// layout (e.g. the calendar showing Today + Calendar side by side).
import { ref } from 'vue'

export const deckFull = ref(true)

// Live peek: path (`Screen.to`) of the card covering most of the deck frame
// while it scrolls — updated synchronously on every scroll event, so the
// topbar hub label follows the finger mid-drag instead of waiting for the
// release/settle commit. Null when the deck is at rest on the route's card.
export const deckPeek = ref<string | null>(null)
