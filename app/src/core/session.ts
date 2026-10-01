// Session state shared by auth, the HTTP client and the query cache.
// Kept free of imports from auth.ts / api to avoid cycles:
//   auth.ts → api → api/http.ts → session.ts → token.ts
import { ref } from 'vue'
import type { User } from '@/mock/db'
import { setToken, token } from './token'

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

export function startSession(newToken: string, user: User): void {
  setToken(newToken)
  currentUser.value = user
  notify()
}

export function clearSession(): void {
  const hadSession = token.value !== null || currentUser.value !== null
  setToken(null)
  currentUser.value = null
  if (hadSession) notify()
}
