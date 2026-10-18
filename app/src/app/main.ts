import { createApp } from 'vue'
import './style.css'
import App from './App.vue'
import { VueQueryPlugin, QueryClient } from '@tanstack/vue-query'
import { onSessionChange } from '@/core/session'
import { rewriteDeepLink } from '@/core/deeplink'
import { restoreBrandTheme } from '@/core/theme'

// Paint the remembered brand theme before anything mounts, so the first
// paint already wears the brand instead of flashing the default tokens.
restoreBrandTheme()

const queryClient = new QueryClient({
  defaultOptions: {
    queries: { staleTime: 30_000, retry: 1, refetchOnWindowFocus: false },
    // Mutations are user actions — never replay them silently.
    mutations: { retry: 0 },
  },
})

// Never serve one account's cached data to the next session.
onSessionChange(() => queryClient.clear())

const app = createApp(App)
app.config.errorHandler = (err, _instance, info) => {
  console.error('[content-planner] unhandled error:', info, err)
}

// The hash router pins the URL when its module initialises, so upgrade a bare
// deep link (`/members` without `#`) before importing it.
const deepLink = rewriteDeepLink(window.location)
if (deepLink) window.history.replaceState(null, '', deepLink)

void import('@/app/router').then(async ({ default: router }) => {
  app.use(router).use(VueQueryPlugin, { queryClient })
  // Wait for the initial navigation (including `/` → first-hub redirects)
  // so the first paint already knows the real route — never START_LOCATION.
  // (isReady only settles after the router is installed, hence this order.)
  await router.isReady().catch(() => undefined)
  app.mount('#app')
})
