import { createRouter, createWebHistory } from 'vue-router'
import DocView from './views/DocView.vue'

export default createRouter({
  history: createWebHistory(),
  routes: [
    { path: '/', redirect: '/index' },
    { path: '/:slug', component: DocView, props: true },
  ],
  scrollBehavior: () => ({ top: 0 }),
})
