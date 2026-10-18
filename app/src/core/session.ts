// Session state shared by auth, the HTTP client and the query cache.
// Kept free of imports from auth.ts / api to avoid cycles:
//   auth.ts → api → api/http.ts → session.ts
//
// The session itself lives in the server's HttpOnly `cp_session` cookie
// (SOP Hard Rule 1) — nothing in the SPA ever stores or reads a token.
import { ref } from 'vue'
import type { User } from '@/mock/db'

export const currentUser = ref<User | null>(null)

type SessionListener = () => void
const listeners: SessionListener[] = []

/** Registers a callback fired whenever the session is established or cleared
 * (used to drop cached query data on login/logout/session-expiry). */
export function onSessionChange(listener: SessionListener): void {
  listeners.push(listener)
}

function notify(): void {
  for (const listener of listeners) {
    try {
      listener()
    } catch {
      // listeners must never break the auth flow
    }
  }
}

export function startSession(user: User): void {
  currentUser.value = user
  notify()
}

export function clearSession(): void {
  const hadSession = currentUser.value !== null
  currentUser.value = null
  if (hadSession) notify()
}
