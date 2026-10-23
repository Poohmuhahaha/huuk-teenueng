import { computed, type Ref } from 'vue'
import type { Post, LiveData, LivePost } from '@/mock/db'

export interface TrackedPostRef {
  id: string
  title: string
  date?: string | null
  platform?: string
  isLive?: boolean
  views?: number
  likes?: number
}

export interface TrackedTag {
  tag: string
  rawTag: string
  count: number
  group?: string
  posts: TrackedPostRef[]
  totalViews: number
  totalLikes: number
}

export function extractHashtagsFromText(text: string | null | undefined): string[] {
  if (!text) return []
  const matches = text.match(/#([A-Za-z0-9_\u0E00-\u0E7F]+)/g) || []
  return matches.map((m) => m.replace(/^#/, '').toLowerCase())
}

export function useHashtagTracker(
  allPosts: Ref<Post[] | undefined>,
  liveData?: Ref<LiveData | undefined>,
) {
  const trackedMap = computed<Map<string, TrackedTag>>(() => {
    const map = new Map<string, TrackedTag>()

    const recordTag = (
      rawTag: string,
      postRef: TrackedPostRef,
      groupName?: string,
    ) => {
      const normalized = rawTag.trim().toLowerCase().replace(/^#/, '')
      if (!normalized) return

      const existing = map.get(normalized) ?? {
        tag: normalized,
        rawTag: `#${normalized}`,
        count: 0,
        group: groupName,
        posts: [],
        totalViews: 0,
        totalLikes: 0,
      }

      existing.count += 1
      existing.posts.push(postRef)
      if (postRef.views) existing.totalViews += postRef.views
      if (postRef.likes) existing.totalLikes += postRef.likes
      if (!existing.group && groupName) existing.group = groupName

      map.set(normalized, existing)
    }

    // 1. Process calendar / planned posts
    const postsList = allPosts.value ?? []
    for (const p of postsList) {
      const tagsInPost = new Set<string>()

      if (Array.isArray(p.hashtags)) {
        for (const t of p.hashtags) {
          if (t) tagsInPost.add(t.toLowerCase().replace(/^#/, ''))
        }
      }

      if (p.caption) {
        for (const t of extractHashtagsFromText(p.caption)) {
          tagsInPost.add(t)
        }
      }

      const postRef: TrackedPostRef = {
        id: p.id,
        title: p.topic || 'Untitled Post',
        date: p.date,
        platform: p.platforms?.[0] || 'Studio',
        isLive: false,
      }

      for (const t of tagsInPost) {
        recordTag(t, postRef, p.hashtagGroup)
      }
    }

    // 2. Process fetched live platform posts
    const liveList: LivePost[] = liveData?.value?.posts ?? []
    for (const lp of liveList) {
      const liveTags = extractHashtagsFromText(lp.caption)
      const postRef: TrackedPostRef = {
        id: lp.id,
        title: lp.attachmentTitle || lp.caption?.slice(0, 30) || 'Live Feed Post',
        date: lp.createdAt?.slice(0, 10),
        platform: lp.platform,
        isLive: true,
        views: lp.views || 0,
        likes: lp.likes || 0,
      }

      for (const t of new Set(liveTags)) {
        recordTag(t, postRef)
      }
    }

    return map
  })

  const trackedList = computed<TrackedTag[]>(() => {
    return Array.from(trackedMap.value.values()).sort((a, b) => {
      if (b.count !== a.count) return b.count - a.count
      return a.tag.localeCompare(b.tag)
    })
  })

  const totalTrackedTags = computed(() => trackedMap.value.size)

  const totalPostsAnalyzed = computed(() => {
    const plannedCount = (allPosts.value ?? []).length
    const liveCount = (liveData?.value?.posts ?? []).length
    return plannedCount + liveCount
  })

  function getTagStat(tag: string): TrackedTag | undefined {
    return trackedMap.value.get(tag.trim().toLowerCase().replace(/^#/, ''))
  }

  function getPostCount(tag: string): number {
    return getTagStat(tag)?.count ?? 0
  }

  function getDiscoveredTagsForGroup(groupTitle: string, existingTags: string[]): TrackedTag[] {
    const existingSet = new Set(existingTags.map((t) => t.toLowerCase().replace(/^#/, '')))
    const normGroup = groupTitle.trim().toLowerCase()

    return trackedList.value.filter((stat) => {
      if (existingSet.has(stat.tag)) return false
      return stat.group?.trim().toLowerCase() === normGroup
    })
  }

  return {
    trackedMap,
    trackedList,
    totalTrackedTags,
    totalPostsAnalyzed,
    getTagStat,
    getPostCount,
    getDiscoveredTagsForGroup,
  }
}
