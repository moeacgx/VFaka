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
    meta: { requiresAuth: true },
    children: [
      {
        path: '',
        redirect: '/admin/dashboard',
      },
      {
        path: 'dashboard',
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
        path: 'coupons',
        name: 'Coupons',
        component: () => import('../views/admin/Coupons.vue'),
      },
      {
        path: 'payment',
        name: 'PaymentConfig',
        component: () => import('../views/admin/PaymentConfig.vue'),
        meta: { requiresSuperAdmin: true },
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
        meta: { requiresSuperAdmin: true },
      },
      {
        path: 'settings',
        name: 'Settings',
        component: () => import('../views/admin/Settings.vue'),
        meta: { requiresSuperAdmin: true },
      },
    ],
  },
]

const router = createRouter({
  history: createWebHistory(),
  routes,
})

function getAdminRole(): string {
  const token = localStorage.getItem('admin_token')
  if (!token) return 'admin'
  try {
    const payload = token.split('.')[1]
    const decoded = JSON.parse(atob(payload))
    return decoded.role || 'admin'
  } catch {
    return 'admin'
  }
}

router.beforeEach((to, _from, next) => {
  if (to.matched.some(r => r.meta.requiresAuth)) {
    const token = localStorage.getItem('admin_token')
    if (!token) {
      next('/admin/login')
      return
    }
    // Check super_admin guard
    if (to.matched.some(r => r.meta.requiresSuperAdmin)) {
      const role = getAdminRole()
      if (role !== 'super_admin') {
        next('/admin/dashboard')
        return
      }
    }
  }
  if (to.path === '/admin/login' && localStorage.getItem('admin_token')) {
    next('/admin/dashboard')
    return
  }
  next()
})

export default router
