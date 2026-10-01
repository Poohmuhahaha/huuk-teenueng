import { describe, expect, it } from 'vitest'
import { safeAuthorizeUrl } from '@/core/oauth'

describe('safeAuthorizeUrl', () => {
  it('accepts only https URLs on known provider hosts', () => {
    expect(
      safeAuthorizeUrl('https://www.facebook.com/v26.0/dialog/oauth?client_id=1&state=st-1'),
    ).toContain('www.facebook.com')
    expect(safeAuthorizeUrl('https://accounts.google.com/o/oauth2/v2/auth')).toContain(
      'accounts.google.com',
    )
    expect(safeAuthorizeUrl('https://www.tiktok.com/v2/auth/authorize/')).toContain('tiktok.com')
  })

  it('rejects other hosts, plain http, and non-URLs', () => {
    expect(safeAuthorizeUrl('https://evil.example.com/oauth')).toBeNull()
    expect(safeAuthorizeUrl('http://www.facebook.com/oauth')).toBeNull()
    expect(safeAuthorizeUrl('javascript:alert(1)')).toBeNull()
    expect(safeAuthorizeUrl('not a url')).toBeNull()
  })
})
