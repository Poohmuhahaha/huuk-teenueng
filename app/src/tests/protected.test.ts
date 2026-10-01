// @vitest-environment happy-dom
import { afterEach, describe, expect, it, vi } from 'vitest'
import { guardSuppressed, suppressGuard } from '@/core/protected'

afterEach(() => {
  sessionStorage.clear()
  vi.useRealTimers()
})

describe('computed-cell guard suppression', () => {
  it('suppresses until the timeout passes', () => {
    vi.useFakeTimers()
    expect(guardSuppressed()).toBe(false)
    suppressGuard(5)
    expect(guardSuppressed()).toBe(true)

    vi.advanceTimersByTime(5 * 60_000 + 1)
    expect(guardSuppressed()).toBe(false)
  })

  it('ignores corrupt timestamps', () => {
    sessionStorage.setItem('cp.guard.hideUntil', 'not-a-number')
    expect(guardSuppressed()).toBe(false)
  })
})
