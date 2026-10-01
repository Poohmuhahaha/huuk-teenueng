// Data-source switch.
//
// - dev without VITE_API_URL (or VITE_USE_MOCK=true) → in-memory mock
// - otherwise                                       → HTTP client → Rust backend
//
// The mock is a dynamic import on purpose: production builds fold the flag to
// `false`, so Vite drops the branch and the demo data/credentials never ship.
import type { Api } from './contract'
import * as http from './http'

const useMock = import.meta.env.VITE_USE_MOCK === 'true'
  || (!import.meta.env.VITE_API_URL && import.meta.env.DEV)

/** True when the app runs on the in-memory mock (dev / explicit opt-in). */
export const usingMock = useMock

const api: Api = useMock
  ? ((await import('@/mock/api')) as unknown as Api)
  : http

export default api
