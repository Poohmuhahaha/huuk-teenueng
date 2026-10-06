// Screen registry — the compact slide deck: 6 hub screens (user journey).
// One sheet, one screen; routes and the deck both derive from this list.
// Related features are grouped as tabs inside a hub instead of separate cards
// (tab metadata lives in @/core/subnav so pages and the subnavbar share it).
// Note: the Guide and Workspace settings are overlays, not deck cards.
import type { Component } from 'vue'
import DashboardPage from '@/pages/DashboardPage.vue'
import PlanPage from '@/pages/PlanPage.vue'
import ContentPage from '@/pages/ContentPage.vue'
import PromotePage from '@/pages/PromotePage.vue'
import AnalyzePage from '@/pages/AnalyzePage.vue'
import SettingsPage from '@/pages/SettingsPage.vue'
import { HUB_TABS } from '@/core/subnav'
import type { HubTab } from '@/core/subnav'

export interface Screen {
  path: string        // route path (may contain :param)
  to: string          // link target
  label: string
  match: string[]     // path prefixes that resolve to this screen
  component: Component
  sheet: string       // owning sheet (right rail)
  access: string      // read/write note (right rail)
  tabs?: HubTab[]     // hub sections shown in the subnavbar (?tab=id)
}

export const screens: Screen[] = [
  // Brand Identity leads the deck: it is the workspace's foundation, so it is
  // the front-most card (and the landing screen at "/").
  { path: '/settings', to: '/settings', label: 'Settings', match: ['/settings'], component: SettingsPage, sheet: 'Settings', access: 'owner', tabs: HUB_TABS['/settings'] },
  { path: '/dashboard', to: '/dashboard', label: 'Home', match: ['/dashboard', '/home'], component: DashboardPage, sheet: 'Home', access: 'computed', tabs: HUB_TABS['/dashboard'] },
  { path: '/plan', to: '/plan', label: 'Plan', match: ['/plan'], component: PlanPage, sheet: 'Plan', access: 'WRITE master', tabs: HUB_TABS['/plan'] },
  { path: '/content', to: '/content', label: 'Content', match: ['/content'], component: ContentPage, sheet: 'Content', access: 'WRITE master', tabs: HUB_TABS['/content'] },
  { path: '/promote', to: '/promote', label: 'Promote', match: ['/promote'], component: PromotePage, sheet: 'Promote', access: 'WRITE master', tabs: HUB_TABS['/promote'] },
  { path: '/analyze', to: '/analyze', label: 'Analyze', match: ['/analyze'], component: AnalyzePage, sheet: 'Analyze', access: 'semi-computed', tabs: HUB_TABS['/analyze'] },
]

export function screenIndexOf(path: string): number {
  const i = screens.findIndex((s) => s.match.some((m) => path.startsWith(m)))
  return i < 0 ? 0 : i
}

export function screenOf(path: string): Screen | undefined {
  return screens.find((s) => s.match.some((m) => path.startsWith(m)))
}
