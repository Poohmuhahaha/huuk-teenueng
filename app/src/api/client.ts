// Single-flight HTTP client & silent refresh queue per Master Engineering SOP §2.3.
import { z } from 'zod'
import { clearSession } from '@/core/session'
import { ApiErrorResponseSchema } from './schemas'

/** Error carrying the HTTP status, standardized error code, and server message. */
export class ApiError extends Error {
  readonly status: number
  readonly errorCode: string
  readonly details: Record<string, unknown> | null

  constructor(
    status: number,
    message: string,
    errorCode = 'API_ERROR',
    details: Record<string, unknown> | null = null,
  ) {
    super(message)
    this.name = 'ApiError'
    this.status = status
    this.errorCode = errorCode
    this.details = details
  }
}

/** Normalize the API origin: strip trailing slashes and a trailing `/api`. */
export function normalizeBase(raw: string | undefined | null): string {
  const trimmed = (raw ?? '').trim()
  if (!trimmed || trimmed === '/') return ''
  let base = trimmed.replace(/\/+$/, '')
  if (base.endsWith('/api')) base = base.slice(0, -4)
  if (!base.startsWith('/') && !/^https?:\/\//i.test(base)) base = `/${base}`
  return base.replace(/\/+$/, '')
}

const RAW_BASE = import.meta.env.VITE_API_URL

if (import.meta.env.PROD && !RAW_BASE && import.meta.env.VITE_USE_MOCK !== 'true') {
  console.info('[content-planner] VITE_API_URL is unset — using same-origin /api')
}

export const BASE = normalizeBase(RAW_BASE)
export const CREDENTIALS: RequestCredentials = BASE ? 'include' : 'same-origin'

let isRefreshing = false
let failedQueue: Array<{
  resolve: (value?: unknown) => void
  reject: (reason?: unknown) => void
}> = []

function processQueue(error: unknown = null): void {
  const queue = failedQueue
  failedQueue = []
  for (const prom of queue) {
    if (error) {
      prom.reject(error)
    } else {
      prom.resolve()
    }
  }
}

export async function parseApiError(res: Response): Promise<ApiError> {
  const text = await res.text().catch(() => '')
  if (text) {
    try {
      const json: unknown = JSON.parse(text)
      const parsed = ApiErrorResponseSchema.safeParse(json)
      if (parsed.success) {
        return new ApiError(
          res.status,
          parsed.data.message || parsed.data.error,
          parsed.data.error_code,
          parsed.data.details,
        )
      }
      if (
        typeof json === 'object' &&
        json !== null &&
        'error' in json &&
        typeof (json as { error?: unknown }).error === 'string' &&
        (json as { error: string }).error
      ) {
        return new ApiError(res.status, (json as { error: string }).error)
      }
      if (
        typeof json === 'object' &&
        json !== null &&
        'message' in json &&
        typeof (json as { message?: unknown }).message === 'string' &&
        (json as { message: string }).message
      ) {
        return new ApiError(res.status, (json as { message: string }).message)
      }
    } catch {
      // non-JSON body — fall through
    }
    if (text.length < 300) {
      return new ApiError(res.status, text)
    }
  }
  return new ApiError(res.status, `Request failed (${res.status})`)
}

export async function refreshSession(): Promise<void> {
  if (isRefreshing) {
    return new Promise<void>((resolve, reject) => {
      failedQueue.push({
        resolve: () => resolve(),
        reject,
      })
    })
  }
  isRefreshing = true
  try {
    const refreshRes = await fetch(`${BASE}/api/auth/refresh`, {
      method: 'POST',
      credentials: CREDENTIALS,
    })
    if (!refreshRes.ok) {
      throw await parseApiError(refreshRes)
    }
    processQueue(null)
  } catch (err) {
    processQueue(err)
    clearSession()
    throw err
  } finally {
    isRefreshing = false
  }
}

/**
 * Low-level fetch wrapper implementing the Master Engineering SOP §2.3
 * single-flight 401 refresh queue.
 */
export async function customFetch(url: string, options: RequestInit = {}): Promise<Response> {
  const fetchOptions: RequestInit = {
    ...options,
    credentials: options.credentials ?? CREDENTIALS,
  }

  let response = await fetch(url, fetchOptions)

  if (response.status === 401 && !url.includes('/api/auth/refresh')) {
    await refreshSession()
    response = await fetch(url, fetchOptions)
  }

  return response
}

/** Validates an unknown JSON payload using a Zod schema. */
export function validatePayload<T>(schema: z.ZodType<T>, data: unknown): T {
  return schema.parse(data)
}
