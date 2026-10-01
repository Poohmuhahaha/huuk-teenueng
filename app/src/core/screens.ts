// Screen registry — the slide deck order (user journey J1–J10).
// One sheet, one screen; routes and the deck both derive from this list.
// Note: the Guide is not a deck card — it opens as an overlay from the navbar.
import type { Component } from 'vue'
import BrandPage from '@/pages/BrandPage.vue'
import PlannerPage from '@/pages/PlannerPage.vue'
import CalendarPage from '@/pages/CalendarPage.vue'
import FeedPage from '@/pages/FeedPage.vue'
import DashboardPage from '@/pages/DashboardPage.vue'
import PerformancePage from '@/pages/PerformancePage.vue'
import IdeasPage from '@/pages/IdeasPage.vue'
import HashtagsPage from '@/pages/HashtagsPage.vue'
import FinancePage from '@/pages/FinancePage.vue'
import LivePage from '@/pages/LivePage.vue'
import CampaignsPage from '@/pages/CampaignsPage.vue'
import AdsPage from '@/pages/AdsPage.vue'
import MembersPage from '@/pages/MembersPage.vue'
import StudioPage from '@/pages/StudioPage.vue'

export interface Screen {
  path: string        // route path (may contain :param)
  to: string          // link target
  label: string
  match: string[]     // path prefixes that resolve to this screen
  component: Component
  sheet: string       // owning sheet (right rail)
  access: string      // read/write note (right rail)
}

export const screens: Screen[] = [
  { path: '/brand', to: '/brand', label: 'Brand', match: ['/brand'], component: BrandPage, sheet: 'Brand Identity', access: 'standalone' },
  { path: '/planner/:month', to: '/planner', label: 'Monthly 01–12', match: ['/planner'], component: PlannerPage, sheet: '01–12', access: 'WRITE master' },
  { path: '/calendar', to: '/calendar', label: 'Calendar', match: ['/calendar'], component: CalendarPage, sheet: 'Smart Calendar', access: 'computed' },
  { path: '/feed', to: '/feed', label: 'Feed', match: ['/feed'], component: FeedPage, sheet: 'Feed Review', access: 'computed' },
  { path: '/dashboard', to: '/dashboard', label: 'Dashboard', match: ['/dashboard'], component: DashboardPage, sheet: 'Dashboard', access: 'computed' },
  { path: '/performance', to: '/performance', label: 'Performance', match: ['/performance'], component: PerformancePage, sheet: 'Performance', access: 'semi-computed' },
  { path: '/ideas', to: '/ideas', label: 'Ideas', match: ['/ideas'], component: IdeasPage, sheet: 'Content idea Bank', access: 'standalone' },
  { path: '/hashtags', to: '/hashtags', label: 'Hashtags', match: ['/hashtags'], component: HashtagsPage, sheet: 'Hashtag #', access: 'standalone' },
  { path: '/finance', to: '/finance', label: 'Finance', match: ['/finance'], component: FinancePage, sheet: 'Finance', access: 'standalone' },
  // Former standalone pages — each is a deck card now (wide content uses the
  // card's "Go full page" toggle). Members stays owner-only via its page +
  // nav link; Studio doubles as the Client's only card (deck filters by role).
  { path: '/live', to: '/live', label: 'Live', match: ['/live'], component: LivePage, sheet: 'Live Mirror', access: 'computed' },
  { path: '/campaigns/:id?', to: '/campaigns', label: 'Campaigns', match: ['/campaigns'], component: CampaignsPage, sheet: 'Campaigns', access: 'WRITE master' },
  { path: '/ads', to: '/ads', label: 'Ads', match: ['/ads'], component: AdsPage, sheet: 'Meta Ads', access: 'semi-computed' },
  { path: '/members', to: '/members', label: 'Members', match: ['/members'], component: MembersPage, sheet: 'Members', access: 'owner only' },
  { path: '/studio/:id?', to: '/studio', label: 'Content', match: ['/studio'], component: StudioPage, sheet: 'Content Studio', access: 'WRITE master' },
]

export function screenIndexOf(path: string): number {
  const i = screens.findIndex((s) => s.match.some((m) => path.startsWith(m)))
  return i < 0 ? 0 : i
}

export function screenOf(path: string): Screen | undefined {
  return screens.find((s) => s.match.some((m) => path.startsWith(m)))
}
