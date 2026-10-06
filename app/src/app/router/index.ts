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
import ComponentsPage from '@/pages/ComponentsPage.vue'

// One sheet, one route — routes derived from the hub registry (compact IA).
// Legacy deep links (the old per-feature paths) redirect into the owning hub
// + tab, so bookmarks keep working.
const legacy: RouteRecordRaw[] = [
  { path: '/brand', redirect: (to) => ({ path: '/settings', query: { ...to.query, tab: 'brand' } }) },
  { path: '/home', redirect: (to) => ({ path: '/dashboard', query: to.query }) },
  { path: '/planner', redirect: (to) => ({ path: '/plan', query: { ...to.query, tab: 'monthly' } }) },
  {
    path: '/planner/:month',
    redirect: (to) => ({ path: '/plan', query: { ...to.query, tab: 'monthly', m: String(to.params.month) } }),
  },
  { path: '/calendar', redirect: (to) => ({ path: '/plan', query: { ...to.query, tab: 'calendar' } }) },
  { path: '/ideas', redirect: (to) => ({ path: '/plan', query: { ...to.query, tab: 'ideas' } }) },
  { path: '/hashtags', redirect: (to) => ({ path: '/plan', query: { ...to.query, tab: 'hashtags' } }) },
  { path: '/feed', redirect: (to) => ({ path: '/content', query: { ...to.query, tab: 'feed' } }) },
  {
    path: '/studio/:id?',
    redirect: (to) => ({
      path: '/content',
      query: { ...to.query, tab: 'studio', ...(typeof to.params.id === 'string' && to.params.id ? { id: to.params.id } : {}) },
    }),
  },
  { path: '/live', redirect: (to) => ({ path: '/dashboard', query: to.query }) },
  {
    path: '/campaigns/:id?',
    redirect: (to) => ({
      path: '/promote',
      query: { ...to.query, tab: 'campaigns', ...(typeof to.params.id === 'string' && to.params.id ? { id: to.params.id } : {}) },
    }),
  },
  { path: '/ads', redirect: (to) => ({ path: '/promote', query: { ...to.query, tab: 'ads' } }) },
  { path: '/performance', redirect: (to) => ({ path: '/analyze', query: { ...to.query, tab: 'performance' } }) },
  { path: '/finance', redirect: (to) => ({ path: '/analyze', query: { ...to.query, tab: 'finance' } }) },
  { path: '/members', redirect: (to) => ({ path: '/settings', query: { ...to.query, tab: 'members' } }) },
]

const routes: RouteRecordRaw[] = [
  { path: '/', redirect: (to) => ({ path: screens[0].to, query: to.query }) },
  { path: '/profile', redirect: (to) => ({ path: '/', query: to.query }) },
  { path: '/setup', redirect: (to) => ({ path: '/', query: to.query }) },
  ...screens.map((s) => ({ path: s.path, component: s.component })),
  ...legacy,
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
  // Design-system gallery (dev reference).
  { path: '/components', component: ComponentsPage, meta: { standalone: true, public: true } },
  { path: '/:pathMatch(.*)*', component: NotFoundPage },
]

const router = createRouter({ history: createWebHashHistory(), routes })

// Signed-in users skip the onboarding pages (but may revisit /plans).
router.beforeEach((to) => {
  if (currentUser.value && (to.path === '/login' || to.path === '/register')) return '/'
  return true
})

export default router
