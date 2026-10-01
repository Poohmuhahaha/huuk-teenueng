import { createApp } from 'vue'
import './style.css'
import App from './App.vue'
import { VueQueryPlugin, QueryClient } from '@tanstack/vue-query'
import { onSessionChange } from '@/core/session'
import { rewriteDeepLink } from '@/core/deeplink'

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

void import('@/app/router').then(({ default: router }) => {
  app.use(router).use(VueQueryPlugin, { queryClient }).mount('#app')
})
