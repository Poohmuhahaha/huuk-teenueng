<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { useAllPosts, useSetup } from '@/core/queries'
import { MONTHS } from '@/mock/db'
import { t } from '@/core/i18n'
import { activePlatform, FEED_META, platformIdBySetupName } from '@/core/platforms'
import type { FeedPlatformId } from '@/core/platforms'
import { currentRole } from '@/core/auth'
import FeedGrid from '@/components/ui/FeedGrid.vue'
import LiveSection from '@/components/live/LiveSection.vue'
import PlatformLogo from '@/components/ui/PlatformLogo.vue'
import SocialMediaBar from '@/components/ui/SocialMediaBar.vue'
import MonthToggle from '@/components/ui/MonthToggle.vue'
import DockBar from '@/components/ui/DockBar.vue'
import { activeMonth, activeYear } from '@/core/navdate'

const { data: setup } = useSetup()
const platform = ref('')
const viewMode = ref<'grid' | 'phone'>('grid')
/** Once the user picks a platform here, the connections bar stops overriding it. */
const userPicked = ref(false)
const isClient = computed(() => currentRole.value === 'Client')
const upto = ref('')
const variantIndex = ref(-1) // -1 = platform primary feed size
const { data: allPosts, isPending, isError, error: loadError, refetch } = useAllPosts()

// Follow the connections bar until the user picks a platform here.
watch(
  [() => (setup.value?.platforms ?? []).join('|'), activePlatform],
  () => {
    if (userPicked.value) return
    const names = setup.value?.platforms ?? []
    const match = names.find((n) => platformIdBySetupName(n) === activePlatform.value)
    platform.value = match ?? FEED_META[activePlatform.value].name
  },
  { immediate: true },
)

// Default cutoff = end of the configured year; editable.
watch(setup, (s) => {
  if (s && !upto.value) upto.value = `${s.year}-12-31`
}, { immediate: true })

// Changing platform resets the canvas to its primary feed size.
watch(platform, () => {
  variantIndex.value = -1
})

/** Platform names from Set up, with how many planned posts each one has in range. */
const platformTabs = computed(() => {
  const names = setup.value?.platforms ?? []
  const rows = allPosts.value ?? []
  return names.map((name) => ({
    name,
    count: rows.filter(
      (p) => p.date && (!upto.value || p.date <= upto.value) && p.platforms.includes(name),
    ).length,
  }))
})

function choosePlatform(name: string): void {
  userPicked.value = true
  platform.value = name
}

const platformId = computed<FeedPlatformId>(() => platformIdBySetupName(platform.value) ?? 'meta')
const meta = computed(() => FEED_META[platformId.value])
const size = computed(() => {
  const feed = meta.value.feed
  const v = variantIndex.value
  return v >= 0 && v < feed.variants.length ? feed.variants[v] : feed
})

const preview = computed(() =>
  (allPosts.value ?? [])
    .filter((p) => p.date && (!upto.value || p.date <= upto.value) && p.platforms.includes(platform.value))
    .sort((a, b) => (b.date ?? '').localeCompare(a.date ?? ''))
    .slice(0, 9),
)

const monthLabel = computed(() => {
  const m = Number(upto.value.slice(5, 7))
  return MONTHS.includes(m) ? m : null
})

function goMonth(m: number | string): void {
  activeMonth.value = Number(m)
  const lastDay = new Date(activeYear.value, Number(m), 0).getDate()
  upto.value = `${activeYear.value}-${String(m).padStart(2, '0')}-${String(lastDay).padStart(2, '0')}`
}

function setYear(y: number): void {
  activeYear.value = y
  const m = activeMonth.value
  const lastDay = new Date(y, m, 0).getDate()
  upto.value = `${y}-${String(m).padStart(2, '0')}-${String(lastDay).padStart(2, '0')}`
}
</script>

<template>
  <div class="feedpage text-left w-full flex flex-col gap-5">
    <DockBar label="Feed controls">
      <div class="flex items-center justify-between w-full gap-3">
        <div class="flex items-center gap-2">
          <MonthToggle :model-value="activeMonth" :year="activeYear" @update:model-value="goMonth" @update:year="setYear" />
          <span class="dock-sep" aria-hidden="true" />
          <span class="text-xs font-semibold text-muted tracking-wider uppercase font-mono">
            {{ preview.length }} Posts
          </span>
        </div>
        <div class="flex items-center gap-2 ml-auto">
          <RouterLink class="btn btn-icon w-auto" to="/analyze?tab=performance" title="Import metrics" aria-label="Import metrics">
            <svg viewBox="0 0 16 16" width="16" height="16" fill="none" stroke="currentColor"
              stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
              <path d="M8 2.5v7.5m0 0L5.2 7.2M8 10l2.8-2.8M3 13.5h10" />
            </svg>
          </RouterLink>
          <RouterLink class="btn btn-primary btn-icon w-auto" to="/plan?tab=monthly" title="Add new topic" aria-label="Add new topic">+</RouterLink>
        </div>
      </div>
    </DockBar>

    <div v-if="isError" class="card" role="alert">
      <p class="muted">Could not load posts: {{ loadError?.message }}</p>
      <button class="btn" @click="() => refetch()">{{ t('common.tryAgain') }}</button>
    </div>

    <!-- Section Header with Post Count Badge & View Toggle (Swiss CI) -->
    <div v-if="setup" class="flex flex-col gap-3">
      <div class="flex items-center justify-between flex-wrap gap-2.5">
        <div class="flex items-center gap-2 flex-wrap">
          <h2 class="text-xs font-bold uppercase tracking-widest text-muted m-0">Feed Preview</h2>
          <span class="inline-flex items-center px-2 py-0.5 rounded-full text-[11px] font-semibold bg-black text-white font-mono">
            {{ preview.length }} posts
          </span>
          <span class="text-xs text-muted font-mono">
            {{ t('feed.canvas') }} {{ size.w }} × {{ size.h }} px · {{ size.ratio }} · {{ meta.name }}
            <template v-if="monthLabel">· {{ t('feed.uptoMonth') }} {{ monthLabel }}</template>
          </span>
        </div>

        <!-- View Switcher -->
        <div class="inline-flex items-center p-0.5 rounded-lg bg-wash border border-line">
          <button
            type="button"
            class="px-2.5 py-1 rounded-md text-xs font-semibold transition cursor-pointer flex items-center gap-1.5"
            :class="viewMode === 'grid' ? 'bg-surface text-ink shadow-2xs' : 'text-muted hover:text-ink'"
            @click="viewMode = 'grid'"
          >
            <svg class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="3" y="3" width="7" height="7"/><rect x="14" y="3" width="7" height="7"/><rect x="14" y="14" width="7" height="7"/><rect x="3" y="14" width="7" height="7"/></svg>
            <span>Canvas Board</span>
          </button>

          <button
            type="button"
            class="px-2.5 py-1 rounded-md text-xs font-semibold transition cursor-pointer flex items-center gap-1.5"
            :class="viewMode === 'phone' ? 'bg-surface text-ink shadow-2xs' : 'text-muted hover:text-ink'"
            @click="viewMode = 'phone'"
          >
            <svg class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="5" y="2" width="14" height="20" rx="3"/><line x1="12" y1="18" x2="12.01" y2="18"/></svg>
            <span>Mobile Simulator</span>
          </button>
        </div>
      </div>

      <!-- Unified Toolbar Card: Platform Tabs + Settings -->
      <div class="p-3 sm:p-3.5 bg-surface border border-line rounded-xl shadow-2xs flex flex-wrap items-center justify-between gap-3">
        <!-- Platform switcher tabs -->
        <div class="flex items-center flex-wrap gap-1.5" role="tablist" :aria-label="t('feed.platform')">
          <button v-for="tab in platformTabs" :key="tab.name" type="button"
            class="feed-tab inline-flex items-center gap-2 px-3 py-1.5 border rounded-full text-xs font-semibold cursor-pointer transition shadow-2xs"
            :class="platform === tab.name
              ? 'active bg-ink border-ink text-surface'
              : 'bg-surface border-line text-muted hover:bg-wash hover:text-ink'"
            role="tab" :aria-selected="platform === tab.name"
            @click="choosePlatform(tab.name)">
            <PlatformLogo :name="tab.name" :size="14" />
            <span>{{ tab.name }}</span>
            <span class="feed-tab-count px-1.5 py-0.2 rounded-full text-[10px] font-bold"
              :class="platform === tab.name ? 'bg-white/20 text-surface' : 'bg-wash text-muted'">
              {{ tab.count }}
            </span>
          </button>

          <!-- Manage Channels Button (Opens Popup Modal) -->
          <SocialMediaBar v-if="!isClient" :popup="true" />
        </div>

        <!-- Filters: Date & Size -->
        <div class="flex flex-wrap items-center gap-3">
          <div class="flex items-center gap-1.5">
            <label class="lbl text-[11px] font-semibold text-muted uppercase tracking-wider whitespace-nowrap" for="feed-upto">
              {{ t('feed.upto') }}
            </label>
            <input id="feed-upto" class="field text-xs py-1 px-2.5 h-8 w-auto rounded-lg" type="date" v-model="upto" />
          </div>

          <div class="flex items-center gap-1.5">
            <label class="lbl text-[11px] font-semibold text-muted uppercase tracking-wider whitespace-nowrap" for="feed-size">
              {{ t('feed.size') }}
            </label>
            <select id="feed-size" class="field text-xs py-1 px-2.5 h-8 w-auto rounded-lg" v-model="variantIndex">
              <option :value="-1">{{ meta.feed.w }}×{{ meta.feed.h }} · {{ meta.feed.ratio }}</option>
              <option v-for="(v, i) in meta.feed.variants" :key="i" :value="i">
                {{ v.label }} · {{ v.w }}×{{ v.h }} · {{ v.ratio }}
              </option>
            </select>
          </div>
        </div>
      </div>
    </div>

    <!-- Feed Content -->
    <div v-if="!isPending">
      <!-- Canvas Grid View -->
      <div v-show="viewMode === 'grid'">
        <FeedGrid :posts="preview" :platform="platformId" :variant="variantIndex" />
      </div>

      <!-- Phone Simulator View -->
      <div v-show="viewMode === 'phone'" class="py-4 flex flex-col items-center justify-center">
        <div class="w-[360px] sm:w-[380px] h-[720px] bg-surface rounded-[44px] border-[10px] border-neutral-950 shadow-[0_25px_60px_-15px_rgba(0,0,0,0.3)] ring-1 ring-white/15 overflow-hidden flex flex-col relative select-none">
          <!-- Top Status Bar & Dynamic Island -->
          <div class="pt-2 px-6 flex items-center justify-between text-[11px] font-semibold text-ink shrink-0 z-30">
            <span>9:41</span>
            <div class="w-24 h-5 rounded-full bg-black flex items-center justify-end px-2">
              <div class="w-2.5 h-2.5 rounded-full bg-emerald-500/80" />
            </div>
            <div class="flex items-center gap-1.5 text-xs">
              <span>5G</span>
              <svg class="w-3 h-3 fill-current" viewBox="0 0 24 24"><path d="M12 3c-4.97 0-9 4.03-9 9 0 2.12.74 4.07 1.97 5.61L4.35 19.4c-.39.39-.39 1.02 0 1.41.39.39 1.02.39 1.41 0l1.9-1.9C9.28 19.59 10.59 20 12 20c4.97 0 9-4.03 9-9s-4.03-9-9-9z"/></svg>
              <div class="w-5 h-2.5 rounded-sm border border-current p-0.5 flex items-center"><div class="w-full h-full bg-current rounded-2xs" /></div>
            </div>
          </div>

          <!-- Native App Header -->
          <div class="px-4 py-2.5 border-b border-line flex items-center justify-between shrink-0 bg-surface/95 backdrop-blur-md z-20">
            <div class="flex items-center gap-2">
              <PlatformLogo :id="platformId" :size="20" :tile="true" />
              <span class="font-bold text-sm text-ink tracking-tight">{{ meta.name }}</span>
            </div>
            <div class="flex items-center gap-3 text-muted">
              <svg class="w-5 h-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M20.84 4.61a5.5 5.5 0 0 0-7.78 0L12 5.67l-1.06-1.06a5.5 5.5 0 0 0-7.78 7.78l1.06 1.06L12 21.23l7.78-7.78 1.06-1.06a5.5 5.5 0 0 0 0-7.78z"/></svg>
              <svg class="w-5 h-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M21 11.5a8.38 8.38 0 0 1-.9 3.8 8.5 8.5 0 0 1-7.6 4.7 8.38 8.38 0 0 1-3.8-.9L3 21l1.9-5.7a8.38 8.38 0 0 1-.9-3.8 8.5 8.5 0 0 1 4.7-7.6 8.38 8.38 0 0 1 3.8-.9h.5a8.48 8.48 0 0 1 8 8v.5z"/></svg>
            </div>
          </div>

          <!-- Scrollable Mobile Feed Body -->
          <div class="flex-1 overflow-y-auto divide-y divide-line/60 bg-surface">
            <div v-if="!preview.length" class="p-8 text-center text-muted text-sm">
              Nothing scheduled in range.
            </div>
            <article v-for="p in preview" :key="p.id" class="p-3.5 flex flex-col gap-2.5">
              <!-- Author Row -->
              <div class="flex items-center justify-between">
                <div class="flex items-center gap-2.5">
                  <div class="w-8 h-8 rounded-full bg-accent/20 border border-accent/40 flex items-center justify-center font-bold text-xs text-ink">
                    {{ (p.topic || 'S').slice(0, 1).toUpperCase() }}
                  </div>
                  <div>
                    <div class="font-bold text-xs text-ink leading-tight">Studio Channel</div>
                    <div class="text-[10px] text-muted">{{ p.date || 'Upcoming' }} · {{ p.time || '12:00' }}</div>
                  </div>
                </div>
                <span class="text-xs text-muted">•••</span>
              </div>

              <!-- Post Media Visual -->
              <div class="w-full rounded-xl overflow-hidden border border-line shadow-xs relative bg-wash" :style="{ aspectRatio: `${size.w} / ${size.h}` }">
                <img v-if="p.imageUrl" :src="p.imageUrl" :alt="p.topic" class="w-full h-full object-cover" />
                <div v-else class="w-full h-full p-4 flex flex-col justify-between text-ink bg-wash/50">
                  <span class="text-[10px] font-mono px-2 py-0.5 rounded bg-surface border border-line self-start">{{ size.w }}×{{ size.h }}</span>
                  <div class="text-center my-auto px-2">
                    <h4 class="font-bold text-sm leading-snug text-ink m-0">{{ p.topic }}</h4>
                    <p v-if="p.hook" class="text-xs text-muted italic mt-1 line-clamp-2 m-0">“{{ p.hook }}”</p>
                  </div>
                  <div class="flex items-center justify-between text-[10px] text-muted">
                    <span>{{ p.pillar }}</span>
                    <span>{{ p.format }}</span>
                  </div>
                </div>
              </div>

              <!-- Social Action Row -->
              <div class="flex items-center justify-between pt-0.5">
                <div class="flex items-center gap-3 text-ink">
                  <svg class="w-4 h-4 fill-none stroke-current" stroke-width="2" viewBox="0 0 24 24"><path d="M20.84 4.61a5.5 5.5 0 0 0-7.78 0L12 5.67l-1.06-1.06a5.5 5.5 0 0 0-7.78 7.78l1.06 1.06L12 21.23l7.78-7.78 1.06-1.06a5.5 5.5 0 0 0 0-7.78z"/></svg>
                  <svg class="w-4 h-4 fill-none stroke-current" stroke-width="2" viewBox="0 0 24 24"><path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z"/></svg>
                  <svg class="w-4 h-4 fill-none stroke-current" stroke-width="2" viewBox="0 0 24 24"><circle cx="18" cy="5" r="3"/><circle cx="6" cy="12" r="3"/><circle cx="18" cy="19" r="3"/><line x1="8.59" y1="13.51" x2="15.42" y2="17.49"/><line x1="15.41" y1="6.51" x2="8.59" y2="10.49"/></svg>
                </div>
                <svg class="w-4 h-4 fill-none stroke-current text-ink" stroke-width="2" viewBox="0 0 24 24"><path d="M19 21l-7-5-7 5V5a2 2 0 0 1 2-2h10a2 2 0 0 1 2 2z"/></svg>
              </div>

              <!-- Caption -->
              <div class="text-xs text-ink leading-relaxed">
                <span class="font-bold mr-1">Studio Channel</span>
                <span>{{ p.caption || p.hook || p.topic }}</span>
                <div v-if="p.hashtags?.length" class="flex flex-wrap gap-1 mt-1 text-accent font-medium">
                  <span v-for="tag in p.hashtags" :key="tag">#{{ tag }}</span>
                </div>
              </div>
            </article>
          </div>

          <!-- Native Bottom Dock Bar -->
          <div class="px-6 py-3 border-t border-line bg-surface flex items-center justify-between text-muted shrink-0 z-20">
            <svg class="w-5 h-5 text-ink" viewBox="0 0 24 24" fill="currentColor"><path d="M10 20v-6h4v6h5v-8h3L12 3 2 12h3v8z"/></svg>
            <svg class="w-5 h-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="11" cy="11" r="8"/><line x1="21" y1="21" x2="16.65" y2="16.65"/></svg>
            <div class="w-6 h-6 rounded-lg border-2 border-current flex items-center justify-center font-bold text-xs">+</div>
            <svg class="w-5 h-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polygon points="12 2 15.09 8.26 22 9.27 17 14.14 18.18 21.02 12 17.77 5.82 21.02 7 14.14 2 9.27 8.91 8.26 12 2"/></svg>
            <div class="w-5 h-5 rounded-full border-2 border-current" />
          </div>

          <!-- Home Indicator Bar -->
          <div class="pb-1 pt-0.5 flex justify-center bg-surface shrink-0">
            <div class="w-32 h-1 bg-ink/40 rounded-full" />
          </div>
        </div>
      </div>
    </div>

    <!-- Live Mirror Section -->
    <LiveSection variant="grid" :limit="6" />
  </div>
</template>