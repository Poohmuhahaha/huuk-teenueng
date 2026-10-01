// @vitest-environment happy-dom
import { describe, expect, it } from 'vitest'
import { mount } from '@vue/test-utils'
import FeedGrid from '@/components/ui/FeedGrid.vue'
import { FEED_META, PLATFORM_META, platformIdBySetupName } from '@/core/platforms'
import { posts } from '@/mock/db'

describe('platform feed sizing', () => {
  it('every platform has a valid feed spec', () => {
    for (const meta of Object.values(PLATFORM_META)) {
      expect(meta.feed.w).toBeGreaterThan(0)
      expect(meta.feed.h).toBeGreaterThan(0)
      expect(meta.feed.ratio.length).toBeGreaterThan(0)
      expect(Array.isArray(meta.feed.variants)).toBe(true)
    }
  })

  it('primary ratios match the platform specs', () => {
    expect(FEED_META.instagram.feed.ratio).toBe('1:1')
    expect(PLATFORM_META.meta.feed.ratio).toBe('1.91:1')
    expect(PLATFORM_META.youtube.feed.ratio).toBe('16:9')
    expect(PLATFORM_META.tiktok.feed.ratio).toBe('9:16')
  })

  it('resolves setup names to platform ids (case-insensitive)', () => {
    expect(platformIdBySetupName('Youtube')).toBe('youtube')
    expect(platformIdBySetupName('tiktok')).toBe('tiktok')
    // Legacy setup labels: Facebook is the same login as Meta, Instagram is a
    // planner-only platform (published through Meta).
    expect(platformIdBySetupName('Facebook')).toBe('meta')
    expect(platformIdBySetupName('Meta')).toBe('meta')
    expect(platformIdBySetupName('Instagram')).toBe('instagram')
    expect(platformIdBySetupName('Nope')).toBeUndefined()
  })
})

describe('FeedGrid', () => {
  it('renders tiles with the platform aspect ratio and size caption', () => {
    const w = mount(FeedGrid, { props: { posts: posts.slice(0, 2), platform: 'tiktok' } })
    const cells = w.findAll('.cell')
    expect(cells).toHaveLength(2)
    expect(cells[0].attributes('style')).toContain('1080')
    expect(w.text()).toContain('1080×1920')
  })

  it('uses the variant size when a variant index is given', () => {
    const w = mount(FeedGrid, { props: { posts: posts.slice(0, 2), platform: 'youtube', variant: 0 } })
    expect(w.find('.cell').attributes('style')).toContain('1080 / 1920')
    expect(w.text()).toContain('1080×1920')
  })
})
