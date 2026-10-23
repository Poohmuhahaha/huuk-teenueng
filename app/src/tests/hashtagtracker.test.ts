import { describe, it, expect } from 'vitest'
import { ref } from 'vue'
import { useHashtagTracker, extractHashtagsFromText } from '@/composables/useHashtagTracker'
import type { Post, LiveData } from '@/mock/db'

describe('useHashtagTracker', () => {
  it('extracts hashtags from caption text correctly', () => {
    const text = 'Morning routine vlog with #selfcare and #morning_glow plus #สโลว์ไลฟ์'
    const tags = extractHashtagsFromText(text)
    expect(tags).toContain('selfcare')
    expect(tags).toContain('morning_glow')
    expect(tags).toContain('สโลว์ไลฟ์')
  })

  it('tracks hashtags and aggregates frequency from planned and live posts', () => {
    const mockPosts = ref<Post[]>([
      {
        id: 'p-1',
        month: 10,
        year: 2026,
        topic: 'Desk Tour',
        pillar: 'Pillar I',
        format: 'Short-video',
        goal: 'Growth',
        date: '2026-10-01',
        time: '09:00',
        status: 'Done',
        hook: 'Desk',
        caption: 'My setup #desksetup #minimal',
        cta: '',
        hashtagGroup: 'Reach',
        hashtags: ['desksetup', 'productivity'],
        imageUrl: '',
        note: '',
        done: true,
        platforms: ['Instagram'],
      },
      {
        id: 'p-2',
        month: 10,
        year: 2026,
        topic: 'Lighting Hacks',
        pillar: 'Pillar II',
        format: 'Short-video',
        goal: 'Growth',
        date: '2026-10-03',
        time: '18:00',
        status: 'Done',
        hook: 'Lights',
        caption: 'Hacks for filming #lighting',
        cta: '',
        hashtagGroup: 'Niche',
        hashtags: ['lighting', 'desksetup'],
        imageUrl: '',
        note: '',
        done: true,
        platforms: ['TikTok'],
      },
    ])

    const mockLive = ref<LiveData>({
      fetchedAt: 1772340000,
      accounts: [],
      error: '',
      posts: [
        {
          id: 'lp-1',
          platform: 'instagram',
          kind: 'photo',
          caption: 'BTS studio tour #desksetup #bts',
          mediaUrl: '',
          thumbnailUrl: '',
          permalink: '',
          createdAt: '2026-10-02T10:00:00',
          likes: 250,
          comments: 10,
          shares: 2,
          views: 1200,
          reactions: {},
          reach: 1000,
          saves: 45,
          clicks: 12,
          linkClicks: 0,
          videoLength: 0,
          videoAvgWatchTime: 0,
          attachmentTitle: 'Studio Tour',
          linkUrl: '',
          fetchedAt: 1772340000,
        },
      ],
    })

    const {
      trackedList,
      totalTrackedTags,
      totalPostsAnalyzed,
      getTagStat,
      getPostCount,
    } = useHashtagTracker(mockPosts, mockLive)

    expect(totalPostsAnalyzed.value).toBe(3)
    // Tags: desksetup, productivity, minimal, lighting, bts -> 5 tags
    expect(totalTrackedTags.value).toBe(5)

    // 'desksetup' is in p-1 (hashtags + caption), p-2 (hashtags), lp-1 (caption) -> total 3 posts
    expect(getPostCount('desksetup')).toBe(3)
    expect(getPostCount('#desksetup')).toBe(3)

    const deskStat = getTagStat('desksetup')
    expect(deskStat?.count).toBe(3)
    expect(deskStat?.posts.length).toBe(3)
    expect(deskStat?.totalViews).toBe(1200)

    // 'lighting' is in p-2 -> 1 post
    expect(getPostCount('lighting')).toBe(1)

    // Order: desksetup (3) should be first
    expect(trackedList.value[0].tag).toBe('desksetup')
  })
})
