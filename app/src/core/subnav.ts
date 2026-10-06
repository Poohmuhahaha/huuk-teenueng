// Hub sub-navigation: tab metadata (single source of truth) plus the hover
// preview state used by the subnavbar. Pages read the active tab from here —
// no per-page tab lists.
import { computed, ref } from 'vue'
import { useRoute } from 'vue-router'

export interface HubTab {
  id: string
  label: string
}

export const HUB_TABS: Record<string, HubTab[]> = {
  '/dashboard': [
    { id: 'overview', label: 'Overview' },
    { id: 'live', label: 'Live' },
    { id: 'ads', label: 'Ads' },
    { id: 'top', label: 'Top posts' },
  ],
  '/plan': [
    { id: 'monthly', label: 'Monthly' },
    { id: 'calendar', label: 'Calendar' },
    { id: 'ideas', label: 'Ideas' },
    { id: 'hashtags', label: 'Hashtags' },
  ],
  '/content': [
    { id: 'studio', label: 'Studio' },
    { id: 'feed', label: 'Feed preview' },
  ],
  '/promote': [
    { id: 'campaigns', label: 'Campaigns' },
    { id: 'ads', label: 'Meta Ads' },
  ],
  '/analyze': [
    { id: 'performance', label: 'Performance' },
    { id: 'finance', label: 'Finance' },
  ],
  '/settings': [
    { id: 'brand', label: 'Brand' },
    { id: 'workspace', label: 'Workspace' },
    { id: 'members', label: 'Members' },
    { id: 'connections', label: 'Connections' },
  ],
}

export function tabsForPath(path: string): HubTab[] {
  const key = Object.keys(HUB_TABS).find((p) => path === p || path.startsWith(p + '/'))
  return key ? HUB_TABS[key] : []
}

// Active hub tab for the page's own hub. The deck mounts every hub card at
// once, so only the card matching the current route honours ?tab=; the others
// fall back to their first tab.
export function useHubTab(path: string) {
  const route = useRoute()
  const tabs = computed(() => HUB_TABS[path] ?? [])
  const active = computed<string>(() => {
    const onThisHub = route.path === path || route.path.startsWith(path + '/')
    const q = route.query.tab
    if (onThisHub && typeof q === 'string' && tabs.value.some((t) => t.id === q)) return q
    return tabs.value[0]?.id ?? ''
  })
  return { tabs, active }
}

// Subnavbar hover preview; `null` = follow the route.
export const previewedScreen = ref<string | null>(null)

export function previewScreen(path: string | null): void {
  previewedScreen.value = path
}
