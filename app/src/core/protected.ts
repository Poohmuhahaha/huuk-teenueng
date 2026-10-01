// "Do not show again for N minutes" suppression for the computed-cell guard.
const KEY = 'cp.guard.hideUntil'

export function guardSuppressed(): boolean {
  try {
    const raw = sessionStorage.getItem(KEY)
    if (!raw) return false
    const until = Number(raw)
    if (!Number.isFinite(until) || until <= Date.now()) {
      sessionStorage.removeItem(KEY)
      return false
    }
    return true
  } catch {
    return false
  }
}

export function suppressGuard(minutes = 5): void {
  try {
    sessionStorage.setItem(KEY, String(Date.now() + minutes * 60_000))
  } catch {
    // storage unavailable (private mode) — the guard simply keeps showing
  }
}
