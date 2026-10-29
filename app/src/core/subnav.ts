// Hub sub-navigation: tab metadata (single source of truth). Pages read the
// active tab from here — no per-page tab lists. The subnavbar follows the
// route only: nothing previews on hover, a hub's tabs appear when it is
// actually opened.
import { computed, reactive, watch } from 'vue'
import { useRoute } from 'vue-router'

export interface HubTab {
  id: string
  label: string
}

export const HUB_TABS: Record<string, HubTab[]> = {
  '/dashboard': [
    { id: 'overview', label: 'Overview' },
    { id: 'ads', label: 'Ads' },
    { id: 'top', label: 'Top posts' },
    { id: 'performance', label: 'Performance' },
    { id: 'finance', label: 'Finance' },
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
  '/branding': [
    { id: 'brand', label: 'Brand' },
    { id: 'members', label: 'Members' },
    { id: 'connections', label: 'Connections' },
  ],
  '/settings': [
    { id: 'brand', label: 'Brand' },
    { id: 'members', label: 'Members' },
    { id: 'connections', label: 'Connections' },
  ],
}

// Store the active tab for each hub so sliding between cards in the deck
// retains each card's selected tab instead of snapping back to the first tab.
const hubLastTab = reactive<Record<string, string>>({})

export function getRememberedTab(path: string): string | undefined {
  return hubLastTab[path]
}

export function setRememberedTab(path: string, tab: string): void {
  hubLastTab[path] = tab
  if (path === '/branding') hubLastTab['/settings'] = tab
  if (path === '/settings') hubLastTab['/branding'] = tab
}

export function tabsForPath(path: string): HubTab[] {
  const key = Object.keys(HUB_TABS).find((p) => path === p || path.startsWith(p + '/'))
  return key ? HUB_TABS[key] : []
}

// Active hub tab for the page's own hub. The deck mounts every hub card at
// once, and each card retains its active tab when sliding across the deck.
export function useHubTab(path: string) {
  const route = useRoute()
  const tabs = computed(() => HUB_TABS[path] ?? [])

  // Sync route.query.tab into remembered tab whenever the user is on this hub
  watch(
    () => [route.path, route.query.tab],
    ([p, q]) => {
      const onThisHub = p === path || (typeof p === 'string' && p.startsWith(path + '/'))
      if (onThisHub && typeof q === 'string' && tabs.value.some((t) => t.id === q)) {
        setRememberedTab(path, q)
      }
    },
    { immediate: true },
  )

  const active = computed<string>(() => {
    const onThisHub = route.path === path || route.path.startsWith(path + '/')
    const q = route.query.tab
    if (onThisHub && typeof q === 'string' && tabs.value.some((t) => t.id === q)) {
      return q
    }
    return getRememberedTab(path) || tabs.value[0]?.id || ''
  })
  return { tabs, active }
}
