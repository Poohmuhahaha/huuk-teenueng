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
import SocialMediaBar from '@/components/ui/SocialMediaBar.vue'

const { data: setup } = useSetup()
const platform = ref('')
/** Once the user picks a platform here, the connections bar stops overriding it. */
const userPicked = ref(false)
const isClient = computed(() => currentRole.value === 'Client')
const upto = ref('')
const variantIndex = ref(-1) // -1 = platform primary feed size
const { data: allPosts, isPending, isError, error: loadError, refetch } = useAllPosts()

// Follow the connections bar until the user picks a platform here.
// The setup query refetches on window focus and returns a fresh object, so the
// watch keys on the platform *names* — a refetch must not reset the choice.
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
</script>

<template>
  <h1 class="text-2xl font-bold mb-4">{{ t('feed.title') }}</h1>
  <SocialMediaBar v-if="!isClient" />
  <div v-if="isError" class="card">
    <p class="muted">Could not load posts: {{ loadError?.message }}</p>
    <button class="btn" @click="() => refetch()">Try again</button>
  </div>
  <div v-if="setup" class="mb-2.5">
    <div class="row flex-wrap justify-center gap-1.5" role="tablist" :aria-label="t('feed.platform')">
      <button v-for="tab in platformTabs" :key="tab.name" type="button"
        class="feed-tab inline-flex items-center gap-1.5 px-3 py-1.5 border rounded-full text-[13.5px] font-semibold cursor-pointer transition"
        :class="platform === tab.name
          ? 'active bg-accent border-accent text-accent-ink'
          : 'bg-surface border-line text-muted hover:bg-wash hover:text-ink'"
        role="tab" :aria-selected="platform === tab.name"
        @click="choosePlatform(tab.name)">
        {{ tab.name }}
        <span class="feed-tab-count px-1.5 rounded-full text-[11.5px] font-bold"
          :class="platform === tab.name ? 'bg-white/25 text-inherit' : 'bg-wash text-muted'">
          {{ tab.count }}
        </span>
      </button>
    </div>
    <div class="grid2 max-w-[620px] mt-2">
      <div><label class="lbl" for="feed-upto">{{ t('feed.upto') }}</label>
        <input id="feed-upto" class="field" type="date" v-model="upto" /></div>
      <div><label class="lbl" for="feed-size">{{ t('feed.size') }}</label>
        <select id="feed-size" class="field" v-model="variantIndex">
          <option :value="-1">{{ meta.feed.w }}×{{ meta.feed.h }} · {{ meta.feed.ratio }}</option>
          <option v-for="(v, i) in meta.feed.variants" :key="i" :value="i">
            {{ v.label }} · {{ v.w }}×{{ v.h }} · {{ v.ratio }}
          </option>
        </select></div>
    </div>
  </div>
  <p class="muted">{{ t('feed.canvas') }} {{ size.w }} × {{ size.h }} px · {{ size.ratio }} · {{ meta.name }}
    <span v-if="monthLabel"> · {{ t('feed.uptoMonth') }} {{ monthLabel }}</span>
    <span> · {{ preview.length }} {{ t('feed.posts') }}</span></p>
  <div class="mt-4" v-if="!isPending"><FeedGrid :posts="preview" :platform="platformId" :variant="variantIndex" /></div>
  <LiveSection variant="grid" :limit="6" />
</template>
