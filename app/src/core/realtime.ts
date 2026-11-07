// Server-sent change notifications.
//
// The Rust API exposes `/api/events` (SSE). Every successful mutation bumps a
// process-wide revision; this hook invalidates the active Vue Query cache when
// another tab/device changes something, so views stay live without polling.
import { onBeforeUnmount, onMounted } from 'vue'
import { useQueryClient } from '@tanstack/vue-query'
import { BASE } from '@/api/client'

export function useRealtimeUpdates(): void {
  const qc = useQueryClient()
  let source: EventSource | null = null
  let reconnectTimer: number | undefined
  let refreshTimer: number | undefined
  let lastRevision: number | null = null

  function scheduleRefresh(): void {
    window.clearTimeout(refreshTimer)
    refreshTimer = window.setTimeout(() => {
      void qc.invalidateQueries()
    }, 200)
  }

  function connect(): void {
    if (typeof EventSource === 'undefined') return
    source = new EventSource(`${BASE}/api/events`, { withCredentials: true })

    source.addEventListener('revision', (event) => {
      const revision = Number((event as MessageEvent).data)
      if (!Number.isFinite(revision)) return
      if (lastRevision === null) {
        lastRevision = revision
        return
      }
      if (revision === lastRevision) return
      lastRevision = revision
      scheduleRefresh()
    })

    // A reconnect may have missed a change while the connection was down.
    source.onopen = () => {
      if (lastRevision !== null) scheduleRefresh()
    }

    source.onerror = () => {
      source?.close()
      source = null
      if (reconnectTimer === undefined) {
        reconnectTimer = window.setTimeout(() => {
          reconnectTimer = undefined
          connect()
        }, 5000)
      }
    }
  }

  onMounted(connect)
  onBeforeUnmount(() => {
    source?.close()
    source = null
    window.clearTimeout(reconnectTimer)
    window.clearTimeout(refreshTimer)
  })
}
