// Session auth (US-012): email + password accounts.
// The backend hands back an opaque token; token.ts persists it for reloads and
// session.ts exposes the shared session state to the HTTP client and UI.
import { computed } from 'vue'
import api from '@/api'
import { isAuthError } from '@/api/http'
import { clearSession, currentUser, startSession } from './session'
import { readToken, token } from './token'
import type { User } from '@/mock/db'

export { currentUser, token }

export const isLoggedIn = computed(() => currentUser.value !== null)
export const currentName = computed<string | null>(() => currentUser.value?.name ?? null)
export const currentRole = computed<string | null>(() => currentUser.value?.role ?? null)

export async function login(email: string, password: string): Promise<User> {
  const res = await api.login(email, password)
  startSession(res.token, res.user)
  return res.user
}

export async function register(v: { name: string; email: string; password: string }): Promise<User> {
  const res = await api.register(v)
  startSession(res.token, res.user)
  return res.user
}

export async function changePassword(oldPassword: string, newPassword: string): Promise<void> {
  await api.changePassword(oldPassword, newPassword)
}

export async function updateProfile(name: string): Promise<User> {
  const res = await api.updateProfile(name)
  currentUser.value = res.user
  return res.user
}

/** Records the SaaS package chosen on the plans page. */
export async function choosePlan(plan: string): Promise<User> {
  const res = await api.setPlan(plan)
  currentUser.value = res.user
  return res.user
}

export async function restore(): Promise<void> {
  const t = token.value ?? readToken()
  if (!t) {
    clearSession()
    return
  }
  try {
    const res = await api.me(t)
    startSession(t, res.user)
  } catch (error) {
    // Only a rejected session clears the token — a network/5xx blip keeps it
    // so the user is not logged out by an unreachable server.
    if (isAuthError(error)) clearSession()
    else console.warn('[content-planner] session restore deferred:', error)
  }
}

export async function logout(): Promise<void> {
  const t = token.value
  clearSession()
  if (t) await api.logout(t).catch(() => undefined)
}

export async function logoutAll(): Promise<void> {
  const t = token.value
  clearSession()
  if (t) await api.logoutAll(t).catch(() => undefined)
}
