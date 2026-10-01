import { createRouter, createWebHashHistory } from 'vue-router'
import type { RouteRecordRaw } from 'vue-router'
import { screens } from '@/core/screens'
import { currentUser } from '@/core/auth'
import NotFoundPage from '@/pages/NotFoundPage.vue'
import ReadPage from '@/pages/ReadPage.vue'
import LoginPage from '@/pages/LoginPage.vue'
import RegisterPage from '@/pages/RegisterPage.vue'
import PlansPage from '@/pages/PlansPage.vue'
import WorkspaceSetupPage from '@/pages/WorkspaceSetupPage.vue'

// One sheet, one route — routes derived from the slide deck registry.
// Profile & setup is not a deck card: it opens as a popup from the avatar, and
// `/profile` + `/setup` are kept as aliases for links and the OAuth callback
// (both preserve query params so the callback notice still lands).
const routes: RouteRecordRaw[] = [
  { path: '/', redirect: (to) => ({ path: screens[0].to, query: to.query }) },
  { path: '/profile', redirect: (to) => ({ path: '/', query: to.query }) },
  { path: '/setup', redirect: (to) => ({ path: '/', query: to.query }) },
  ...screens.map((s) => ({ path: s.path, component: s.component, props: true })),
  // Monthly planner without a month in the URL opens the current month.
  { path: '/planner', component: () => import('@/pages/PlannerPage.vue'), meta: { standalone: false } },
  // SaaS onboarding: login, signup and packages (no app shell).
  { path: '/login', component: LoginPage, meta: { public: true } },
  { path: '/register', component: RegisterPage, meta: { public: true } },
  { path: '/plans', component: PlansPage, meta: { public: true } },
  // First-run workspace creation (App.vue renders it full-screen while the
  // signed-in account has no workspace yet).
  { path: '/welcome', component: WorkspaceSetupPage, meta: { standalone: true } },
  // Public published-content reader (no app shell).
  { path: '/read', component: ReadPage, meta: { standalone: true, public: true } },
  { path: '/read/:slug', component: ReadPage, meta: { standalone: true, public: true } },
  { path: '/:pathMatch(.*)*', component: NotFoundPage },
]

const router = createRouter({ history: createWebHashHistory(), routes })

// Signed-in users skip the onboarding pages (but may revisit /plans).
router.beforeEach((to) => {
  if (currentUser.value && (to.path === '/login' || to.path === '/register')) return '/'
  return true
})

export default router
