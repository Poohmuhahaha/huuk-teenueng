<script setup lang="ts">
import { computed } from 'vue'
import type { Post } from '@/mock/db'

const props = defineProps<{
  year: number
  month: number
  posts: Post[]
  weekStart: 0 | 1
  showPillar: boolean
  showPlatform: boolean
  showStatus: boolean
}>()

interface DayCell {
  day: number | null
  posts: Post[]
}

const DOW = ['S', 'M', 'T', 'W', 'T', 'F', 'S']

const orderedDow = computed(() =>
  props.weekStart === 1 ? [...DOW.slice(1), DOW[0]] : DOW,
)

const cells = computed<DayCell[]>(() => {
  const first = new Date(props.year, props.month - 1, 1)
  const lead = (first.getDay() - props.weekStart + 7) % 7
  const days = new Date(props.year, props.month, 0).getDate()
  const prefix = `${props.year}-${String(props.month).padStart(2, '0')}-`
  const out: DayCell[] = []
  for (let i = 0; i < lead; i++) out.push({ day: null, posts: [] })
  for (let d = 1; d <= days; d++) {
    const key = prefix + String(d).padStart(2, '0')
    out.push({ day: d, posts: props.posts.filter((p) => p.date === key) })
  }
  while (out.length % 7 !== 0) out.push({ day: null, posts: [] })
  return out
})
</script>

<template>
  <div class="cal">
    <div v-for="(d, i) in orderedDow" :key="i" class="dow">{{ d }}</div>
    <div v-for="(c, i) in cells" :key="i" class="day" :class="{ empty: c.day === null }">
      <template v-if="c.day !== null">
        <div class="daynum">{{ c.day }}</div>
        <div>
          <span v-for="p in c.posts.slice(0, 4)" :key="p.id" class="dot" :class="{ open: p.status !== 'Done' }" :title="p.topic" />
        </div>
        <div v-if="c.posts.length > 4" class="muted">+{{ c.posts.length - 4 }}</div>
        <div v-if="showPillar && c.posts.length" class="muted">{{ c.posts[0].pillar }}</div>
        <div v-if="showPlatform && c.posts.length" class="muted">{{ c.posts[0].platforms.join(' ') }}</div>
        <div v-if="showStatus && c.posts.length" class="muted">{{ c.posts[0].status }}</div>
      </template>
    </div>
  </div>
</template>
