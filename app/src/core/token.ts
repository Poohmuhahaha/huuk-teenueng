// Session token storage, isolated so the HTTP client can read it without a cycle:
// auth.ts → api → api/http.ts would otherwise need to import auth.ts back.
import { ref } from 'vue'

export const TOKEN_KEY = 'cp.token'

// localStorage is absent in some test/runtime setups (happy-dom is fine) — guard it.
function storage(): Storage | null {
  return typeof localStorage === 'undefined' ? null : localStorage
}

export function readToken(): string | null {
  const s = storage()
  if (!s) return null
  try {
    return s.getItem(TOKEN_KEY)
  } catch {
    return null
  }
}

export const token = ref<string | null>(readToken())

export function setToken(v: string | null): void {
  token.value = v
  const s = storage()
  if (!s) return
  try {
    if (v === null) s.removeItem(TOKEN_KEY)
    else s.setItem(TOKEN_KEY, v)
  } catch {
    // storage can be unavailable (private mode, quota) — keep the in-memory token
  }
}
