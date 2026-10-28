<script setup lang="ts">
import { computed, ref } from 'vue'
import type { Post } from '@/mock/db'
import { FEED_META, type FeedPlatformId } from '@/core/platforms'
import PlatformLogo from '@/components/ui/PlatformLogo.vue'

const props = withDefaults(
  defineProps<{ posts: Post[]; platform: FeedPlatformId; variant?: number }>(),
  { variant: -1 },
)

// Current canvas size: primary feed default, or a variant by index.
// Unknown platform ids fall back to the default feed spec instead of crashing.
const size = computed(() => {
  const feed = (FEED_META[props.platform] ?? FEED_META.meta).feed
  return props.variant >= 0 && props.variant < feed.variants.length ? feed.variants[props.variant] : feed
})

const platformName = computed(() => (FEED_META[props.platform] ?? FEED_META.meta).name)
const tileStyle = computed(() => ({ aspectRatio: `${size.value.w} / ${size.value.h}` }))

// Consistent, balanced grid layout based on aspect ratio
const gridClass = computed(() => {
  // Vertical 9:16 (TikTok, Shorts, Reels)
  if (size.value.h > size.value.w) {
    return 'grid gap-4 sm:gap-5 grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5'
  }
  // Square 1:1 (Instagram)
  if (size.value.h === size.value.w) {
    return 'grid gap-4 sm:gap-5 grid-cols-2 sm:grid-cols-3 lg:grid-cols-4'
  }
  // Landscape / Wide (Meta 1.91:1, YouTube 16:9)
  return 'grid gap-4 sm:gap-5 grid-cols-1 sm:grid-cols-2 lg:grid-cols-3'
})

// Interactive Post Simulator Modal
const activePost = ref<Post | null>(null)
const likedPosts = ref<Record<string, boolean>>({})
const savedPosts = ref<Record<string, boolean>>({})
const copiedNotice = ref(false)

function openPreview(post: Post): void {
  activePost.value = post
}

function closePreview(): void {
  activePost.value = null
  copiedNotice.value = false
}

function toggleLike(postId: string): void {
  likedPosts.value[postId] = !likedPosts.value[postId]
}

function toggleSave(postId: string): void {
  savedPosts.value[postId] = !savedPosts.value[postId]
}

async function copyCaption(text: string): Promise<void> {
  try {
    await navigator.clipboard.writeText(text)
    copiedNotice.value = true
    setTimeout(() => { copiedNotice.value = false }, 2000)
  } catch {
    // fallback
  }
}

function statusBadgeClass(status?: string): string {
  switch (status) {
    case 'Done':
      return 'bg-ink text-white border-ink'
    case 'Dev':
    case 'Design':
      return 'bg-wash text-ink border-line'
    default:
      return 'bg-wash text-muted border-line'
  }
}

function simLikes(p: Post): string {
  const seed = (p.topic.length * 137 + (p.date ? p.date.charCodeAt(p.date.length - 1) : 42)) % 450 + 120
  return (seed * 10).toLocaleString()
}

function simComments(p: Post): string {
  const seed = (p.topic.length * 31 + (p.date ? p.date.charCodeAt(0) : 19)) % 80 + 12
  return seed.toString()
}
</script>

<template>
  <div class="feed">
    <div :class="gridClass" role="list">
      <div
        v-for="p in posts"
        :key="p.id"
        role="listitem"
        tabindex="0"
        :aria-label="p.topic"
        :title="p.topic"
        class="cell group relative flex flex-col justify-between overflow-hidden rounded-xl border border-line bg-surface p-3.5 sm:p-4 text-left shadow-2xs hover:shadow-md hover:border-ink/30 transition-all duration-200 cursor-pointer select-none"
        :style="tileStyle"
        @click="openPreview(p)"
        @keydown.enter="openPreview(p)"
      >
        <!-- Optional Cover Image Background -->
        <template v-if="p.imageUrl">
          <img
            :src="p.imageUrl"
            :alt="p.topic"
            class="absolute inset-0 w-full h-full object-cover transition-transform duration-500 group-hover:scale-105"
            loading="lazy"
          />
          <div class="absolute inset-0 bg-gradient-to-t from-black/80 via-black/30 to-black/50 pointer-events-none" />
        </template>
        <div v-else class="absolute inset-0 bg-wash/30 pointer-events-none" />

        <!-- Card Top Bar: Platform + Format + Status -->
        <div class="relative z-10 flex items-center justify-between gap-2">
          <div class="flex items-center gap-1.5 min-w-0">
            <PlatformLogo :id="platform" :size="15" :tile="true" />
            <span
              class="text-xs font-semibold truncate"
              :class="p.imageUrl ? 'text-white' : 'text-ink'"
            >
              {{ platformName }}
            </span>
            <span
              v-if="p.format"
              class="text-[10px] px-1.5 py-0.5 rounded border truncate"
              :class="p.imageUrl ? 'bg-black/50 text-white/90 border-white/20' : 'bg-wash text-muted border-line/60'"
            >
              {{ p.format }}
            </span>
          </div>

          <span
            v-if="p.status"
            class="px-2 py-0.5 rounded-full text-[10px] font-bold uppercase tracking-wider border shrink-0"
            :class="p.imageUrl ? 'bg-black/60 text-white border-white/20' : statusBadgeClass(p.status)"
          >
            {{ p.status }}
          </span>
        </div>

        <!-- Card Center / Body: Topic & Hook -->
        <div class="relative z-10 my-auto py-1">
          <span
            v-if="p.pillar"
            class="text-[9.5px] uppercase font-bold tracking-wider block mb-0.5"
            :class="p.imageUrl ? 'text-white/70' : 'text-muted/70'"
          >
            {{ p.pillar }}
          </span>

          <div
            class="font-bold text-sm sm:text-base line-clamp-2 leading-snug tracking-tight group-hover:text-accent transition-colors"
            :class="p.imageUrl ? 'text-white' : 'text-ink'"
          >
            {{ p.topic || 'Untitled topic' }}
          </div>

          <p
            v-if="p.hook"
            class="text-xs line-clamp-1 italic mt-0.5"
            :class="p.imageUrl ? 'text-white/80' : 'text-muted'"
          >
            “{{ p.hook }}”
          </p>
        </div>

        <!-- Card Bottom Bar: Date, Engagement, Dimension -->
        <div
          class="relative z-10 flex items-center justify-between gap-2 pt-1 border-t"
          :class="p.imageUrl ? 'border-white/15 text-white/85' : 'border-line/60 text-muted'"
        >
          <div class="flex items-center gap-2 text-xs">
            <span v-if="p.date" class="font-mono text-[11px]">{{ p.date }}</span>
            <span class="inline-flex items-center gap-1 font-mono text-[10px]">
              <svg class="w-3 h-3 fill-current opacity-60" viewBox="0 0 24 24"><path d="M20.84 4.61a5.5 5.5 0 0 0-7.78 0L12 5.67l-1.06-1.06a5.5 5.5 0 0 0-7.78 7.78l1.06 1.06L12 21.23l7.78-7.78 1.06-1.06a5.5 5.5 0 0 0 0-7.78z"/></svg>
              {{ simLikes(p) }}
            </span>
          </div>

          <span
            class="px-1.5 py-0.5 rounded text-[10px] font-mono font-medium border shrink-0"
            :class="p.imageUrl ? 'bg-black/60 text-white/90 border-white/20' : 'bg-wash text-muted border-line/60'"
          >
            {{ size.w }}×{{ size.h }}
          </span>
        </div>
      </div>
    </div>
  </div>

  <p v-if="!posts.length" role="status" class="text-muted text-sm mt-3">Nothing scheduled in range.</p>

  <!-- Interactive Social Post Simulator Modal -->
  <div
    v-if="activePost"
    class="fixed inset-0 z-[1000] bg-black/60 backdrop-blur-xs flex items-center justify-center p-4 overflow-y-auto"
    @click.self="closePreview"
  >
    <div
      class="w-full max-w-lg bg-surface border border-line rounded-2xl shadow-2xl overflow-hidden animate-in fade-in zoom-in-95 duration-150"
      role="dialog"
      aria-modal="true"
    >
      <!-- Simulator Modal Header -->
      <div class="flex items-center justify-between px-5 py-3.5 border-b border-line bg-wash/50">
        <div class="flex items-center gap-2">
          <PlatformLogo :id="platform" :size="20" :tile="true" />
          <div>
            <h4 class="font-bold text-sm text-ink leading-tight m-0">{{ platformName }} Post Preview</h4>
            <span class="text-xs text-muted font-mono">{{ size.w }}×{{ size.h }} · {{ size.ratio }}</span>
          </div>
        </div>
        <button
          type="button"
          class="w-7 h-7 rounded-full flex items-center justify-center text-muted hover:text-ink hover:bg-wash transition cursor-pointer"
          aria-label="Close preview"
          @click="closePreview"
        >
          ✕
        </button>
      </div>

      <!-- Native Social Card Body -->
      <div class="p-5 flex flex-col gap-4">
        <!-- Post Header -->
        <div class="flex items-center justify-between">
          <div class="flex items-center gap-3">
            <div class="w-10 h-10 rounded-full bg-accent/15 border border-accent/30 flex items-center justify-center font-bold text-ink text-sm">
              {{ (activePost.topic || 'S').slice(0, 1).toUpperCase() }}
            </div>
            <div>
              <div class="flex items-center gap-1.5">
                <span class="font-bold text-sm text-ink">Studio Channel</span>
                <span class="text-xs text-muted">@studio.channel</span>
                <span class="text-xs text-muted">·</span>
                <span class="text-xs text-muted">{{ activePost.date || 'Scheduled' }}</span>
              </div>
              <div class="flex items-center gap-2 mt-0.5">
                <span v-if="activePost.pillar" class="text-[10px] font-semibold px-2 py-0.5 rounded bg-wash border border-line text-muted">
                  {{ activePost.pillar }}
                </span>
                <span v-if="activePost.format" class="text-[10px] font-semibold px-2 py-0.5 rounded bg-wash border border-line text-muted">
                  {{ activePost.format }}
                </span>
                <span v-if="activePost.status" class="text-[10px] font-bold px-2 py-0.5 rounded uppercase" :class="statusBadgeClass(activePost.status)">
                  {{ activePost.status }}
                </span>
              </div>
            </div>
          </div>

          <button type="button" class="px-3 py-1 rounded-full text-xs font-semibold bg-accent text-accent-ink hover:opacity-90 transition">
            Follow
          </button>
        </div>

        <!-- Post Artwork Canvas -->
        <div
          class="w-full relative rounded-xl overflow-hidden border border-line shadow-xs flex items-center justify-center bg-wash/50"
          :style="tileStyle"
        >
          <img
            v-if="activePost.imageUrl"
            :src="activePost.imageUrl"
            :alt="activePost.topic"
            class="w-full h-full object-cover"
          />
          <div
            v-else
            class="w-full h-full p-6 flex flex-col justify-between text-ink relative"
          >
            <div class="flex items-center justify-between z-10">
              <span class="px-2.5 py-1 rounded-full bg-surface border border-line text-xs font-semibold text-ink shadow-2xs">
                {{ platformName }}
              </span>
              <span class="text-xs font-mono px-2 py-0.5 rounded bg-surface border border-line text-muted">
                {{ size.w }}×{{ size.h }}
              </span>
            </div>

            <div class="flex flex-col items-center justify-center my-auto text-center z-10 px-4">
              <div class="font-bold text-lg sm:text-xl text-ink tracking-tight mb-1">
                {{ activePost.topic }}
              </div>
              <p v-if="activePost.hook" class="text-sm text-muted italic max-w-md m-0">
                “{{ activePost.hook }}”
              </p>
            </div>

            <div class="flex items-center justify-between text-xs text-muted z-10">
              <span>{{ activePost.date }}</span>
              <span>{{ activePost.time || '12:00' }}</span>
            </div>
          </div>
        </div>

        <!-- Social Action Bar -->
        <div class="flex items-center justify-between pt-1 border-t border-line">
          <div class="flex items-center gap-4">
            <button
              type="button"
              class="flex items-center gap-1.5 text-xs font-semibold cursor-pointer transition"
              :class="likedPosts[activePost.id] ? 'text-red-500' : 'text-muted hover:text-ink'"
              @click="toggleLike(activePost.id)"
            >
              <svg class="w-5 h-5 fill-current" viewBox="0 0 24 24"><path d="M20.84 4.61a5.5 5.5 0 0 0-7.78 0L12 5.67l-1.06-1.06a5.5 5.5 0 0 0-7.78 7.78l1.06 1.06L12 21.23l7.78-7.78 1.06-1.06a5.5 5.5 0 0 0 0-7.78z"/></svg>
              <span>{{ likedPosts[activePost.id] ? 'Liked' : simLikes(activePost) }}</span>
            </button>

            <button
              type="button"
              class="flex items-center gap-1.5 text-xs font-semibold text-muted hover:text-ink cursor-pointer transition"
            >
              <svg class="w-5 h-5 fill-current" viewBox="0 0 24 24"><path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z"/></svg>
              <span>{{ simComments(activePost) }}</span>
            </button>

            <button
              type="button"
              class="flex items-center gap-1.5 text-xs font-semibold text-muted hover:text-ink cursor-pointer transition"
              @click="copyCaption(activePost.caption || activePost.topic)"
            >
              <svg class="w-5 h-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="18" cy="5" r="3"/><circle cx="6" cy="12" r="3"/><circle cx="18" cy="19" r="3"/><line x1="8.59" y1="13.51" x2="15.42" y2="17.49"/><line x1="15.41" y1="6.51" x2="8.59" y2="10.49"/></svg>
              <span>Share</span>
            </button>
          </div>

          <button
            type="button"
            class="flex items-center gap-1.5 text-xs font-semibold cursor-pointer transition"
            :class="savedPosts[activePost.id] ? 'text-accent' : 'text-muted hover:text-ink'"
            @click="toggleSave(activePost.id)"
          >
            <svg class="w-5 h-5 fill-current" viewBox="0 0 24 24"><path d="M19 21l-7-5-7 5V5a2 2 0 0 1 2-2h10a2 2 0 0 1 2 2z"/></svg>
            <span>{{ savedPosts[activePost.id] ? 'Saved' : 'Save' }}</span>
          </button>
        </div>

        <!-- Post Caption & Details -->
        <div class="flex flex-col gap-2 text-sm bg-wash/40 p-4 rounded-xl border border-line/60">
          <div class="flex items-start justify-between gap-3">
            <p class="text-ink leading-relaxed font-normal m-0">
              <strong class="font-bold text-ink mr-1">studio.channel</strong>
              {{ activePost.caption || activePost.hook || activePost.topic }}
            </p>
            <button
              type="button"
              class="shrink-0 px-2.5 py-1 rounded text-xs font-semibold bg-surface border border-line text-muted hover:text-ink hover:bg-wash transition cursor-pointer"
              @click="copyCaption(activePost.caption || activePost.topic)"
            >
              {{ copiedNotice ? 'Copied! ✓' : 'Copy' }}
            </button>
          </div>

          <!-- Hashtags -->
          <div v-if="activePost.hashtags?.length" class="flex flex-wrap gap-1.5 mt-1">
            <span
              v-for="tag in activePost.hashtags"
              :key="tag"
              class="text-xs font-mono font-medium text-accent hover:underline cursor-pointer"
            >
              #{{ tag }}
            </span>
          </div>

          <!-- CTA / Goal -->
          <div v-if="activePost.cta || activePost.goal" class="flex items-center justify-between pt-2 border-t border-line/40 text-xs text-muted">
            <span v-if="activePost.cta">👉 <strong class="text-ink font-semibold">{{ activePost.cta }}</strong></span>
            <span v-if="activePost.goal">🎯 Goal: <strong class="text-ink font-semibold">{{ activePost.goal }}</strong></span>
          </div>
        </div>
      </div>

      <!-- Simulator Footer -->
      <div class="px-5 py-3 border-t border-line bg-wash/30 flex items-center justify-between">
        <span class="text-xs text-muted font-mono">Planned post #{{ activePost.id }}</span>
        <button
          type="button"
          class="btn px-4 py-1.5 text-xs font-semibold"
          @click="closePreview"
        >
          Done
        </button>
      </div>
    </div>
  </div>
</template>