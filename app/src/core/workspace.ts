// Active workspace storage, isolated so the HTTP client can read it without a
// cycle: queries.ts → api → api/http.ts would otherwise need to import queries
// back. The server resolves a missing header to the first workspace.
import { ref } from 'vue'

export const WORKSPACE_KEY = 'cp.workspace'

// localStorage is absent in some test/runtime setups (happy-dom is fine) — guard it.
function storage(): Storage | null {
  return typeof localStorage === 'undefined' ? null : localStorage
}

export function readWorkspace(): string | null {
  const s = storage()
  if (!s) return null
  try {
    return s.getItem(WORKSPACE_KEY)
  } catch {
    return null
  }
}

/** Id of the workspace every API request targets; `null` = server default. */
export const activeWorkspaceId = ref<string | null>(readWorkspace())

export function setActiveWorkspace(id: string | null): void {
  activeWorkspaceId.value = id
  const s = storage()
  if (!s) return
  try {
    if (id === null) s.removeItem(WORKSPACE_KEY)
    else s.setItem(WORKSPACE_KEY, id)
  } catch {
    // storage can be unavailable (private mode, quota) — keep the in-memory id
  }
}
