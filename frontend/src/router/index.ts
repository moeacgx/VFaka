import { createRouter, createWebHistory } from 'vue-router'

const routes = [
  {
    path: '/',
    name: 'Home',
    component: () => import('../views/public/Home.vue'),
  },
  {
    path: '/order',
    name: 'OrderQuery',
    component: () => import('../views/public/OrderQuery.vue'),
  },
  {
    path: '/aff',
    name: 'Aff',
    component: () => import('../views/public/Aff.vue'),
  },
  // Admin routes
  {
    path: '/admin/login',
    name: 'AdminLogin',
    component: () => import('../views/admin/Login.vue'),
  },
  {
    path: '/admin',
    component: () => import('../views/admin/Layout.vue'),
    children: [
      {
        path: '',
        name: 'Dashboard',
        component: () => import('../views/admin/Dashboard.vue'),
      },
      {
        path: 'products',
        name: 'Products',
        component: () => import('../views/admin/Products.vue'),
      },
      {
        path: 'categories',
        name: 'Categories',
        component: () => import('../views/admin/Categories.vue'),
      },
      {
        path: 'cards',
        name: 'Cards',
        component: () => import('../views/admin/Cards.vue'),
      },
      {
        path: 'orders',
        name: 'Orders',
        component: () => import('../views/admin/Orders.vue'),
      },
      {
        path: 'payment',
        name: 'PaymentConfig',
        component: () => import('../views/admin/PaymentConfig.vue'),
      },
      {
        path: 'aff',
        name: 'AffManage',
        component: () => import('../views/admin/AffManage.vue'),
      },
      {
        path: 'withdrawals',
        name: 'Withdrawals',
        component: () => import('../views/admin/Withdrawals.vue'),
      },
      {
        path: 'admins',
        name: 'Admins',
        component: () => import('../views/admin/Admins.vue'),
      },
      {
        path: 'settings',
        name: 'Settings',
        component: () => import('../views/admin/Settings.vue'),
      },
    ],
  },
]

const router = createRouter({
  history: createWebHistory(),
  routes,
})

export default router
