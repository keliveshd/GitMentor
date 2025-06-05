import { createRouter, createWebHistory } from 'vue-router'
import Layout from '../components/Layout.vue'

const routes = [
  {
    path: '/',
    component: Layout,
    redirect: '/dashboard',
    children: [
      {
        path: '/dashboard',
        name: 'Dashboard',
        component: () => import('../views/Dashboard.vue'),
        meta: { title: '仪表板', icon: 'Dashboard' }
      },
      {
        path: '/repositories',
        name: 'Repositories',
        component: () => import('../views/Repositories.vue'),
        meta: { title: '仓库管理', icon: 'Folder' }
      },
      {
        path: '/repository/:id',
        name: 'RepositoryDetail',
        component: () => import('../views/RepositoryDetail.vue'),
        meta: { title: '仓库详情', icon: 'Document' }
      },
      {
        path: '/commits',
        name: 'Commits',
        component: () => import('../views/Commits.vue'),
        meta: { title: '提交历史', icon: 'List' }
      },
      {
        path: '/contributors',
        name: 'Contributors',
        component: () => import('../views/Contributors.vue'),
        meta: { title: '贡献者', icon: 'User' }
      },
      {
        path: '/reports',
        name: 'Reports',
        component: () => import('../views/Reports.vue'),
        meta: { title: '报告', icon: 'Document' }
      },
      {
        path: '/settings',
        name: 'Settings',
        component: () => import('../views/Settings.vue'),
        meta: { title: '设置', icon: 'Setting' }
      }
    ]
  }
]

const router = createRouter({
  history: createWebHistory(),
  routes
})

export default router
