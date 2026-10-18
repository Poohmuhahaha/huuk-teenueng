// Session auth (US-012): email + password accounts.
// The backend keeps the session in an HttpOnly `cp_session` cookie, so the SPA
// never stores or reads a token (SOP Hard Rule 1); session.ts exposes the
// shared session state to the HTTP client and UI.
import { computed } from 'vue'
import api from '@/api'
import { isAuthError } from '@/api/http'
import { clearSession, currentUser, startSession } from './session'
import type { User } from '@/mock/db'

export { currentUser }

export const isLoggedIn = computed(() => currentUser.value !== null)
export const currentName = computed<string | null>(() => currentUser.value?.name ?? null)
export const currentRole = computed<string | null>(() => currentUser.value?.role ?? null)

export async function login(email: string, password: string): Promise<User> {
  const res = await api.login(email, password)
  startSession(res.user)
  return res.user
}

export async function register(v: { name: string; email: string; password: string }): Promise<User> {
  const res = await api.register(v)
  startSession(res.user)
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
  try {
    const res = await api.me()
    startSession(res.user)
  } catch (error) {
    // Only a rejected session clears the UI state — a network/5xx blip keeps
    // the cookie so the user is not logged out by an unreachable server.
    if (isAuthError(error)) clearSession()
    else console.warn('[content-planner] session restore deferred:', error)
  }
}

export async function logout(): Promise<void> {
  clearSession()
  await api.logout().catch(() => undefined)
}

export async function logoutAll(): Promise<void> {
  clearSession()
  await api.logoutAll().catch(() => undefined)
}
