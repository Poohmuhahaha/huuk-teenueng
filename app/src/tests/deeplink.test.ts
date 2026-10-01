import { describe, expect, it } from 'vitest'
import { monthOfDate, plannerPathForDate, rewriteDeepLink } from '@/core/deeplink'

describe('rewriteDeepLink', () => {
  it('turns a bare path into its hash route', () => {
    expect(rewriteDeepLink({ pathname: '/members', search: '', hash: '' })).toBe('/#/members')
    expect(rewriteDeepLink({ pathname: '/studio/abc', search: '', hash: '' })).toBe('/#/studio/abc')
  })

  it('keeps the query string', () => {
    expect(rewriteDeepLink({ pathname: '/ads', search: '?tab=meta', hash: '' })).toBe(
      '/#/ads?tab=meta',
    )
  })

  it('leaves hash links, the root and index.html alone', () => {
    expect(rewriteDeepLink({ pathname: '/', search: '', hash: '#/members' })).toBeNull()
    expect(rewriteDeepLink({ pathname: '/', search: '', hash: '' })).toBeNull()
    expect(rewriteDeepLink({ pathname: '/index.html', search: '', hash: '' })).toBeNull()
    expect(rewriteDeepLink({ pathname: '/members', search: '', hash: '#/members' })).toBeNull()
  })
})

describe('plannerPathForDate', () => {
  it('jumps to the picked month in the Monthly Planner', () => {
    expect(plannerPathForDate('2026-09-24')).toBe('/planner/9')
    expect(plannerPathForDate('2026-01-05')).toBe('/planner/1')
    expect(plannerPathForDate('2026-12-31')).toBe('/planner/12')
  })

  it('rejects empty and malformed dates', () => {
    expect(plannerPathForDate(null)).toBeNull()
    expect(plannerPathForDate('')).toBeNull()
    expect(plannerPathForDate('2026-13-01')).toBeNull()
    expect(plannerPathForDate('2026-00-10')).toBeNull()
    expect(plannerPathForDate('not-a-date')).toBeNull()
  })
})

describe('monthOfDate', () => {
  it('extracts the month the Smart Calendar should show', () => {
    expect(monthOfDate('2026-09-24')).toBe(9)
    expect(monthOfDate('2026-02-01')).toBe(2)
    expect(monthOfDate(null)).toBeNull()
    expect(monthOfDate('2026-13-01')).toBeNull()
  })
})
