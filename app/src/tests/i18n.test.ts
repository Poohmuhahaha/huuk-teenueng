import { describe, expect, it } from 'vitest'
import { dicts, monthName, monthNameShort, t } from '@/core/i18n'

describe('i18n dictionary', () => {
  it('ships English only', () => {
    expect(Object.keys(dicts)).toEqual(['en'])
  })

  it('translates month names and abbreviations', () => {
    expect(monthName(1)).toBe('January')
    expect(monthNameShort(12)).toBe('Dec')
  })

  it('t() returns the English copy and falls back to the key', () => {
    expect(t('nav.plan')).toBe('Plan')
    expect(t('setup.year')).toBe('Year')
    expect(t('missing.key')).toBe('missing.key')
  })

  it('has no Thai-script copy left in the dictionary', () => {
    for (const [key, value] of Object.entries(dicts.en)) {
      // The baht sign (U+0E3F) is currency, not language copy.
      const thai = /[\u0E00-\u0E3E\u0E40-\u0E7F]/.test(value)
      expect(thai, `${key} still contains Thai text`).toBe(false)
    }
  })
})
