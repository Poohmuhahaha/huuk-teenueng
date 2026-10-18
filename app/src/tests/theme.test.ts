// @vitest-environment happy-dom
import { beforeEach, describe, expect, it } from 'vitest'
import { restoreBrandTheme, persistBrandTheme } from '@/core/theme'
import type { Brand } from '@/mock/db'

const KEY = 'cp.brand-theme'

function brand(): Brand {
  return {
    channel: '', positioning: '', slogan: '', audience: '', voice: '',
    dos: [], donts: [],
    palette: ['#123456'],
    fonts: [],
    logos: [], moodboard: [],
    radius: 8, fillOpacity: 100, strokeWidth: 1, shadow: 'soft',
  }
}

describe('brand theme persistence', () => {
  beforeEach(() => {
    localStorage.clear()
    document.documentElement.removeAttribute('style')
  })

  it('restores the remembered theme onto first paint', () => {
    persistBrandTheme(brand(), [])
    document.documentElement.removeAttribute('style')
    expect(restoreBrandTheme()).toBe(true)
    expect(document.documentElement.style.getPropertyValue('--accent')).toBe('#123456')
    expect(document.documentElement.style.getPropertyValue('--radius')).toBe('8px')
  })

  it('restores defaults when the remembered brand is empty', () => {
    persistBrandTheme(null, [])
    document.documentElement.style.setProperty('--accent', '#123456')
    expect(restoreBrandTheme()).toBe(true)
    expect(document.documentElement.style.getPropertyValue('--accent')).toBe('')
  })

  it('returns false with nothing or garbage stored', () => {
    expect(restoreBrandTheme()).toBe(false)
    localStorage.setItem(KEY, 'not-json{{{')
    expect(restoreBrandTheme()).toBe(false)
  })
})
