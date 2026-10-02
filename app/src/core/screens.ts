// Screen registry — the compact slide deck: 6 hub screens (user journey).
// One sheet, one screen; routes and the deck both derive from this list.
// Related features are grouped as tabs inside a hub instead of separate cards.
// Note: the Guide and Workspace settings are overlays, not deck cards.
import type { Component } from 'vue'
import DashboardPage from '@/pages/DashboardPage.vue'
import PlanPage from '@/pages/PlanPage.vue'
import ContentPage from '@/pages/ContentPage.vue'
import PromotePage from '@/pages/PromotePage.vue'
import AnalyzePage from '@/pages/AnalyzePage.vue'
import SettingsPage from '@/pages/SettingsPage.vue'

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
  { path: '/dashboard', to: '/dashboard', label: 'Home', match: ['/dashboard', '/home'], component: DashboardPage, sheet: 'Home', access: 'computed' },
  { path: '/plan', to: '/plan', label: 'Plan', match: ['/plan'], component: PlanPage, sheet: 'Plan', access: 'WRITE master' },
  { path: '/content', to: '/content', label: 'Content', match: ['/content'], component: ContentPage, sheet: 'Content', access: 'WRITE master' },
  { path: '/promote', to: '/promote', label: 'Promote', match: ['/promote'], component: PromotePage, sheet: 'Promote', access: 'WRITE master' },
  { path: '/analyze', to: '/analyze', label: 'Analyze', match: ['/analyze'], component: AnalyzePage, sheet: 'Analyze', access: 'semi-computed' },
  { path: '/settings', to: '/settings', label: 'Settings', match: ['/settings'], component: SettingsPage, sheet: 'Settings', access: 'owner' },
]

export function screenIndexOf(path: string): number {
  const i = screens.findIndex((s) => s.match.some((m) => path.startsWith(m)))
  return i < 0 ? 0 : i
}

export function screenOf(path: string): Screen | undefined {
  return screens.find((s) => s.match.some((m) => path.startsWith(m)))
}
