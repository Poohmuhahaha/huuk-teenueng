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
const wide = computed(() => size.value.w > size.value.h)
const platformName = computed(() => (FEED_META[props.platform] ?? FEED_META.meta).name)
const tileStyle = computed(() => ({ aspectRatio: `${size.value.w} / ${size.value.h}` }))

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

// Visual helpers for the canvas cards
function getCardGradient(p: Post): string {
  if (p.pillar === 'Pillar I') {
    return 'linear-gradient(135deg, #1e1b4b 0%, #0f172a 50%, #020617 100%)'
  }
  if (p.pillar === 'Pillar II') {
    return 'linear-gradient(135deg, #2a1608 0%, #1c1917 50%, #0c0a09 100%)'
  }
  if (p.pillar === 'Pillar III') {
    return 'linear-gradient(135deg, #052e16 0%, #092015 50%, #020805 100%)'
  }
  return 'linear-gradient(135deg, #18181b 0%, #09090b 50%, #000000 100%)'
}

function formatIcon(format?: string): string {
  const f = (format || '').toLowerCase()
  if (f.includes('short') || f.includes('reel') || f.includes('tiktok')) return '▶'
  if (f.includes('video') || f.includes('long')) return '🎬'
  if (f.includes('carousel') || f.includes('album')) return '🗂'
  if (f.includes('photo') || f.includes('image')) return '📷'
  return '✦'
}

function statusBadgeClass(status?: string): string {
  switch (status) {
    case 'Done':
      return 'bg-emerald-500/20 text-emerald-300 border-emerald-500/40'
    case 'Dev':
      return 'bg-sky-500/20 text-sky-300 border-sky-500/40'
    case 'Design':
      return 'bg-amber-500/20 text-amber-300 border-amber-500/40'
    default:
      return 'bg-white/10 text-white/70 border-white/20'
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
  <div class="feed" :class="{ wide }">
    <div
      class="grid gap-3.5 sm:gap-4.5 grid-cols-[repeat(auto-fill,minmax(200px,1fr))] [grid-auto-flow:dense]"
      role="list"
    >
      <div
        v-for="p in posts"
        :key="p.id"
        role="listitem"
        tabindex="0"
        :aria-label="p.topic"
        :title="p.topic"
        class="cell group relative flex flex-col justify-between overflow-hidden rounded-2xl border border-line/80 bg-surface shadow-xs hover:shadow-2xl hover:border-accent/80 transition-all duration-300 cursor-pointer select-none"
        :class="wide ? 'col-span-2' : ''"
        :style="tileStyle"
        @click="openPreview(p)"
        @keydown.enter="openPreview(p)"
      >
        <!-- Background Media or Editorial Canvas Poster -->
        <div class="absolute inset-0 w-full h-full overflow-hidden">
          <img
            v-if="p.imageUrl"
            :src="p.imageUrl"
            :alt="p.topic"
            class="w-full h-full object-cover transition-transform duration-700 ease-out group-hover:scale-105"
            loading="lazy"
          />
          <div
            v-else
            class="w-full h-full relative p-3 sm:p-4 flex flex-col justify-between overflow-hidden"
            :style="{ background: getCardGradient(p) }"
          >
            <div class="absolute inset-0 bg-[radial-gradient(ellipse_at_top,_var(--tw-gradient-stops))] from-white/10 via-transparent to-transparent pointer-events-none" />
            <div class="absolute -right-8 -bottom-8 w-36 h-36 rounded-full border border-white/5 pointer-events-none" />
            
            <!-- Platform watermark -->
            <div class="absolute right-2 bottom-10 opacity-5 group-hover:opacity-10 transition-opacity duration-300 pointer-events-none scale-150">
              <PlatformLogo :id="platform" :size="72" :color="false" />
            </div>
          </div>
        </div>

        <!-- Top Gradient Overlay -->
        <div class="absolute inset-x-0 top-0 h-24 bg-gradient-to-b from-black/80 via-black/40 to-transparent pointer-events-none z-10" />

        <!-- Bottom Gradient Overlay -->
        <div class="absolute inset-x-0 bottom-0 h-36 bg-gradient-to-t from-black/90 via-black/60 to-transparent pointer-events-none z-10" />

        <!-- Card Header: Platform Tag, Format, Status -->
        <div class="relative z-20 flex items-center justify-between gap-1.5 p-3">
          <div class="flex items-center gap-1.5 min-w-0">
            <span class="inline-flex items-center gap-1 px-2 py-0.5 rounded-full bg-black/60 backdrop-blur-md border border-white/15 text-white text-[11px] font-semibold tracking-wide shadow-xs">
              <PlatformLogo :id="platform" :size="12" :color="true" />
              <span class="truncate">{{ platformName }}</span>
            </span>
            <span v-if="p.format" class="hidden sm:inline-flex items-center gap-1 px-1.5 py-0.5 rounded-full bg-white/10 backdrop-blur-md border border-white/10 text-white/90 text-[10px] font-medium">
              <span>{{ formatIcon(p.format) }}</span>
              <span>{{ p.format }}</span>
            </span>
          </div>

          <div class="flex items-center gap-1 shrink-0">
            <span
              v-if="p.status"
              class="px-2 py-0.5 rounded-full text-[10px] font-bold uppercase tracking-wider backdrop-blur-md border"
              :class="statusBadgeClass(p.status)"
            >
              {{ p.status }}
            </span>
          </div>
        </div>

        <!-- Card Center Visual & Title -->
        <div class="relative z-20 flex-1 flex flex-col justify-center items-center text-center px-3 py-1 my-auto">
          <!-- Play icon indicator for videos -->
          <div
            v-if="p.format?.toLowerCase().includes('video') || platform === 'tiktok' || platform === 'youtube'"
            class="w-10 h-10 sm:w-11 sm:h-11 rounded-full bg-white/20 backdrop-blur-md border border-white/30 flex items-center justify-center text-white mb-1.5 shadow-md group-hover:scale-110 group-hover:bg-white/30 transition-all duration-300"
          >
            <svg class="w-4 h-4 fill-white translate-x-0.5" viewBox="0 0 24 24"><polygon points="5 3 19 12 5 21 5 3"/></svg>
          </div>

          <span v-if="p.pillar" class="text-[9.5px] uppercase font-bold tracking-widest text-white/70 mb-0.5 drop-shadow-sm">
            {{ p.pillar }}
          </span>

          <h3 class="font-bold text-sm sm:text-base text-white drop-shadow-md line-clamp-2 leading-snug tracking-tight px-1 group-hover:text-accent-wash transition-colors">
            {{ p.topic }}
          </h3>

          <p v-if="p.hook" class="text-[11px] text-white/80 line-clamp-2 italic drop-shadow-sm mt-1 px-1 font-normal">
            “{{ p.hook }}”
          </p>

          <!-- Hover Preview Badge -->
          <div class="mt-2 opacity-0 translate-y-1 group-hover:opacity-100 group-hover:translate-y-0 transition-all duration-200">
            <span class="inline-flex items-center gap-1 px-2.5 py-1 rounded-full bg-white text-ink text-[11px] font-semibold shadow-lg hover:bg-wash transition">
              <span>Preview</span>
              <svg class="w-3 h-3" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5"><path d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6"/><polyline points="15 3 21 3 21 9"/><line x1="10" y1="14" x2="21" y2="3"/></svg>
            </span>
          </div>
        </div>

        <!-- Card Bottom Overlay -->
        <div class="relative z-20 p-2.5 sm:p-3 flex flex-col gap-1.5">
          <!-- Hashtags -->
          <div v-if="p.hashtags?.length" class="flex flex-wrap gap-1">
            <span
              v-for="tag in p.hashtags.slice(0, 3)"
              :key="tag"
              class="text-[9.5px] text-white/75 font-mono"
            >
              #{{ tag }}
            </span>
            <span v-if="p.hashtags.length > 3" class="text-[9.5px] text-white/50">
              +{{ p.hashtags.length - 3 }}
            </span>
          </div>

          <!-- Bottom Row: Simulated Engagements, Date, Dimension -->
          <div class="flex items-center justify-between gap-1.5 pt-1 border-t border-white/10 text-white/90">
            <!-- Social Metrics -->
            <div class="flex items-center gap-2 text-[11px] text-white/80">
              <span class="flex items-center gap-1 hover:text-red-400 transition-colors">
                <svg class="w-3 h-3 fill-current" viewBox="0 0 24 24"><path d="M20.84 4.61a5.5 5.5 0 0 0-7.78 0L12 5.67l-1.06-1.06a5.5 5.5 0 0 0-7.78 7.78l1.06 1.06L12 21.23l7.78-7.78 1.06-1.06a5.5 5.5 0 0 0 0-7.78z"/></svg>
                <span class="font-mono text-[10px]">{{ simLikes(p) }}</span>
              </span>
              <span class="flex items-center gap-1 hover:text-sky-400 transition-colors">
                <svg class="w-3 h-3 fill-current" viewBox="0 0 24 24"><path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z"/></svg>
                <span class="font-mono text-[10px]">{{ simComments(p) }}</span>
              </span>
            </div>

            <!-- Date & Canvas Spec -->
            <div class="flex items-center gap-1.5 shrink-0">
              <span v-if="p.date" class="text-[10px] font-mono font-medium text-white/85">
                {{ p.date }}
              </span>
              <span class="px-1.5 py-0.5 rounded bg-black/60 backdrop-blur-md text-white/90 text-[10px] font-mono font-semibold border border-white/15">
                {{ size.w }}×{{ size.h }}
              </span>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>

  <p v-if="!posts.length" role="status" class="text-muted text-sm mt-3">Nothing scheduled in range.</p>

  <!-- Interactive Social Post Simulator Modal -->
  <div
    v-if="activePost"
    class="fixed inset-0 z-[1000] bg-black/60 backdrop-blur-sm flex items-center justify-center p-4 overflow-y-auto"
    @click.self="closePreview"
  >
    <div
      class="w-full max-w-xl bg-surface border border-line rounded-2xl shadow-2xl overflow-hidden animate-in fade-in zoom-in-95 duration-200"
      role="dialog"
      aria-modal="true"
    >
      <!-- Simulator Modal Header -->
      <div class="flex items-center justify-between px-5 py-3.5 border-b border-line bg-wash/50">
        <div class="flex items-center gap-2">
          <PlatformLogo :id="platform" :size="20" :tile="true" />
          <div>
            <h4 class="font-bold text-sm text-ink leading-tight">{{ platformName }} Post Preview</h4>
            <span class="text-xs text-muted font-mono">{{ size.w }}×{{ size.h }} · {{ size.ratio }}</span>
          </div>
        </div>
        <button
          type="button"
          class="w-8 h-8 rounded-full flex items-center justify-center text-muted hover:text-ink hover:bg-wash transition cursor-pointer"
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
            <div class="w-10 h-10 rounded-full bg-accent/20 border border-accent/30 flex items-center justify-center font-bold text-ink text-sm">
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
          class="w-full relative rounded-xl overflow-hidden border border-line/60 shadow-inner flex items-center justify-center"
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
            class="w-full h-full p-6 flex flex-col justify-between text-white relative"
            :style="{ background: getCardGradient(activePost) }"
          >
            <div class="flex items-center justify-between z-10">
              <span class="px-2.5 py-1 rounded-full bg-black/60 backdrop-blur-md text-xs font-semibold border border-white/10">
                {{ platformName }}
              </span>
              <span class="text-xs font-mono px-2 py-0.5 rounded bg-white/10 backdrop-blur-md">
                {{ size.w }}×{{ size.h }}
              </span>
            </div>

            <div class="flex flex-col items-center justify-center my-auto text-center z-10 px-4">
              <div
                v-if="activePost.format?.toLowerCase().includes('video')"
                class="w-14 h-14 rounded-full bg-white/20 backdrop-blur-md border border-white/30 flex items-center justify-center text-white mb-3 shadow-lg"
              >
                <svg class="w-6 h-6 fill-white translate-x-0.5" viewBox="0 0 24 24"><polygon points="5 3 19 12 5 21 5 3"/></svg>
              </div>
              <h2 class="text-xl sm:text-2xl font-bold tracking-tight mb-2 drop-shadow-md">
                {{ activePost.topic }}
              </h2>
              <p v-if="activePost.hook" class="text-sm text-white/85 italic max-w-md drop-shadow-sm">
                “{{ activePost.hook }}”
              </p>
            </div>

            <div class="flex items-center justify-between text-xs text-white/70 z-10">
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
            <p class="text-ink leading-relaxed font-normal">
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